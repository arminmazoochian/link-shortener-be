use mongodb::bson::doc;
use mongodb::Database;
use crate::{URLMapping};
use crate::models::User;

pub trait IDataBaseManager {
    fn new() -> Self;
    fn get_db(&self) -> Database;
    fn set_db(&mut self, db: Database);
    async fn check_mapping(&self, link: String) -> String;
    async fn get_short_link_from_url(&self, url: String) -> String;

    async fn find_user_by_username(&self, username: String) -> String;
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

    async fn check_mapping(&self, link: String) -> String {
        let mapping: Option<URLMapping> = self.get_db().collection("mapping")
            .find_one(doc! {"short_link": link}).await.expect("No such mapping");

        match mapping {
            None => "".to_string(),
            Some(_) => mapping.unwrap().url
        }
    }

    async fn get_short_link_from_url(&self, url: String) -> String {
        let mapping: Option<URLMapping> = self.get_db().collection("mapping")
            .find_one(doc! {"url": url}).await.expect("No such mapping");

        match mapping {
            None => "".to_string(),
            Some(_) => mapping.unwrap().short_link
        }
    }

    async fn find_user_by_username(&self, username: String) -> String {
        let user_opt: Option<User> = self.get_db().collection("user")
            .find_one(doc! {"username": username}).await.expect("No such user");

        match user_opt {
            None => "".to_string(),
            Some(_) => user_opt.unwrap().password
        }
    }
}
