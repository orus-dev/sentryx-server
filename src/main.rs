mod server_config;
mod service;
use actix_web::{
    get,
    web::{self},
    App, HttpServer, Responder, Result,
};
use serde_json::json;
use sysinfo::{Disks, Networks, System};

use crate::server_config::{validate_master_key, ServerStats};

#[derive(serde::Deserialize)]
struct Auth {
    auth: String,
}

#[get("/")]
async fn sys_info(query: web::Query<Auth>) -> Result<impl Responder> {
    if !validate_master_key(&query.auth) {
        return Err(actix_web::error::ErrorUnauthorized("Invalid master key"));
    }

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
async fn get_apps(query: web::Query<Auth>) -> Result<impl Responder> {
    if !validate_master_key(&query.auth) {
        return Err(actix_web::error::ErrorUnauthorized("Invalid master key"));
    }

    let systemctl = systemctl::SystemCtl::default();
    match systemctl.list_units(None, None, None) {
        Ok(units) => Ok(web::Json(
            units
                .iter()
                .filter(|i| i.ends_with(".service"))
                .map(|v| match systemctl.status(v) {
                    Ok(unit) => {
                        let a = service::parse_service(unit);
                        // In the case of debugging, i found out these have a pattern of .target or .device
                        // if a.all_fields_none() {
                        //     println!("\n--START--\n{v}\n--END--\n");
                        // }
                        a
                    }
                    Err(_) => service::parse_service(String::new()),
                })
                .filter(|i| !i.all_fields_none())
                .collect::<Vec<_>>(),
        )),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(format!(
            "Failed to list units: {}",
            e
        ))),
    }
}

#[get("/apps/{app}")]
async fn get_app(id: web::Path<String>, query: web::Query<Auth>) -> Result<impl Responder> {
    if !validate_master_key(&query.auth) {
        return Err(actix_web::error::ErrorUnauthorized("Invalid master key"));
    }

    let systemctl = systemctl::SystemCtl::default();
    match systemctl.status(&*id) {
        Ok(unit) => Ok(web::Json(
            serde_json::to_value(service::parse_service(unit)).unwrap(),
        )),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(format!(
            "Failed to get unit: {}",
            e
        ))),
    }
}

#[get("/enable/{app}")]
async fn enable_app(id: web::Path<String>, query: web::Query<Auth>) -> Result<impl Responder> {
    if !validate_master_key(&query.auth) {
        return Err(actix_web::error::ErrorUnauthorized("Invalid master key"));
    }

    let systemctl = systemctl::SystemCtl::default();
    match systemctl.enable(&*id) {
        Ok(status) => Ok(web::Json(json!({ "success": status.success() }))),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(format!(
            "Failed to enable unit: {}",
            e
        ))),
    }
}

#[get("/disable/{app}")]
async fn disable_app(id: web::Path<String>, query: web::Query<Auth>) -> Result<impl Responder> {
    if !validate_master_key(&query.auth) {
        return Err(actix_web::error::ErrorUnauthorized("Invalid master key"));
    }

    let systemctl = systemctl::SystemCtl::default();
    match systemctl.disable(&*id) {
        Ok(status) => Ok(web::Json(json!({ "success": status.success() }))),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(format!(
            "Failed to disable unit: {}",
            e
        ))),
    }
}

#[get("/start/{app}")]
async fn start_app(id: web::Path<String>, query: web::Query<Auth>) -> Result<impl Responder> {
    if !validate_master_key(&query.auth) {
        return Err(actix_web::error::ErrorUnauthorized("Invalid master key"));
    }

    let systemctl = systemctl::SystemCtl::default();
    match systemctl.start(&*id) {
        Ok(status) => Ok(web::Json(json!({ "success": status.success() }))),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(format!(
            "Failed to start unit: {}",
            e
        ))),
    }
}

#[get("/stop/{app}")]
async fn stop_app(id: web::Path<String>, query: web::Query<Auth>) -> Result<impl Responder> {
    if !validate_master_key(&query.auth) {
        return Err(actix_web::error::ErrorUnauthorized("Invalid master key"));
    }

    let systemctl = systemctl::SystemCtl::default();
    match systemctl.stop(&*id) {
        Ok(status) => Ok(web::Json(json!({ "success": status.success() }))),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(format!(
            "Failed to stop unit: {}",
            e
        ))),
    }
}

#[get("/restart/{app}")]
async fn restart_app(id: web::Path<String>, query: web::Query<Auth>) -> Result<impl Responder> {
    if !validate_master_key(&query.auth) {
        return Err(actix_web::error::ErrorUnauthorized("Invalid master key"));
    }

    let systemctl = systemctl::SystemCtl::default();
    match systemctl.restart(&*id) {
        Ok(status) => Ok(web::Json(json!({ "success": status.success() }))),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(format!(
            "Failed to restart unit: {}",
            e
        ))),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(sys_info)
            .service(get_apps)
            .service(get_app)
            .service(enable_app)
            .service(disable_app)
            .service(start_app)
            .service(stop_app)
            .service(restart_app)
    })
    .bind(("0.0.0.0", 63774))?
    .run()
    .await
}
