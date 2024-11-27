use std::{fs, path::Path};

use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

use crate::config;

const DB_URL: &str = "sqlite://.tern/tern.db";

pub struct Database {
    db: Option<sqlx::Pool<Sqlite>>,
}

pub enum Protocol {
    Creation,
    Access,
}

impl Database {
    pub fn new() -> Self {
        Self { db: None }
    }

    pub async fn init(&mut self) -> Protocol {
        if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
            fs::create_dir(".tern").unwrap();
            match Sqlite::create_database(DB_URL).await {
                Ok(_) => Protocol::Creation,
                Err(err) => panic!("Could not create database: {}", err),
            }
        } else {
            Protocol::Access
        }
    }

    pub async fn connect(&mut self) {
        self.db = Some(SqlitePool::connect(DB_URL).await.unwrap());
    }

    pub async fn setup(&self) {
        // sqlx::query(include_str!(concat!(
        //     env!("CARGO_MANIFEST_DIR"),
        //     "/db/blueprint.sql"
        // )))
        // .execute(self.db.as_ref().unwrap())
        // .await
        // .unwrap();
        sqlx::migrate::Migrator::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations"))
            .await
            .unwrap()
            .run(self.db.as_ref().unwrap())
            .await
            .unwrap();
    }

    pub async fn fetch_profiles(&self) -> Vec<config::Profile> {
        sqlx::query_as::<_, config::Profile>(
            r#"
SELECT
    p.id,
    p.engine,
    p.source_path,
    p.source_file_extension,
    p.output_path,
    p.output_file_extension,
    p.options,
    p.ignore_patterns,
    m.file,
    m.mtime
FROM profiles p
LEFT JOIN metadata m ON p.metadata_index = m.idx;
        "#,
        )
        .fetch_all(self.db.as_ref().unwrap())
        .await
        .unwrap()
    }

    pub async fn store(&self) {}
}
