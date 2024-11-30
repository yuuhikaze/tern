use std::{collections::BTreeMap, fs, path::Path, sync::Arc};

use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

use crate::controller::{DatabaseEvent, Profile};

const DB_URL: &str = "sqlite://.tern/tern.db";

pub struct Database {
    db: Option<sqlx::Pool<Sqlite>>,
}

impl Database {
    pub fn new() -> Self {
        Self { db: None }
    }

    pub async fn init(&mut self) -> DatabaseEvent {
        if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
            fs::create_dir(".tern").unwrap();
            match Sqlite::create_database(DB_URL).await {
                Ok(_) => DatabaseEvent::Write,
                Err(err) => panic!("Could not create database: {}", err),
            }
        } else {
            DatabaseEvent::Read
        }
    }

    pub async fn connect(&mut self) {
        self.db = Some(SqlitePool::connect(DB_URL).await.unwrap());
    }

    pub async fn setup(&self) {
        sqlx::migrate::Migrator::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations"))
            .await
            .unwrap()
            .run(self.db.as_ref().unwrap())
            .await
            .unwrap();
    }

    pub async fn fetch_profiles(&self) -> Vec<Profile> {
        let raw_profiles = sqlx::query("SELECT * FROM profiles")
            .fetch_all(self.db.as_ref().unwrap())
            .await
            .unwrap();
        let profiles_future = raw_profiles.into_iter().map(async |row| {
            use sqlx::Row;
            let id: u32 = row.try_get("id").unwrap();
            let try_get_row_as_vector = |column| {
                // &str: data received from database
                // String: parse target
                match row.try_get::<String, &str>(column) {
                    Ok(it) if !it.is_empty() => Some(it.lines().map(|e| e.into()).collect()),
                    Ok(_) => None,
                    Err(err) => panic!("Could not retrieve {}: {}", column, err),
                }
            };
            let options = try_get_row_as_vector("options");
            let ignore_patterns = try_get_row_as_vector("ignore_patterns");
            let metadata = sqlx::query("SELECT file, mtime FROM metadata WHERE id = $1;")
                .bind(id)
                .fetch_all(self.db.as_ref().unwrap())
                .await
                .unwrap()
                .into_iter()
                .fold(Option::<BTreeMap<String, i32>>::None, |acc, row| {
                    let file: String = row.try_get("file").unwrap();
                    let mtime: i32 = row.try_get("mtime").unwrap();
                    let mut map = acc.unwrap_or_default();
                    map.insert(file, mtime);
                    Some(map)
                });
            Profile {
                engine: row.try_get("engine").unwrap(),
                source_path: row.try_get("source_path").unwrap(),
                source_file_extension: row.try_get("source_file_extension").unwrap(),
                output_path: row.try_get("output_path").unwrap(),
                output_file_extension: row.try_get("output_file_extension").unwrap(),
                options,
                ignore_patterns,
                metadata,
            }
        });
        futures::future::join_all(profiles_future).await
    }

    pub async fn save_profile(&self, mut profile: Option<Arc<Profile>>) {
        let profile_arc = profile.take().unwrap();
        if let Ok(profile) = Arc::try_unwrap(profile_arc) {
            sqlx::query(
                r#"
INSERT INTO profiles(engine, source_path, source_file_extension, output_path, output_file_extension)
VALUES
    ($1, $2, $3, $4, $5)
"#,
            )
            .bind(profile.engine)
            .bind(profile.source_path)
            .bind(profile.source_file_extension)
            .bind(profile.output_path)
            .bind(profile.output_file_extension)
            .execute(self.db.as_ref().unwrap())
            .await
            .unwrap();
        }
    }
}
