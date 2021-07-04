#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_sync_db_pools;

use diesel::prelude::*;
use eureka_client::{BaseConfig, EurekaClient, PortData};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

pub mod schema;

#[database("database")]
pub(crate) struct Database(PgConnection);

#[launch]
fn rocket() -> _ {
    env_logger::init();

    rocket::build()
        .mount(
            "/api",
            routes![
                ##ROUTES##
            ],
        )
        .mount("/management", routes![healthcheck])
        .manage(init_eureka(
            "admin:admin@jhipster-registry".to_string(),
            8761,
            "##NAME##".to_string(),
            8081,
        ))
        .attach(Database::fairing())
}

##MODELS##

#[get("/health")]
async fn healthcheck() {}

fn init_eureka(
    server_host: String,
    server_port: u16,
    instance_ip_addr: String,
    instance_port: u16,
) -> EurekaClient {
    let mut config = BaseConfig::default();
    config.eureka.host = server_host;
    config.eureka.port = server_port;

    config.instance.ip_addr = instance_ip_addr;
    config.instance.port = Some(PortData::new(instance_port, true));
    config.instance.app = "##NAME##".to_string();
    config.instance.vip_address = "##NAME##".to_string();
    config.instance.secure_vip_address = "##NAME##".to_string();
    let eureka = EurekaClient::new(config);
    eureka.start();
    eureka
}
