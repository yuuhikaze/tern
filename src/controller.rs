use std::{
    collections::BTreeMap,
    fs::{self, ReadDir},
    path::PathBuf,
    sync::{Arc, LazyLock, Mutex},
};

use clap::Parser;
use directories::ProjectDirs;
use tokio::task;

pub enum ModelEvent {
    ReadEvent,
    WriteEvent,
}

pub enum AgentEvent {
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

pub trait ModelMessageBroker {
    fn send_write_event(tx: tokio::sync::oneshot::Sender<ModelEvent>);
    fn send_read_event(tx: tokio::sync::oneshot::Sender<ModelEvent>);
}

pub trait AgentMessageBroker {
    fn send_get_column_event(
        tx: tokio::sync::mpsc::Sender<AgentEvent>,
        column: Arc<Mutex<Vec<String>>>,
        kind: String,
    );
    fn send_get_profiles_event(
        tx: tokio::sync::mpsc::Sender<AgentEvent>,
        profile: Arc<Mutex<Vec<Profile>>>,
    );
    fn send_store_profile_event(tx: tokio::sync::mpsc::Sender<AgentEvent>, profile: Arc<Profile>);
    fn send_quit_event(tx: tokio::sync::mpsc::Sender<AgentEvent>);
}

pub struct Controller;

impl ModelMessageBroker for Controller {
    fn send_write_event(tx: tokio::sync::oneshot::Sender<ModelEvent>) {
        if tx.send(ModelEvent::WriteEvent).is_err() {
            panic!("Receiver dropped before message [DatabaseEvent::WriteEvent] could be sent",);
        }
    }

    fn send_read_event(tx: tokio::sync::oneshot::Sender<ModelEvent>) {
        if tx.send(ModelEvent::ReadEvent).is_err() {
            panic!("Receiver dropped before message [DatabaseEvent::ReadEvent] could be sent",);
        }
    }
}

impl AgentMessageBroker for Controller {
    fn send_get_column_event(
        tx: tokio::sync::mpsc::Sender<AgentEvent>,
        column: Arc<Mutex<Vec<String>>>,
        kind: String,
    ) {
        task::spawn(async move {
            if (tx
                .send(AgentEvent::ReadEvent(ReadEvent::GetColumn(
                    Arc::clone(&column),
                    kind,
                )))
                .await)
                .is_err()
            {
                panic!("Receiver dropped before message [AgentEvent::ReadEvent(ReadEvent::GetColumn(arc))] could be sent");
            }
        });
    }

    fn send_get_profiles_event(
        tx: tokio::sync::mpsc::Sender<AgentEvent>,
        profile: Arc<Mutex<Vec<Profile>>>,
    ) {
        task::spawn(async move {
            if (tx
                .send(AgentEvent::ReadEvent(ReadEvent::GetProfiles(Arc::clone(
                    &profile,
                ))))
                .await)
                .is_err()
            {
                panic!("Receiver dropped before message [AgentEvent::ReadEvent(ReadEvent::GetProfiles(arc))] could be sent");
            }
        });
    }

    fn send_store_profile_event(tx: tokio::sync::mpsc::Sender<AgentEvent>, profile: Arc<Profile>) {
        task::spawn(async move {
            if (tx
                .send(AgentEvent::WriteEvent(WriteEvent::StoreProfile(Some(
                    profile,
                ))))
                .await)
                .is_err()
            {
                panic!("Receiver dropped before message [AgentEvent::WriteEvent(WriteEvent::SaveProfile(arc))] could be sent");
            }
        });
    }

    fn send_quit_event(tx: tokio::sync::mpsc::Sender<AgentEvent>) {
        task::spawn(async move {
            if (tx.send(AgentEvent::Quit).await).is_err() {
                println!("Receiver dropped before message [AgentEvent::Quit] could be sent");
            }
        });
    }
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

pub fn get_converters_dir() -> PathBuf {
    CONVERTERS_DIR.clone()
}

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
    #[arg(short, long, default_value_t = true)]
    pub ignore_hidden_files: bool,
}

pub struct DatabaseArgs {
    pub profile_manager: bool,
}

pub struct InterfaceArgs {
    pub tui: bool,
}

pub struct ConverterArgs {
    pub hidden: bool,
}
