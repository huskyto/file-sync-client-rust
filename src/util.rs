

const SERVER: &str = "localhost:8000";

pub struct Util;
impl Util {
    pub fn build_url(location: &str) -> String {
        format!("http://{}/{}", SERVER, location)
    }
}
