use mongodb::bson::doc;
use mongodb::Database;
use crate::URLMapping;

pub trait IDataBaseManager {
    fn new() -> Self;
    fn get_db(&self) -> Database;
    fn set_db(&mut self, db: Database);
    fn dummy_read(&self) -> &'static str;
    async fn check_mapping(&self, link: String) -> String;
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

    async fn check_mapping(&self, link: String) -> String {
        let mapping: Option<URLMapping> = self.get_db().collection("mapping")
            .find_one(doc! {"short_link": link}, None).await.expect("No such mapping");

        match mapping {
            None => "".to_string(),
            Some(_) => mapping.unwrap().url
        }
    }
}
