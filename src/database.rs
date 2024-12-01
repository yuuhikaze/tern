use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

use sqlx::{migrate::MigrateDatabase, Row, Sqlite, SqlitePool};
use tokio::sync::oneshot::Sender;

use crate::controller::{Controller, DatabaseArgs, DatabaseEvent, Profile};

const DB_URL: &str = "sqlite://.tern/tern.db";

pub struct Database {
    tx: Option<Sender<DatabaseEvent>>,
    args: Option<DatabaseArgs>,
    db: Option<sqlx::Pool<Sqlite>>,
}

impl Database {
    pub fn new(tx: Sender<DatabaseEvent>, args: DatabaseArgs) -> Self {
        Self {
            db: None,
            args: Some(args),
            tx: Some(tx),
        }
    }

    pub async fn connect(&mut self) {
        let tx = self.tx.take().unwrap();
        if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
            fs::create_dir(".tern").unwrap();
            match Sqlite::create_database(DB_URL).await {
                Err(err) => panic!("Could not create database: {}", err),
                Ok(_) => Controller::send_write_event(tx),
            };
        } else if !self.args.as_ref().unwrap().profile_manager {
            Controller::send_read_event(tx);
        } else {
            Controller::send_write_event(tx);
        }
        self.db = Some(SqlitePool::connect(DB_URL).await.unwrap());
    }

    pub async fn migrate(&self) {
        sqlx::migrate::Migrator::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations"))
            .await
            .unwrap()
            .run(self.db.as_ref().unwrap())
            .await
            .unwrap();
    }

    pub async fn get_column(&self, column: Arc<Mutex<Vec<String>>>, kind: &str) {
        *column.lock().unwrap() = sqlx::query(&format!("SELECT {} FROM profiles", kind))
            .fetch_all(self.db.as_ref().unwrap())
            .await
            .unwrap()
            .into_iter()
            .map(|row| row.try_get(kind).unwrap())
            .collect();
        println!("From DB: {:#?}", column);
    }

    pub async fn get_profiles(&self, profile: Arc<Mutex<Vec<Profile>>>) {
        let raw_profiles = sqlx::query("SELECT * FROM profiles")
            .fetch_all(self.db.as_ref().unwrap())
            .await
            .unwrap();
        let profiles_future = raw_profiles.into_iter().map(async |row| {
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
        *profile.lock().unwrap() = futures::future::join_all(profiles_future).await;
    }

    pub async fn store_profile(&self, mut profile: Option<Arc<Profile>>) {
        let profile_arc = profile.take().unwrap();
        if let Ok(profile) = Arc::try_unwrap(profile_arc) {
            let flatten_vector = |v: Option<Vec<String>>| v.map(|option| option.join("\n"));
            let options = flatten_vector(profile.options);
            let ignore_patterns = flatten_vector(profile.ignore_patterns);
            sqlx::query(
                r#"
INSERT INTO profiles(engine, source_path, source_file_extension, output_path, output_file_extension, options, ignore_patterns)
VALUES
    ($1, $2, $3, $4, $5, $6, $7)
"#,
            )
            .bind(profile.engine)
            .bind(profile.source_path)
            .bind(profile.source_file_extension)
            .bind(profile.output_path)
            .bind(profile.output_file_extension)
            .bind(options)
            .bind(ignore_patterns)
            .execute(self.db.as_ref().unwrap())
            .await
            .unwrap();
        }
    }
}
