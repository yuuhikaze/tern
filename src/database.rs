use sqlx::{migrate::MigrateDatabase, Sqlite};

struct Profiles {
    engine: String,
    source_path: String,
    source_file_extension: String,
    output_path: String,
    output_file_extension: String,
    options: Vec<String>,
    ignore_patterns: Vec<String>,
}

struct TrackedFiles {
    mtime: i64,
}

const DB_URL: &str = "sqlite://tern.db";

pub struct Database;

pub enum Protocol {
    Creation,
    Access,
}

impl Database {
    pub fn new() -> Self {
        Self
    }

    pub async fn init(&self) -> Protocol {
        if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
            match Sqlite::create_database(DB_URL).await {
                Ok(_) => Protocol::Creation,
                Err(error) => panic!("Could not create database: {}", error),
            }
        } else {
            return Protocol::Access;
        }
    }
}
