
use std::fs;
use std::fs::ReadDir;
use std::path::Path;
use std::collections::HashMap;

use crate::sevice::SyncService;
use crate::util::Util;
use crate::config::Config;
use crate::model::ChangeType;
use crate::model::FileChange;
use crate::model::LocalState;
use crate::model::FileDefinition;


pub struct StateManager {
    local_state: LocalState
}
impl StateManager {
    pub fn load_default() -> StateManager {
        let state = match Self::load_state() {
            Ok(state) => state,
            Err(_) => {
                println!("No state to load. Creating new empty one...");
                LocalState {
                    revision: 0,
                    files: Vec::new(),
                }
            },
        };

        Self { local_state: state }
    }
    pub fn get_revision(&self) -> u64 {
        self.local_state.revision
    }
    pub fn get_changes(&self) -> Vec<FileChange> {
        let mut res = Vec::new();
        let defs = self.get_local_file_definitions();
        let fs_defs = self.get_actual_fs_state();
        let mut defs_map: HashMap<&String, &FileDefinition> = defs.iter().map(|df| (df.id.as_ref().unwrap(), df)).collect();
        for fsd in fs_defs {
            if fsd.id.is_none() {
                res.push(FileChange::new(fsd.clone(), ChangeType::Create));
                continue;
            }

            let m_fd = *defs_map.get(&fsd.id.as_ref().unwrap()).unwrap();
            if Util::have_changed(&fsd, m_fd) {
                res.push(FileChange::new(fsd.clone(), ChangeType::Update));
            }
            defs_map.remove(m_fd.id.as_ref().unwrap());
        }

        for res_fd in defs_map.values() {
            res.push(FileChange::new((*res_fd).clone(), ChangeType::Delete));
        }

        res
    }
    fn get_local_file_definitions(&self) -> Vec<FileDefinition> {
        self.local_state.files.clone()
    }
    pub fn get_actual_fs_state(&self) -> Vec<FileDefinition> {
        let base_path = &Config::get_base_path();
        let root_path = Path::new(base_path).to_str().unwrap();
        let paths = fs::read_dir(root_path).unwrap();
        let all_files = Self::materialize_dir_defs(paths);
        let mut res: Vec<FileDefinition> = all_files.iter()
                .map(|f| Util::get_local_file_definition(f).unwrap())
                .collect();
        res.iter_mut().for_each(|fd| fd.id = self.get_id_for_file(fd));

        res
    }
    fn materialize_dir_defs(rd: ReadDir) -> Vec<String> {
        let mut res = Vec::new();
        for f in rd {
            let path = f.unwrap().path();
            if path.is_dir() {
                let sub_folder = fs::read_dir(path).unwrap();
                Self::materialize_dir_defs(sub_folder).iter()
                        .for_each(|sp| res.push(sp.to_string()));
            }
            else {
                if path.file_name().unwrap() == ".sync-state" { continue }
                res.push(path.to_str().unwrap().to_string());
            }
        }

        res
    }
    fn get_id_for_file(&self, fd: &FileDefinition) -> Option<String> {
        self.local_state.files.iter()
                .find(|f| f.name == fd.name && f.path == fd.path)
                .and_then(|f| f.id.clone())
    }

    fn get_save_state_path() -> String {
        let base_path = &Config::get_base_path();
        let binding = Path::new(base_path).join(".sync-state");
        binding.to_str().unwrap().to_string()
    }
    fn save_state(&self) -> Result<(), std::io::Error> {
        let state_str = serde_json::to_string(&self.local_state).expect("Serialization error.");
        let path = Self::get_save_state_path();
        std::fs::write(path, state_str.as_bytes())
    }
    fn load_state() -> Result<LocalState, std::io::Error> {
        let path = Self::get_save_state_path();
        match std::fs::read(path) {
            Ok(data) => {
                let stored_state: LocalState = serde_json::from_slice(&data).unwrap();
                Ok(stored_state)
            },
            Err(e) => Err(e),
        }
    }

