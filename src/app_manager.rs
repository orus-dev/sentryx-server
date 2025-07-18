use std::env;
use std::fs;
use std::fs::remove_file;
use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub repo: String,
    pub branch: String,
    pub install_command: String,
    pub run_command: String,
}

impl App {
    /// Converts URL to "user/repo"
    pub fn id(&self) -> Option<String> {
        let url = self.repo.trim_end_matches(".git");

        if url.starts_with("git@") {
            // Format: git@github.com:usr/repo
            url.split_once(':').map(|(_, path)| path.to_string())
        } else if url.starts_with("https://") || url.starts_with("http://") {
            // Format: https://github.com/usr/repo
            url.split_once("github.com/")
                .map(|(_, path)| path.to_string())
        } else {
            None
        }
    }

    /// Converts URL to "user-repo"
    pub fn id_system(&self) -> Option<String> {
        let url = self.repo.trim_end_matches(".git");

        let repo_path = if url.starts_with("git@") {
            url.split_once(':').map(|(_, path)| path)
        } else if url.starts_with("https://") || url.starts_with("http://") {
            url.split_once("github.com/").map(|(_, path)| path)
        } else {
            None
        }?;

        Some(repo_path.replace('/', "-"))
    }

    pub fn repo_folder_name(&self) -> String {
        self.repo
            .trim_end_matches(".git")
            .rsplit('/')
            .next()
            .unwrap_or("unknown")
            .to_string()
    }
}

fn home_dir() -> PathBuf {
    let home = env::var("HOME").expect("Failed to get HOME environment variable");
    PathBuf::from(home)
}

fn apps_dir() -> PathBuf {
    let home = env::var("HOME").expect("Failed to get HOME environment variable");
    PathBuf::from(home).join("sentryx/apps")
}

fn apps_file_path() -> PathBuf {
    apps_dir().join("apps.json")
}

pub fn init() -> Vec<App> {
    let path = apps_file_path();

    if !path.exists() {
        let _ = fs::create_dir_all(path.parent().unwrap());
        let _ = fs::write(&path, "[]");
    }

    let content = fs::read_to_string(&path).unwrap_or_else(|_| "[]".to_string());
    serde_json::from_str(&content).unwrap_or_else(|_| vec![])
}

pub fn save_apps(apps: &[App]) {
    let path = apps_file_path();
    let json = serde_json::to_string_pretty(apps).expect("Failed to serialize apps");

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    fs::write(path, json).expect("Failed to write apps.json");
}

pub fn uninstall_app(apps: &mut Vec<App>, id: &str) {
    if let Some(index) = apps
        .iter()
        .position(|app| app.id().expect("Unable to obtain id") == id)
    {
        let app = &apps[index];
        let repo_dir = apps_dir().join(app.repo_folder_name());

        if repo_dir.exists() {
            if let Err(e) = fs::remove_dir_all(&repo_dir) {
                eprintln!("Failed to remove directory {:?}: {}", repo_dir, e);
            } else {
                println!("Removed app: {}", id);
            }
        }

        let user_systemd_dir = home_dir().join(".config/systemd/user");

        let service_path = user_systemd_dir.join(format!(
            "{}.service",
            app.id_system().expect("Unable to obtain id")
        ));

        remove_file(service_path).unwrap_or_else(|e| {
            eprintln!("Failed to remove service file: {}", e);
        });

        apps.remove(index);
        save_apps(apps);
    } else {
        eprintln!("App not found: {}", id);
    }
}

pub fn install_app(apps: &mut Vec<App>, app: App) {
    let path = apps_dir();
    let repo_name = app.repo_folder_name();

    // Ensure apps directory exists
    if let Err(e) = fs::create_dir_all(&path) {
        eprintln!("Failed to create apps directory: {}", e);
        return;
    }

    if let Err(e) = env::set_current_dir(&path) {
        eprintln!("Failed to switch to apps directory: {}", e);
        return;
    }

    println!("Installing app: {}", app.id().expect("Unable to obtain id"));
    println!("→ Cloning branch `{}` from `{}`", app.branch, app.repo);

    let clone_status = Command::new("git")
        .arg("clone")
        .arg("-b")
        .arg(&app.branch)
        .arg(&app.repo)
        .status();

    if !matches!(clone_status, Ok(s) if s.success()) {
        eprintln!("Failed to clone repository");
        return;
    }

    let repo_path = path.join(&repo_name);
    if let Err(e) = env::set_current_dir(&repo_path) {
        eprintln!("Failed to change into cloned repo directory: {}", e);
        return;
    }

    println!(
        "→ Running install command in {:?}",
        env::current_dir().unwrap()
    );

    let install_status = Command::new("bash")
        .arg("-c")
        .arg(&app.install_command)
        .status();

    if !matches!(install_status, Ok(s) if s.success()) {
        eprintln!("Install command failed");
    }

    let service_contents = format!(
        "[Unit]\nDescription={repo_name} (Installed with SentryX)\nAfter=network.target\n\n[Service]\nWorkingDirectory=/home/server/saturn-server\nExecStart={}\nRestart=always\nRestartSec=5\nStandardOutput=journal\nStandardError=journal\n\n[Install]\nWantedBy=multi-user.target",
        app.run_command,
    );

    let user_systemd_dir = home_dir().join(".config/systemd/user");

    fs::create_dir_all(&user_systemd_dir).expect("Failed to create user systemd dir");

    let service_path = user_systemd_dir.join(format!(
        "{}.service",
        app.id_system().expect("Unable to obtain id")
    ));

    println!("→ Creating service file: {service_path:?}");

    if let Err(e) = fs::write(&service_path, service_contents) {
        eprintln!(
            "Failed to write service file, make sure you run this server with sudo permissions: {}",
            e
        );
        return;
    }

    // Add to app list and save
    apps.push(app);
    save_apps(apps);
}

pub fn toggle_app_state(id: &str, enable: bool) {
    Command::new("systemctl")
        .arg("--user")
        .arg(if enable { "enable" } else { "disable" })
        .arg(id)
        .status()
        .expect("Failed to toggle app enable state");
}

pub fn start_app(id: &str) {
    Command::new("systemctl")
        .arg("--user")
        .arg("start")
        .arg(id)
        .status()
        .expect("Failed to start app");
}

pub fn stop_app(id: &str) {
    Command::new("systemctl")
        .arg("--user")
        .arg("stop")
        .arg(id)
        .status()
        .expect("Failed to stop app");
}

pub fn restart_app(id: &str) {
    Command::new("systemctl")
        .arg("restart")
        .arg(id)
        .status()
        .expect("Failed to restart app");
}
