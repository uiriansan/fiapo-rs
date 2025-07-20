use sqlite;

pub struct Server {
    sources: Vec<String>,
}

impl Server {
    pub fn new() -> Self {
        let mut vec = Vec::new();
        vec.push("".to_string());
        Server { sources: vec }
    }

    pub fn load_sources() {} // ...from db

    pub fn load_source() {}

    pub fn create_source() {}
}
