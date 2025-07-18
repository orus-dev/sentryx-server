pub mod app_manager;
pub mod server_config;

use crate::app_manager::{get_app_by_id, get_app_by_id_mut, install_app, App};
use crate::server_config::ServerStats;
use std::sync::Arc;
use std::thread::spawn;
use std::{net::TcpListener, sync::Mutex};
use sysinfo::{Disks, Networks, System};
use tungstenite::accept;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum Method {
    Install(App),
    SetEnabled(String, bool),
    Edit(String, App),
    Uninstall(String),
    Toggle(String, bool),
    Start(String),
    Stop(String),
    Restart(String),
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
                        let app = get_app_by_id_mut(&mut apps, &i).expect("App not found");
                        if let Some(id) = app.id_system() {
                            app_manager::toggle_app_state(&format!("{}.service", id), b);
                        }
                        app_manager::save_apps(&apps);
                    }
                    Method::Edit(i, app) => {
                        let existing_app = get_app_by_id_mut(&mut apps, &i).expect("App not found");
                        *existing_app = app;
                        app_manager::save_apps(&apps);
                    }
                    Method::Uninstall(id) => {
                        app_manager::uninstall_app(&mut apps, &id);
                    }
                    Method::Toggle(id, enable) => {
                        let id = get_app_by_id(&mut apps, &id)
                            .expect("App not found")
                            .id_system()
                            .unwrap();
                        app_manager::toggle_app_state(&format!("{}.service", id), enable);
                    }
                    Method::Start(id) => {
                        let id = get_app_by_id(&mut apps, &id)
                            .expect("App not found")
                            .id_system()
                            .unwrap();
                        app_manager::start_app(&format!("{}.service", id));
                    }
                    Method::Stop(id) => {
                        let id = get_app_by_id(&mut apps, &id)
                            .expect("App not found")
                            .id_system()
                            .unwrap();
                        app_manager::stop_app(&format!("{}.service", id));
                    }
                    Method::Restart(id) => {
                        let id = get_app_by_id(&mut apps, &id)
                            .expect("App not found")
                            .id_system()
                            .unwrap();
                        app_manager::restart_app(&format!("{}.service", id));
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
