use model::FileDefinition;
use sevice::SyncService;


mod model;
mod sevice;
mod util;

fn main() {
    quick_test();
}


fn quick_test() {
    let file_def = FileDefinition::new("Filename01".to_string(), "Path02".to_string());
    let new_id = SyncService::create_empty(file_def).expect("Create empty failed");
    println!("New id: {new_id}");



}