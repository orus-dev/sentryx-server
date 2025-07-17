pub mod server_config;

use std::net::TcpListener;
use std::sync::Arc;
use std::thread::spawn;
use sysinfo::{Disks, Networks, System};
use tungstenite::accept;

use crate::server_config::ServerStats;
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

fn main() {
    let server = TcpListener::bind("127.0.0.1:5273").unwrap();
    for stream in server.incoming() {
        spawn(move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            let config = Arc::new(server_config::ServerConfig::new());

            {
                let master_key = websocket.read().unwrap();

                if !master_key.is_text()
                    || master_key.to_text().unwrap() != config.master_key.clone()
                {
                    websocket
                        .send(tungstenite::Message::text("!Invalid master key"))
                        .unwrap();

                    return;
                }

                websocket
                    .send(tungstenite::Message::text("Success"))
                    .unwrap();
            }

            let mut sys = System::new_all();
            let mut networks = Networks::new();
            let mut disks = Disks::new();

            while websocket.can_write() {
                sys.refresh_all();
                std::thread::sleep(std::time::Duration::from_millis(200));
                sys.refresh_all();
                networks.refresh(true);
                disks.refresh(true);

                websocket
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
