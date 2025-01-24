
use std::path::Path;

use crate::model::FileDefinition;
use crate::config::Config;



const SERVER: &str = "localhost:8000";

pub struct Util;
impl Util {
    pub fn build_url(location: &str) -> String {
        format!("http://{}/{}", SERVER, location)
    }

    pub fn full_path(file_def: &FileDefinition) -> String {
        let path = Path::new(&Config::get_base_path())
                    .join(&file_def.path)
                    .join(&file_def.name);

        path.to_str().expect("Invalid path").to_string()
    }
    pub fn read_file_content(local_path: &str) -> Vec<u8> {
        std::fs::read(local_path).expect("Failed to read local file")
    }
    pub fn get_local_file_definition(full_path: &str) -> Option<FileDefinition> {
        let stripped_path = Path::new(full_path)
                .strip_prefix(Config::get_base_path())
                .unwrap().to_str().unwrap();
        let (path, name) = Self::split_full_path(stripped_path);
        let content = Self::read_file_content(full_path);
        let checksum = Self::checksum(&content);
        let last_update = std::fs::metadata(full_path).expect("Failed to retrieve file metadata")
                    .modified().expect("Failed to retrieve modified date");

        Some(FileDefinition::new_full_no_id(name, path, content.len() as u64, checksum, last_update))
    }
    pub fn split_full_path(full_path: &str) -> (String, String) {
        let _path = Path::new(full_path);
        let path = _path.parent().expect("Invalid path")
                .to_str().expect("Invalid path")
                .to_string();
        let name = _path.file_name().expect("Invalid file name")
                .to_str().expect("Invalid file name")
                .to_string();

        (path, name)
    }
    pub fn checksum(content: &[u8]) -> String {
        let digest = md5::compute(content);
        format!("{:x}", digest)
    }
    pub fn have_changed(a: &FileDefinition, b: &FileDefinition) -> bool{
        a.name != b.name
                || a.path != b.path
                || a.size != b.size
    }
}
