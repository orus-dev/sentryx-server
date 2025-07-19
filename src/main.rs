pub mod app_manager;
pub mod server_config;

use crate::server_config::ServerStats;
use std::sync::Arc;
use std::thread::spawn;
use std::{net::TcpListener, sync::Mutex};
use sysinfo::{Disks, Networks, System};
use tungstenite::accept;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum Method {
    Install(app_manager::App),
    Edit(usize, app_manager::App),
    Uninstall(usize),
    Toggle(usize, bool),
    Start(usize),
    Stop(usize),
    Restart(usize),
}

fn main() {
    println!("SentryX server starting...");
    let apps = app_manager::init();
    println!("Loaded {} apps", apps.len());

    let server = TcpListener::bind("0.0.0.0:5273").unwrap();
    println!("Listening on 127.0.0.1:5273");
    for stream in server.incoming() {
        let mut apps = apps.clone();

        spawn(move || {
            let websocket = Arc::new(Mutex::new(accept(stream.unwrap()).unwrap()));
            let reader_socket = Arc::clone(&websocket);
            let config = Arc::new(server_config::ServerConfig::new());
            println!("Client connected, awaiting master key...");

            {
                let master_key = reader_socket.lock().unwrap().read().unwrap();

                if !master_key.is_text()
                    || master_key.to_text().unwrap() != config.master_key.clone()
                {
                    println!("Invalid master key from client");
                    websocket
                        .lock()
                        .unwrap()
                        .send(tungstenite::Message::text("!Invalid master key"))
                        .unwrap();
                    return;
                }

                println!("Client authenticated successfully");

                websocket
                    .lock()
                    .unwrap()
                    .send(tungstenite::Message::text("Success"))
                    .unwrap();
            }

            std::thread::spawn(move || loop {
                let mth: Method = serde_json::from_str(
                    &reader_socket
                        .lock()
                        .unwrap()
                        .read()
                        .unwrap()
                        .to_text()
                        .unwrap(),
                )
                .unwrap();

                println!("Received command: {:?}", mth);

                match mth {
                    Method::Install(app) => {
                        println!("→ Installing app: {}", app.repo);
                        app_manager::install_app(&mut apps, app);
                    }
                    Method::Edit(i, app) => {
                        println!("→ Editing app at index {}: {}", i, app.repo);
                        if let Some(existing_app) = apps.get_mut(i) {
                            *existing_app = app;
                            app_manager::save_apps(&apps);
                        }
                    }
                    Method::Uninstall(i) => {
                        println!("→ Uninstalling app at index {}", i);
                        app_manager::uninstall_app(&mut apps, i);
                    }
                    Method::Toggle(index, enable) => {
                        println!("→ Toggling app at index {} to {}", index, enable);
                        app_manager::toggle_app_state(&apps, index, enable);
                    }
                    Method::Start(index) => {
                        println!("→ Starting app at index {}", index);
                        app_manager::start_app(&apps, index);
                    }
                    Method::Stop(index) => {
                        println!("→ Stopping app at index {}", index);
                        app_manager::stop_app(&apps, index);
                    }
                    Method::Restart(index) => {
                        println!("→ Restarting app at index {}", index);
                        app_manager::restart_app(&apps, index);
                    }
                }
            });

            let mut sys = System::new_all();
            let mut networks = Networks::new();
            let mut disks = Disks::new();

            while websocket.lock().unwrap().can_write() {
                sys.refresh_all();
                std::thread::sleep(std::time::Duration::from_millis(200));
                sys.refresh_all();
                networks.refresh(true);
                disks.refresh(true);

                println!("→ Sending system stats update...");
                websocket
                    .lock()
                    .unwrap()
                    .send(tungstenite::Message::text(
                        serde_json::to_string(&get_system_info(&mut sys, &networks, &disks))
                            .unwrap(),
                    ))
                    .unwrap();

                std::thread::sleep(std::time::Duration::from_secs(2));
            }
            println!("Client disconnected, cleaning up...");
        });
    }
}

fn get_system_info(sys: &mut System, networks: &Networks, disks: &Disks) -> ServerStats {
    let cpus = sys.cpus();
    let total_cpu_usage: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
    let average_cpu_usage = total_cpu_usage / cpus.len() as f32;

    let network_usage: u64 = networks
        .iter()
        .map(|(_, data)| data.received() + data.transmitted())
        .sum();

    let total_disk_space: u64 = disks.iter().map(|d| d.total_space()).sum();
    let available_disk_space: u64 = disks.iter().map(|d| d.available_space()).sum();

    let used_disk_space = total_disk_space.saturating_sub(available_disk_space);
    let disk_usage_percentage = if total_disk_space == 0 {
        0
    } else {
        ((used_disk_space as f32 / total_disk_space as f32) * 100.0) as u8
    };

    ServerStats {
        memory: (sys.used_memory() as f32 / sys.total_memory() as f32 * 100.0) as u8,
        cpu: average_cpu_usage as u8,
        disk: disk_usage_percentage,
        network: network_usage,
    }
}
