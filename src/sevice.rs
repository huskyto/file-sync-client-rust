
use crate::util::Util;
use crate::model::ChangePatch;
use crate::model::FileDefinition;


pub struct SyncService;
impl SyncService {
    pub fn create_empty(fd: &FileDefinition) -> Result<FileDefinition, String> {
        let url = Util::build_url("api/v1/file");
        let client = reqwest::blocking::Client::new();
        let js = serde_json::to_string(&fd).expect("Serialization error.");
        match client.post(url)
                .body(js)
                .header("Content-Type", "application/json")
                .send() {
            Ok(res) => {
                if res.status() == 201 {
                    let new_id = res.headers().get("location")
                            .expect("Location not present")
                            .to_str().unwrap().to_string();
                    let mut new_fd = fd.clone();
                    new_fd.id = Some(new_id);
                    Ok(new_fd)
                }
                else {
                    Err("Wrong status".to_string())
                }
            },
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn update_file_fd(file_definition: &FileDefinition) -> Result<bool, String> {
        let local_path = Util::full_path(file_definition);
        let file_id = file_definition.clone().id.expect("No id in File Definition");
        println!("Local path: {local_path}");
        let content = std::fs::read(local_path).expect("Failed to read local file");
        let url = Util::build_url(&format!("api/v1/file/{file_id}"));
        let client = reqwest::blocking::Client::new();
        match client.put(url)
                .body(content)
                .send() {
            Ok(res) => {
                if res.status() == 202 {
                    Ok(true)
                }
                else {
                    Err("Wrong status".to_string())
                }
            },
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn get_file(file_def: &FileDefinition)  -> Result<Vec<u8>, String> {
        let file_id = file_def.id.as_ref().expect("Missing file id!");
        let url = Util::build_url(&format!("api/v1/file/{}", file_id));
        let client = reqwest::blocking::Client::new();
        match client.get(url)
                .send() {
            Ok(res) => {
                if res.status() == 200 {
                    let body = res.bytes().expect("Erro getting bytes").to_vec();
                    Ok(body)
                }
                else {
                    Err("Wrong status".to_string())
                }
            },
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn delete_file(file_id: &str) -> Result<bool, String> {
        let url = Util::build_url(&format!("api/v1/file/{}", file_id));
        let client = reqwest::blocking::Client::new();
        match client.delete(url)
                .send() {
            Ok(res) => {
                if res.status() == 202 {
                    Ok(true)
                }
                else {
                    Err("Wrong status".to_string())
                }
            },
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn get_patch(rev: u64, file_list: Vec<FileDefinition>) -> Result<ChangePatch, String> {
        let url = Util::build_url(&format!("api/v1/patch/{rev}"));
        let client = reqwest::blocking::Client::new();
        let js = serde_json::to_string(&file_list).expect("Serialization error.");
        match client.post(url)
                .body(js)
                .header("Content-Type", "application/json")
                .send() {
            Ok(res) => {
                if res.status() == 200 {
                    let change_patch: ChangePatch = res.json().expect("Deserialization error!");
                    Ok(change_patch)
                }
                else {
                    let e = res.text().unwrap();
                    Err(format!("Wrong status: {e}"))
                }
            },
            Err(e) => Err(e.to_string()),
        }
    }

}
