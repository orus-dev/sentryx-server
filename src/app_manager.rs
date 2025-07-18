use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

#[derive(Clone, Serialize, Deserialize)]
pub struct App {
    pub repo: String,
    pub branch: String,
    pub install_command: String,
    pub run_command: String,
    pub enabled: Option<bool>,
}

pub fn remove_app(apps: &mut Vec<App>, index: usize) {
    if let Some(app) = apps.get(index) {
        let home = env::var("HOME").expect("Failed to get HOME environment variable");
        let apps_dir = PathBuf::from(home).join("sentryx/apps");
        let repo_name = app
            .repo
            .trim_end_matches(".git")
            .rsplit('/')
            .next()
            .expect("Failed to extract repo name");
        fs::remove_dir_all(&apps_dir.join(repo_name)).expect("Failed to remove app directory");
        println!("Removed app: {}", app.repo);
    } else {
        println!("App not found: {}", index);
    }

    apps.remove(index);
    save_apps(apps);
}

pub fn install_app(apps: &mut Vec<App>, app: App) {
    let a2 = app.clone();
    // Get home directory and create ~/sentryx/apps if needed
    let home = env::var("HOME").expect("Failed to get HOME environment variable");
    let apps_dir = PathBuf::from(home).join("sentryx/apps");

    if !apps_dir.exists() {
        fs::create_dir_all(&apps_dir).expect("Failed to create apps directory");
    }

    // Change to apps directory
    env::set_current_dir(&apps_dir).expect("Failed to change to apps directory");

    println!("Installing app from repo: {}", app.repo);
    println!("Branch: {}", app.branch);
    println!("Install command: {}", app.install_command);
    println!("Run command: {}", app.run_command);

    // Clone the repo
    println!("Cloning repo");
    Command::new("git")
        .arg("clone")
        .arg("-b")
        .arg(&app.branch)
        .arg(&app.repo)
        .status()
        .expect("Failed to clone repository");

    // Extract repo name from URL (e.g., "https://github.com/user/myapp.git" -> "myapp")
    let repo_name = app
        .repo
        .trim_end_matches(".git")
        .rsplit('/')
        .next()
        .expect("Failed to extract repo name");

    // Change to cloned repo directory
    env::set_current_dir(&repo_name).expect("Failed to change to cloned repo directory");

    // You are now inside the cloned repo
    println!("Changed directory to {:?}", env::current_dir().unwrap());

    apps.push(a2.clone());
    save_apps(&apps);

    Command::new("bash")
        .arg("-c")
        .arg(app.install_command)
        .status()
        .expect("Failed to execute Bash command");

    run_app(a2);
}

pub fn run_app(app: App) {
    // Change to the app directory
    let home = env::var("HOME").expect("Failed to get HOME environment variable");
    let app_dir = PathBuf::from(home)
        .join("sentryx/apps")
        .join(app.repo.trim_end_matches(".git").split('/').last().unwrap());

    env::set_current_dir(&app_dir).expect("Failed to change to app directory");

    // Run the app
    Command::new("bash")
        .arg("-c")
        .arg(app.run_command)
        .status()
        .expect("Failed to execute run command");
}

pub fn run_apps() -> Vec<App> {
    // Change to the app directory
    let home = env::var("HOME").expect("Failed to get HOME environment variable");
    let apps_dir = PathBuf::from(home).join("sentryx/apps");

    if !apps_dir.exists() {
        fs::create_dir_all(&apps_dir).expect("Failed to create apps directory");
        fs::write(apps_dir.join("apps.json"), "[]").expect("Failed to create apps.json");
    }

    let apps_json = serde_json::from_str::<Vec<App>>(
        &fs::read_to_string(apps_dir.join("apps.json")).expect("Failed to read apps.json"),
    )
    .expect("Failed to parse apps.json");

    for app in apps_json.clone() {
        if app.enabled.unwrap_or(true) {
            println!("Running app: {}", app.repo);
            std::thread::spawn(|| {
                run_app(app);
            });
        } else {
            println!("Skipping disabled app: {}", app.repo);
        }
    }

    apps_json
}

pub fn save_apps(apps: &[App]) {
    let home = env::var("HOME").expect("Failed to get HOME environment variable");
    let apps_dir = PathBuf::from(home).join("sentryx/apps");

    fs::write(
        apps_dir.join("apps.json"),
        serde_json::to_string(apps).expect("Failed to serialize apps"),
    )
    .expect("Failed to write apps.json");
}
