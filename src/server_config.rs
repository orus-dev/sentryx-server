use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub master_key: String,
}

impl ServerConfig {
    pub fn new() -> Self {
        std::fs::read_to_string("./server.json")
            .map(|data| serde_json::from_str(&data).unwrap())
            .unwrap_or_else(|_| ServerConfig {
                master_key: "master_key".to_string(),
            })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerStats {
    pub memory: u8,
    pub cpu: u8,
    pub disk: u8,
    pub network: u64,
}
