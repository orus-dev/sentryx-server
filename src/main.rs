pub mod app_manager;
pub mod server_config;

use crate::app_manager::{install_app, App};
use crate::server_config::ServerStats;
use std::sync::Arc;
use std::thread::spawn;
use std::{net::TcpListener, sync::Mutex};
use sysinfo::{Disks, Networks, System};
use tungstenite::accept;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum Method {
    Install(App),
    SetEnabled(usize, bool),
    Edit(usize, App),
}

fn main() {
    let apps = app_manager::init();

    let server = TcpListener::bind("127.0.0.1:5273").unwrap();
    for stream in server.incoming() {
        let mut apps = apps.clone();

        spawn(move || {
            let websocket = Arc::new(Mutex::new(accept(stream.unwrap()).unwrap()));
            let reader_socket = Arc::clone(&websocket);
            let config = Arc::new(server_config::ServerConfig::new());

            {
                let master_key = reader_socket.lock().unwrap().read().unwrap();

                if !master_key.is_text()
                    || master_key.to_text().unwrap() != config.master_key.clone()
                {
                    websocket
                        .lock()
                        .unwrap()
                        .send(tungstenite::Message::text("!Invalid master key"))
                        .unwrap();

                    return;
                }

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

                match mth {
                    Method::Install(app) => {
                        install_app(&mut apps, app);
                    }
                    Method::SetEnabled(i, b) => {
                        if let Some(app) = apps.get_mut(i) {
                        } else {
                            eprintln!("App index {} not found", i);
                        }
                        app_manager::save_apps(&apps);
                    }
                    Method::Edit(i, app) => {
                        if let Some(existing_app) = apps.get_mut(i) {
                            *existing_app = app;
                        } else {
                            eprintln!("App index {} not found", i);
                        }
                        app_manager::save_apps(&apps);
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
