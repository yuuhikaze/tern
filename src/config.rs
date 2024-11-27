use std::{collections::BTreeMap, fs::{self, ReadDir}, path::PathBuf, sync::LazyLock};

use directories::ProjectDirs;
use sqlx::{sqlite::SqliteRow, FromRow};

#[derive(Clone, Debug)]
pub struct Profile {
    engine: String,
    source_path: String,
    source_file_extension: String,
    output_path: String,
    output_file_extension: String,
    options: Option<Vec<String>>,
    ignore_patterns: Option<Vec<String>>,
    metadata: Option<BTreeMap<String, i32>>
}

impl<'r> FromRow<'r, SqliteRow> for Profile {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        let engine = row.try_get("engine")?;
        let source_path = row.try_get("source_path")?;
        let source_file_extension = row.try_get("source_file_extension")?;
        let output_path = row.try_get("output_path")?;
        let output_file_extension = row.try_get("output_file_extension")?;
        // &str: data received from database
        // String: parse target
        let options: Option<Vec<String>> = match row.try_get::<String, &str>("options") {
            Ok(it) if !it.is_empty() => Some(it.lines().map(|e| e.into()).collect()),
            Ok(_) => None,
            Err(err) => panic!("Could not retrieve options: {}", err),
        };
        let ignore_patterns: Option<Vec<String>> = match row.try_get::<String, &str>("ignore_patterns") {
            Ok(it) if !it.is_empty() => Some(it.lines().map(|e| e.into()).collect()),
            Ok(_) => None,
            Err(err) => panic!("Could not retrieve ignore patterns: {}", err),
        };
        Ok(Profile {
            engine,
            source_path,
            source_file_extension,
            output_path,
            output_file_extension,
            options,
            ignore_patterns,
            metadata: None
        })
    }
}

static CONVERTERS_DIR: LazyLock<PathBuf> = LazyLock::new(|| ProjectDirs::from("com", "yuuhikaze", "tern").expect("Unable to determine project directories").data_dir().join("converters"));

// Creates data dir if it does not exist
pub fn create_data_dir() {
    fs::create_dir_all(&*CONVERTERS_DIR).unwrap();
}

// Returns an iterator over the data directory
pub fn read_data_dir() -> ReadDir {
    fs::read_dir(&*CONVERTERS_DIR).unwrap()
}
