use std::{
    collections::BTreeMap,
    fs::{self, ReadDir},
    path::PathBuf,
    sync::{Arc, LazyLock},
};

use clap::Parser;
use directories::ProjectDirs;
use tokio::sync::Mutex;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct ArgParser {
    #[arg(long, action)]
    pub tui: bool,
    #[arg(short, long, action)]
    pub profile_manager: bool,
}

pub struct DatabaseArgs {
    pub profile_manager: bool,
}

pub struct InterfaceArgs {
    pub tui: bool,
}

#[derive(Debug)]
pub enum DatabaseEvent {
    ReadEvent,
    WriteEvent,
}

pub enum ControlEvent {
    ReadEvent(ReadEvent),
    WriteEvent(WriteEvent),
    Quit,
}

pub enum ReadEvent {
    GetColumn(Arc<Mutex<Vec<String>>>, String),
    GetProfiles(Arc<Mutex<Vec<Profile>>>),
}

pub enum WriteEvent {
    SaveProfile(Option<Arc<Profile>>),
}

#[derive(Debug)]
pub struct Profile {
    pub engine: String,
    pub source_path: String,
    pub source_file_extension: String,
    pub output_path: String,
    pub output_file_extension: String,
    pub options: Option<Vec<String>>,
    pub ignore_patterns: Option<Vec<String>>,
    pub metadata: Option<BTreeMap<String, i32>>,
}

static CONVERTERS_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    ProjectDirs::from("com", "yuuhikaze", "tern")
        .expect("Unable to determine project directories")
        .data_dir()
        .join("converters")
});

// Creates data dir if it does not exist
pub fn create_data_dir() {
    fs::create_dir_all(&*CONVERTERS_DIR).unwrap();
}

// Returns an iterator over the data directory
pub fn read_data_dir() -> ReadDir {
    fs::read_dir(&*CONVERTERS_DIR).unwrap()
}
