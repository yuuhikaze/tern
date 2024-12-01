use std::{
    collections::BTreeMap,
    fs::{self, ReadDir},
    path::PathBuf,
    sync::{Arc, LazyLock, Mutex},
};

use clap::Parser;
use directories::ProjectDirs;
use tokio::task;

pub struct Controller;

impl Controller {
    pub fn send_write_event(tx: tokio::sync::oneshot::Sender<DatabaseEvent>) {
        if tx.send(DatabaseEvent::WriteEvent).is_err() {
            panic!("Receiver dropped before message [DatabaseEvent::WriteEvent] could be sent",);
        }
    }
    pub fn send_read_event(tx: tokio::sync::oneshot::Sender<DatabaseEvent>) {
        if tx.send(DatabaseEvent::ReadEvent).is_err() {
            panic!("Receiver dropped before message [DatabaseEvent::ReadEvent] could be sent",);
        }
    }
    pub fn send_quit_event(tx: tokio::sync::mpsc::Sender<ControlEvent>) {
        task::spawn(async move {
            if (tx.send(ControlEvent::Quit).await).is_err() {
                println!("Receiver dropped before message [ControlEvent::Quit] could be sent");
            }
        });
    }

    pub fn send_store_profile_event(
        tx: tokio::sync::mpsc::Sender<ControlEvent>,
        profile: Arc<Profile>,
    ) {
        task::spawn(async move {
            if (tx
                .send(ControlEvent::WriteEvent(WriteEvent::StoreProfile(Some(
                    profile,
                ))))
                .await)
                .is_err()
            {
                panic!("Receiver dropped before message [ControlEvent::WriteEvent(WriteEvent::SaveProfile(arc))] could be sent");
            }
        });
    }

    pub fn send_get_column_event(
        tx: tokio::sync::mpsc::Sender<ControlEvent>,
        column: Arc<Mutex<Vec<String>>>,
        kind: String,
    ) {
        let column_clone = Arc::clone(&column);
        task::spawn(async move {
            if (tx
                .send(ControlEvent::ReadEvent(ReadEvent::GetColumn(
                    column_clone,
                    kind,
                )))
                .await)
                .is_err()
            {
                panic!("Receiver dropped before message [ControlEvent::ReadEvent(ReadEvent::GetProfiles(arc))] could be sent");
            }
        });
    }
}

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
    StoreProfile(Option<Arc<Profile>>),
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
