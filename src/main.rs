mod db;

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use actix_web::web::Redirect;
use mongodb::{Client, Database};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use rand::{distributions::Alphanumeric, Rng};
use crate::db::{DataBaseManager, IDataBaseManager};


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

#[derive(Serialize, Deserialize, Debug)]
struct URLMapping {
    url: String,
    short_link: String,
}

#[get("/")]
async fn get_all_handler() -> impl Responder {
    let mut response: String = "".to_string();
    let url_map = URL_MAP.lock().unwrap();
    url_map.iter().for_each(|entry| {
        response = response.clone() + entry.0 + " => " + entry.1 + "\n";
    });
    HttpResponse::Ok().body(response)
}

#[post("/create")]
async fn create_handler(req_body: web::Json<CreateShortLinkRequest>) -> impl Responder {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    let mut map: MutexGuard<HashMap<String, String>> = URL_MAP.lock().unwrap();
    map.insert(req_body.url.clone(), s.clone());
    let db_manager: MutexGuard<Option<DataBaseManager>> = DATABASE.lock().unwrap();
    let mapping = URLMapping {
        url: req_body.url.clone(),
        short_link: s,
    };
    db_manager.as_ref().expect("DB cannot be null")
        .get_db().collection("mapping")
        .insert_one(mapping, None).await.expect("Insert failed");


    HttpResponse::Ok().body(req_body.url.clone())
}

#[get("/{link}")]
async fn link_handler(req: HttpRequest) -> impl Responder {
    let link: String = req.match_info().get("link").unwrap().parse().unwrap();
    let db_manager: MutexGuard<Option<DataBaseManager>> = DATABASE.lock().unwrap();
    let url: String = db_manager.as_ref().unwrap().check_mapping(link).await.to_string();
    if url.is_empty() {
        return HttpResponse::NotFound().body("Couldn't find the link specified");
    }
    Redirect::to(url).respond_to(&req).map_into_boxed_body()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut db_manager: DataBaseManager = DataBaseManager::new();
    let uri: &str = "mongodb://localhost:27017/";
    let client: Client = Client::with_uri_str(uri).await.expect("DB failed to open");
    let database: Database = client.database("url-mapping");
    db_manager.set_db(database);
    DATABASE.lock().unwrap().replace(db_manager);
    HttpServer::new(|| {
        App::new()
            .service(get_all_handler)
            .service(create_handler)
            .service(link_handler)
    })
        .bind(("127.0.0.1", 9090))?
        .run()
        .await
}