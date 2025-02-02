
use std::time::SystemTime;

use serde::Serialize;
use serde::Deserialize;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileDefinition {
    pub name: String,
    pub path: String,
    pub id: Option<String>,
    pub size: Option<u64>,
    pub checksum: Option<String>,
    pub last_update: Option<SystemTime>
}
impl FileDefinition {
    pub fn new(name: String, path: String) -> Self {
        Self {
            name,
            path,
            id: None,
            size: None,
            checksum: None,
            last_update: None
        }
    }
    pub fn new_full_no_id(name: String, path: String, size: u64, checksum: String, last_update: SystemTime) -> Self {
        Self {
            name,
            path,
            id: None,
            size: Some(size),
            checksum: Some(checksum),
            last_update: Some(last_update),
        }
    }
    pub fn set_to(&mut self, other: &FileDefinition) {
        self.name = other.name.clone();
        self.path = other.path.clone();
        self.size = other.size;
        self.checksum = other.checksum.clone();
        self.last_update = other.last_update;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileChange {
    pub file: FileDefinition,
    pub change: ChangeType
}
impl FileChange {
    pub fn new(file: FileDefinition, change: ChangeType) -> Self {
        Self {
            file,
            change
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangePatch {
    pub revision: u64,
    pub changes: Vec<FileChange>
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ChangeType {
    Create,
    Update,
    Delete,
    DoDownload,
    DoUpload
}

#[derive(Serialize, Deserialize)]
pub struct FileData {
    pub definition: FileDefinition,
    pub content: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct LocalState {
    pub revision: u64,
    pub files: Vec<FileDefinition>
}
impl LocalState {
    pub fn remove_file(&mut self, fd: &FileDefinition) {
        self.files.retain(|f| f.id != fd.id);
    }
    pub fn add_file(&mut self, fd: &FileDefinition) {
        self.files.push(fd.clone());
    }
    pub fn update_file(&mut self, fd: &FileDefinition) {
        let opt_f = self.files.iter_mut()
                .find(|f| f.id == fd.id);
        if let Some(f) = opt_f {
            f.set_to(fd);
        }
    }
    pub fn add_or_update_file(&mut self, fd: &FileDefinition) {
        let current = self.files
                .iter_mut()
                .find(|f| f.id == fd.id);
        match current {
            Some(f) => f.set_to(fd),
            None => self.add_file(fd),
        }
    }
}
