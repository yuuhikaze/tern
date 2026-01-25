use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Condvar, Mutex},
};

use filetime::FileTime;
use sqlx::{migrate::MigrateDatabase, Row, Sqlite, SqlitePool};
use tokio::sync::oneshot::Sender;

use crate::controller::{Controller, DatabaseArgs, ModelEvent, ModelMessageBroker, Profile};

const DB_URL: &str = "sqlite://.tern/store.db";

pub struct Database {
    tx: Option<Sender<ModelEvent>>,
    args: Option<DatabaseArgs>,
    db: Option<sqlx::Pool<Sqlite>>,
}

impl Database {
    pub fn new(tx: Sender<ModelEvent>, args: DatabaseArgs) -> Self {
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
                Ok(_) => Controller::send_write_event(tx).await,
            };
        } else if !self.args.as_ref().unwrap().profile_manager {
            Controller::send_read_event(tx).await;
        } else {
            Controller::send_write_event(tx).await;
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

    pub async fn get_column(&self, column: Arc<(Mutex<Vec<String>>, Condvar)>, kind: &str) {
        *column.0.lock().unwrap() = sqlx::query(&format!("SELECT {} FROM profiles", kind))
            .fetch_all(self.db.as_ref().unwrap())
            .await
            .unwrap()
            .into_iter()
            .map(|row| row.try_get(kind).unwrap())
            .collect();
        column.1.notify_one();
    }

    pub async fn get_profiles(&self, profile: Arc<(Mutex<Vec<Profile>>, Condvar)>) {
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
            let metadata =
                sqlx::query("SELECT source_file, mtime FROM metadata WHERE profile_id = $1")
                    .bind(id)
                    .fetch_all(self.db.as_ref().unwrap())
                    .await
                    .unwrap()
                    .into_iter()
                    .fold(Option::<BTreeMap<String, i64>>::None, |acc, row| {
                        let file = row.try_get("source_file").unwrap();
                        let mtime = row.try_get("mtime").unwrap();
                        let mut map = acc.unwrap_or_default();
                        map.insert(file, mtime);
                        Some(map)
                    });
            Profile {
                id: row.try_get("id").unwrap(),
                engine: row.try_get("engine").unwrap(),
                source_root: row.try_get("source_root").unwrap(),
                source_file_extension: row.try_get("source_file_extension").unwrap(),
                output_root: row.try_get("output_root").unwrap(),
                output_file_extension: row.try_get("output_file_extension").unwrap(),
                options,
                ignore_patterns,
                metadata,
            }
        });
        *profile.0.lock().unwrap() = futures::future::join_all(profiles_future).await;
        profile.1.notify_one();
    }

    pub async fn store_profile(&self, mut profile: Option<Arc<Profile>>) {
        let profile_arc = profile.take().unwrap();
        if let Ok(profile) = Arc::try_unwrap(profile_arc) {
            let flatten_vector = |v: Option<Vec<String>>| v.map(|option| option.join("\n"));
            let options = flatten_vector(profile.options);
            let ignore_patterns = flatten_vector(profile.ignore_patterns);
            sqlx::query(
                r#"
INSERT INTO profiles(engine, source_root, source_file_extension, output_root, output_file_extension, options, ignore_patterns)
VALUES
    ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(profile.engine)
            .bind(profile.source_root)
            .bind(profile.source_file_extension)
            .bind(profile.output_root)
            .bind(profile.output_file_extension)
            .bind(options)
            .bind(ignore_patterns)
            .execute(self.db.as_ref().unwrap())
            .await
            .unwrap();
        }
    }

    /// 1. Inserts a new row if the source_file doesn't exist for this profile
    /// 2. Updates the mtime if the source_file already exists
    pub async fn update_metadata(&self, source_file: PathBuf, profile_id: u8) {
        let mtime = FileTime::from_last_modification_time(&fs::metadata(&source_file).unwrap())
            .unix_seconds();
        sqlx::query(
            r#"
INSERT INTO metadata (profile_id, source_file, mtime)
VALUES ($1, $2, $3)
ON CONFLICT(profile_id, source_file) 
DO UPDATE SET mtime = $3;
        "#,
        )
        .bind(profile_id)
        .bind(source_file.to_str().unwrap())
        .bind(mtime)
        .execute(self.db.as_ref().unwrap())
        .await
        .unwrap();
    }
}
