mod db;
mod models;

use std::collections::HashMap;
use std::string::ToString;
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};
use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use actix_web::web::Redirect;
use mongodb::{Client, Database};
use once_cell::sync::Lazy;
use rand::{distributions::Alphanumeric, Rng};
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::db::{DataBaseManager, IDataBaseManager};
use crate::models::{ApiMessage, Claims, CreateShortLinkRequest, LoginRequest, URLMapping};

const SECRET: &[u8] = b"6bx2s/abJMMjWD+fPRsxsFfGj2Luwdvt";

static DATABASE: Lazy<Mutex<Option<DataBaseManager>>> = Lazy::new(|| {
    Mutex::new(None)
});

static URL_MAP: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
    let map: HashMap<String, String> = HashMap::new();
    Mutex::new(map)
});

#[get("/")]
async fn get_all_handler() -> impl Responder {
    let mut response: String = "".to_string();
    let url_map = URL_MAP.lock().unwrap();
    url_map.iter().for_each(|entry| {
        response = response.clone() + entry.0 + " => " + entry.1 + "\n";
    });
    HttpResponse::Ok().body(response)
}

#[post("/login/")]
async fn login_handler(req: web::Json<LoginRequest>) -> impl Responder {
    let username: String = req.username.clone();
    let password: String = req.password.clone();
    let db_manager: MutexGuard<Option<DataBaseManager>> = DATABASE.lock().unwrap();
    let saved_password: String = db_manager.as_ref().expect("")
        .find_user_by_username(username.clone()).await;
    if password == saved_password {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize
            + 3600; // Token valid for 1 hour

        let claims = Claims {
            sub: username.clone(),
            exp: expiration,
        };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET))
            .expect("Token generation failed");
        HttpResponse::Ok().content_type(mime::APPLICATION_JAVASCRIPT)
            .body(serde_json::to_string(
                &ApiMessage {
                    message: token.parse().unwrap()
                }).unwrap())
    }
    else {
        HttpResponse::Unauthorized().content_type(mime::APPLICATION_JAVASCRIPT)
            .body(serde_json::to_string(
                &ApiMessage {
                    message: "Wrong password".parse().unwrap()
                }).unwrap())
    }
}

#[post("/create/")]
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
        short_link: s.clone(),
    };
    let existing_short_link: String = db_manager.as_ref().unwrap().get_short_link_from_url(req_body.url.clone()).await;
    if existing_short_link.is_empty() {
        db_manager.as_ref().expect("DB cannot be null")
            .get_db().collection("mapping")
            .insert_one(mapping).await.expect("Insert failed");

        HttpResponse::Ok().content_type(mime::APPLICATION_JAVASCRIPT).body(serde_json::to_string(&URLMapping { url: req_body.url.clone(), short_link: s.clone() }).unwrap())
    }

    else {
        HttpResponse::Ok().content_type(mime::APPLICATION_JAVASCRIPT).body(serde_json::to_string(&URLMapping { url: req_body.url.clone(), short_link: existing_short_link }).unwrap())
    }


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
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .service(get_all_handler)
            .service(create_handler)
            .service(link_handler)
            .service(login_handler)
    })
        .bind(("127.0.0.1", 9191))?
        .run()
        .await
}