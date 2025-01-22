use serde_json::json;

use crate::{model::FileDefinition, util::Util};


pub struct SyncService;
impl SyncService {
    pub fn create_empty(fd: FileDefinition) -> Result<String, String> {
        let url = Util::build_url("api/v1/file");
        let client = reqwest::blocking::Client::new();
        let js = serde_json::to_string(&fd).expect("Serialization error.");
        match client.post(url)
                .body(js)
                .header("Content-Type", "application/json")
                .send() {
            Ok(res) => {
                if res.status() == 201 {
                    let new_id = res.headers().get("location").expect("Location not present");
                    Ok(new_id.to_str().unwrap().to_string())
                }
                else {
                    Err("Wrong status".to_string())
                }
            },
            Err(e) => Err(e.to_string()),
        }
    }


}
