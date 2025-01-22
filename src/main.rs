use model::FileDefinition;
use sevice::SyncService;


mod model;
mod sevice;
mod util;
mod config;

fn main() {
    quick_test();
}


fn quick_test() {
    let file_def = FileDefinition::new("Filename01".to_string(), "Path02".to_string());
    let file_def = SyncService::create_empty(&file_def).expect("Create empty failed");
    let new_id = file_def.id.as_ref().expect("Id was not updated");
    println!("New id: {new_id}");

    let is_updated = SyncService::update_file(new_id, "tmp/test-file.txt").expect("Error updating file");
    println!("Is updated? {is_updated}");

    let file_data = SyncService::get_file(&file_def).expect("Failed to get file data!");
    println!("Retrieved data: {}", &String::from_utf8(file_data).unwrap());

    let is_deleted = SyncService::delete_file(new_id).expect("Failed to delete file!");
    println!("Is deleted? {is_deleted}");

    let did_panic = std::panic::catch_unwind(|| SyncService::get_file(&file_def).expect("Getting deleted failed!")).is_err();
    println!("Did retrieve non existant panic? {did_panic}");
}