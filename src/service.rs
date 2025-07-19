use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Service {
    pub name: Option<String>,
    pub description: Option<String>,
    pub loaded_status: Option<String>,
    pub unit_file_path: Option<String>,
    pub enabled: Option<String>,
    pub preset: Option<String>,
    pub active: Option<String>,
    pub active_state: Option<String>,
    pub trigger: Option<String>,
    pub triggers: Option<String>,
    pub docs: Option<String>,
    // Other fields can be added as needed
}

lazy_static! {
    static ref SERVICE_RE: Regex = Regex::new(
        r#"(?x)
        ^●\s(?P<name>\S+)\s-\s(?P<description>.+?)\n
        \s+Loaded:\s(?P<loaded_status>\S+)\s\((?P<unit_file_path>[^;]+)(?:;\s(?P<enabled>\S+))?(?:;\s+preset:\s(?P<preset>\S+))?\)\n
        \s+Active:\s(?P<active>\S+)\s\((?P<active_state>[^)]+)\).*\n
        (?:\s+Trigger:\s(?P<trigger>.+)\n)?
        (?:\s+Triggers:\s●\s(?P<triggers>[^\n]+)\n)?
        (?:\s+Docs:\s(?P<docs>(?:.+\n(?:\s{7,}.+\n)*))?)?
        "#
    ).unwrap();
}

pub fn parse_service(input: String) -> Service {
    if let Some(cap) = SERVICE_RE.captures(&input) {
        Service {
            name: cap.name("name").map(|m| m.as_str().to_string()),
            description: cap.name("description").map(|m| m.as_str().to_string()),
            loaded_status: cap.name("loaded_status").map(|m| m.as_str().to_string()),
            unit_file_path: cap.name("unit_file_path").map(|m| m.as_str().to_string()),
            enabled: cap.name("enabled").map(|m| m.as_str().to_string()),
            preset: cap.name("preset").map(|m| m.as_str().to_string()),
            active: cap.name("active").map(|m| m.as_str().to_string()),
            active_state: cap.name("active_state").map(|m| m.as_str().to_string()),
            trigger: cap.name("trigger").map(|m| m.as_str().to_string()),
            triggers: cap.name("triggers").map(|m| m.as_str().to_string()),
            docs: cap.name("docs").map(|m| m.as_str().trim().to_string()),
        }
    } else {
        // Return an empty Service or handle error as needed
        Service {
            name: None,
            description: None,
            loaded_status: None,
            unit_file_path: None,
            enabled: None,
            preset: None,
            active: None,
            active_state: None,
            trigger: None,
            triggers: None,
            docs: None,
        }
    }
}
