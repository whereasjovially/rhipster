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
                
                patients_mod::get_patients,
                patients_mod::get_patient,
                patients_mod::post_patient,
                patients_mod::delete_patient,
                
                practitioners_mod::get_practitioners,
                practitioners_mod::get_practitioner,
                practitioners_mod::post_practitioner,
                practitioners_mod::delete_practitioner,
                
            ],
        )
        .mount("/management", routes![healthcheck])
        .manage(init_eureka(
            "admin:admin@jhipster-registry".to_string(),
            8761,
            "demo".to_string(),
            8081,
        ))
        .attach(Database::fairing())
}

use chrono::NaiveDate;
#[derive(Queryable, Serialize)]
pub struct Patient {
    pub id: i32,
    pub first: String,
    pub last: String,
    pub dob: NaiveDate,
}

#[derive(Queryable, Serialize)]
pub struct Practitioner {
    pub id: i32,
    pub first: String,
    pub last: String,
}

#[derive(Insertable, Deserialize)]
#[table_name = "patients"]
pub struct NewPatient {
    pub first: String,
    pub last: String,
    pub dob: NaiveDate,
}

#[derive(Insertable, Deserialize)]
#[table_name = "practitioners"]
pub struct NewPractitioner {
    pub first: String,
    pub last: String,
}

use schema::patients;
#[sourcegen::sourcegen(generator = "crud_generator", model = "patient")]
// Generated. All manual edits to the block annotated with #[sourcegen...] will be discarded.
mod patients_mod {
    use super::*;
    #[get("/patients")]
    pub(crate) async fn get_patients(db: Database) -> Json<Vec<Patient>> {
        let patients = db
            .run(|c| patients::table.load::<Patient>(c).unwrap())
            .await;
        Json(patients)
    }
    #[get("/patients/<id>")]
    pub(crate) async fn get_patient(db: Database, id: i32) -> Result<Json<Patient>, ()> {
        let patient = db
            .run(move |c| patients::table.find(id).first::<Patient>(c))
            .await
            .map_err(|_| ())?;
        Ok(Json(patient))
    }
    #[post("/patients", data = "<data>")]
    pub(crate) async fn post_patient(db: Database, data: Json<NewPatient>) -> Result<(), ()> {
        db.run(|c| {
            diesel::insert_into(patients::table)
                .values(data.into_inner())
                .execute(c)
        })
        .await
        .map(|_| ())
        .map_err(|_| ())
    }
    #[delete("/patients/<id>")]
    pub(crate) async fn delete_patient(db: Database, id: i32) -> Result<(), ()> {
        db.run(move |c| diesel::delete(patients::table.filter(patients::id.eq(id))).execute(c))
            .await
            .map(|_| ())
            .map_err(|_| ())
    }
}
use schema::practitioners;
#[sourcegen::sourcegen(generator = "crud_generator", model = "practitioner")]
// Generated. All manual edits to the block annotated with #[sourcegen...] will be discarded.
mod practitioners_mod {
    use super::*;
    #[get("/practitioners")]
    pub(crate) async fn get_practitioners(db: Database) -> Json<Vec<Practitioner>> {
        let practitioners = db
            .run(|c| practitioners::table.load::<Practitioner>(c).unwrap())
            .await;
        Json(practitioners)
    }
    #[get("/practitioners/<id>")]
    pub(crate) async fn get_practitioner(db: Database, id: i32) -> Result<Json<Practitioner>, ()> {
        let practitioner = db
            .run(move |c| practitioners::table.find(id).first::<Practitioner>(c))
            .await
            .map_err(|_| ())?;
        Ok(Json(practitioner))
    }
    #[post("/practitioners", data = "<data>")]
    pub(crate) async fn post_practitioner(
        db: Database,
        data: Json<NewPractitioner>,
    ) -> Result<(), ()> {
        db.run(|c| {
            diesel::insert_into(practitioners::table)
                .values(data.into_inner())
                .execute(c)
        })
        .await
        .map(|_| ())
        .map_err(|_| ())
    }
    #[delete("/practitioners/<id>")]
    pub(crate) async fn delete_practitioner(db: Database, id: i32) -> Result<(), ()> {
        db.run(move |c| {
            diesel::delete(practitioners::table.filter(practitioners::id.eq(id))).execute(c)
        })
        .await
        .map(|_| ())
        .map_err(|_| ())
    }
}

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
    config.instance.app = "demo".to_string();
    config.instance.vip_address = "demo".to_string();
    config.instance.secure_vip_address = "demo".to_string();
    let eureka = EurekaClient::new(config);
    eureka.start();
    eureka
}
