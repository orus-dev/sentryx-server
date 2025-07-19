mod app_manager;
mod server_config;
use actix_web::{get, post, web, App, HttpServer, Responder, Result};
use sysinfo::{Disks, Networks, System};

use crate::server_config::ServerStats;

#[get("/")]
async fn sys_info() -> Result<impl Responder> {
    let mut sys = System::new_all();
    let mut disks: Disks = Disks::new_with_refreshed_list();
    let mut networks: Networks = Networks::new_with_refreshed_list();

    networks.refresh(true);
    disks.refresh(true);

    sys.refresh_all();

    std::thread::sleep(std::time::Duration::from_millis(200));

    sys.refresh_cpu_all();

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

    Ok(web::Json(ServerStats {
        memory: (sys.used_memory() as f32 / sys.total_memory() as f32 * 100.0) as u8,
        cpu: average_cpu_usage as u8,
        disk: disk_usage_percentage,
        network: network_usage,
    }))
}

#[get("/apps")]
async fn get_apps() -> Result<impl Responder> {
    let apps = app_manager::init();
    Ok(web::Json(apps))
}

#[get("/app/{app_index}")]
async fn get_app(id: web::Path<usize>) -> Result<impl Responder> {
    let apps = app_manager::init();
    match apps.get(*id) {
        Some(app) => Ok(web::Json(app.clone())),
        None => Err(actix_web::error::ErrorNotFound("App not found")),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(sys_info)
            .service(get_apps)
            .service(get_app)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
