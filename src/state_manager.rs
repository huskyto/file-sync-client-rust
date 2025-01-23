
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

    pub fn do_local_sync(&mut self) {
        let changes = self.get_changes();
        println!("Changes {:#?}", changes);
        for change in changes {
            match change.change {
                ChangeType::Create => {
                    let res = SyncService::create_empty(&change.file);
                    match res {
                        Ok(fd) => {
                            self.local_state.add_file(&fd);
                            let res = SyncService::update_file_fd(&fd);
                            match res {
                                Ok(_) => {
                                    // self.local_state.update_file(fd);
                                },
                                Err(e) => println!("Error on update: {e}"),
                            }
                        },
                        Err(e) => println!("Error on create: {e}")
                    }
                },
                ChangeType::Update => {
                    let res = SyncService::update_file_fd(&change.file);
                    match res {
                        Ok(_) => {
                            // self.local_state.update_file(fd);
                        },
                        Err(e) => println!("Error on update: {e}"),
                    }
                },
                ChangeType::Delete => {
                    let file_id = change.file.id.as_ref().unwrap();
                    let res = SyncService::delete_file(file_id);
                    match res {
                        Ok(_) => {
                            self.local_state.remove_file(&change.file);
                        },
                        Err(e) => println!("Error on delete: {e}"),
                    }
                },
            }
        }

        let _ = self.save_state();
    }
}