            // TODO remove pub
    pub fn do_local_sync(&mut self) {
        let changes = self.get_changes();
        println!("Changes {:#?}", changes);
        for change in changes {
            match change.change {
                ChangeType::Create => self.sync_create_file(&change),
                ChangeType::Update => self.sync_update_file(&change),
                ChangeType::Delete => self.sync_delete_file(&change),
                ChangeType::DoDownload => { }, // noop locally
                ChangeType::DoUpload => { }, // noop locally
            }
        }

        let _ = self.save_state();
    }

            // TODO remove pub
    pub fn do_remote_sync(&mut self) {
        let local_fs_defs = self.get_local_file_definitions();
        let patch_res = SyncService::get_patch(self.get_revision(), local_fs_defs);
        if let Err(e) = patch_res {
            println!("Remote sync failed to retrieve patch: {e}");
            return;
        }
        let patch = patch_res.unwrap();
        println!("Remote changes {:#?}", patch);

        for change in patch.changes {
            match change.change {
                ChangeType::Create => self.sync_create_file(&change),
                ChangeType::Update => { },  // Noop. Dos handle it.
                ChangeType::Delete => self.sync_delete_local_file(&change),
                ChangeType::DoDownload => self.sync_download_file(&change),
                ChangeType::DoUpload => self.sync_upload_file(&change),
            }
        }

        self.local_state.revision = patch.revision;
        let _ = self.save_state();

            // Check if it's in sync, and update rev number:
        let local_fs_defs = self.get_local_file_definitions();
        let patch_res = SyncService::get_patch(self.get_revision(), local_fs_defs);
        if let Err(e) = patch_res {
            println!("Remote sync failed to retrieve 2nd patch: {e}");
            return;
        }
        let patch = patch_res.unwrap();
        if !patch.changes.is_empty() {
            println!("Still out of sync after update with remote_sync!")
        }
        else {
            self.local_state.revision = patch.revision;
            let _ = self.save_state();
        }
    }


    fn sync_create_file(&mut self, change: &FileChange) {
        let res = SyncService::create_empty(&change.file);
        match res {
            Ok(fd) => {
                self.local_state.add_file(&fd);
                let res = SyncService::update_file_fd(&fd);
                match res {
                    Ok(_) => { },
                    Err(e) => println!("Error on update: {e}"),
                }
            },
            Err(e) => println!("Error on create: {e}")
        }
    }
    fn sync_update_file(&mut self, change: &FileChange) {
        let res = SyncService::update_file_fd(&change.file);
        match res {
            Ok(_) => {
                self.local_state.update_file(&change.file);
            },
            Err(e) => println!("Error on update: {e}"),
        }
    }
    fn sync_delete_file(&mut self, change: &FileChange) {
        let file_id = change.file.id.as_ref().unwrap();
        let res = SyncService::delete_file(file_id);
        match res {
            Ok(_) => {
                self.local_state.remove_file(&change.file);
            },
            Err(e) => println!("Error on delete: {e}"),
        }
    }
    fn sync_download_file(&mut self, change: &FileChange) {
        let res = SyncService::get_file(&change.file);
        match res {
            Ok(data) => {
                let write_success = Util::write_file_content(&data, &change.file);
                if write_success {
                    self.local_state.add_or_update_file(&change.file);
                }
            },
            Err(e) => println!("Error on file download: {e}"),
        }
    }
    fn sync_upload_file(&mut self, change: &FileChange) {
        let res = SyncService::update_file_fd(&change.file);
        match res {
            Ok(_) => {
                self.local_state.update_file(&change.file);
            },
            Err(e) => println!("Error on file upload: {e}"),
        }
    }
    fn sync_delete_local_file(&mut self, change: &FileChange) {
        self.local_state.files.retain(|f| f.id != change.file.id);
        let full_path = Util::full_path(&change.file);
        let del_res = std::fs::remove_file(&full_path);
        if let Err(e) = del_res {
            println!("[Error] Error deleting local file: {} - {}", &full_path, e)
        }
    }
}
