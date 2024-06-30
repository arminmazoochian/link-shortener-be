use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use mongodb::{Client, Database};
use once_cell::sync::Lazy;
use serde::Deserialize;
use rand::{distributions::Alphanumeric, Rng};


static DATABASE: Lazy<Mutex<Option<DataBaseManager>>> = Lazy::new(|| {
    Mutex::new(None)
});

static URL_MAP: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("https://google.com".to_string(), "d47ewks".to_string());
    map.insert("https://mazoochian.com".to_string(), "sdhe62y".to_string());
    Mutex::new(map)
});


#[derive(Deserialize)]
struct CreateShortLinkRequest {
    url: String,
}

#[get("/")]
async fn hello() -> impl Responder {
    let mut response: String = "".to_string();
    let url_map = URL_MAP.lock().unwrap();
    url_map.iter().for_each(|entry| {
        response = response.clone() + entry.0 + " => " + entry.1 + "\n";
    });
    HttpResponse::Ok().body(response)
}

#[post("/create")]
async fn echo(req_body: web::Json<CreateShortLinkRequest>) -> impl Responder {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    let mut map: MutexGuard<HashMap<String, String>> = URL_MAP.lock().unwrap();
    map.insert(req_body.url.clone(), s);

    HttpResponse::Ok().body(req_body.url.clone())
}

#[get("/dummy")]
async fn dummy() -> impl Responder {
    let mut read_test: String = "".to_string();
    let db_manager: MutexGuard<Option<DataBaseManager>> = DATABASE.lock().unwrap();
    read_test = db_manager.as_ref().unwrap().dummy_read().to_string();
    HttpResponse::Ok().body(read_test)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

pub trait IDataBaseManager {
    fn new() -> Self;
    fn get_db(&self) -> Database;
    fn set_db(&mut self, db: Database);
    fn dummy_read(&self) -> &'static str;
}

pub struct DataBaseManager {
    db: Option<Database>,
}

impl IDataBaseManager for DataBaseManager {
    fn new() -> DataBaseManager {
        Self { db: None }
    }

    fn get_db(&self) -> Database {
        if self.db.clone().is_none() {
            panic!("Database is not initialized!");
        }
        self.db.clone().unwrap()
    }

    fn set_db(&mut self, db: Database) {
        self.db = Some(db);
    }

    fn dummy_read(&self) -> &'static str {
        "DUMMY_READ"
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut db_manager: DataBaseManager = DataBaseManager::new();
    let uri: &str = "mongodb://localhost:27017/";
    let client: Client = Client::with_uri_str(uri).await.expect("DB failed to open");
    let database: Database = client.database("link_shortener");
    db_manager.set_db(database);
    DATABASE.lock().unwrap().replace(db_manager);
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .service(dummy)
            .route("/hey", web::get().to(manual_hello))
    })
        .bind(("127.0.0.1", 9090))?
        .run()
        .await
}