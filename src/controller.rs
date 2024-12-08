use clap::Parser;
use directories::ProjectDirs;
use std::{
    collections::BTreeMap,
    fs::{self, ReadDir},
    path::PathBuf,
    sync::{Arc, Condvar, LazyLock, Mutex},
};
use tokio::runtime::Handle;

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
    GetColumn(Arc<(Mutex<Vec<String>>, Condvar)>, String),
    GetProfiles(Arc<(Mutex<Vec<Profile>>, Condvar)>),
}

pub enum WriteEvent {
    StoreProfile(Option<Arc<Profile>>),
}

pub trait ModelMessageBroker {
    async fn send_write_event(tx: tokio::sync::oneshot::Sender<ModelEvent>);
    async fn send_read_event(tx: tokio::sync::oneshot::Sender<ModelEvent>);
}

pub trait AgentMessageBroker {
    async fn send_get_column_event(
        tx: tokio::sync::mpsc::Sender<AgentEvent>,
        column: Arc<(Mutex<Vec<String>>, Condvar)>,
        kind: String,
    );
    async fn send_get_profiles_event(
        tx: tokio::sync::mpsc::Sender<AgentEvent>,
        profile: Arc<(Mutex<Vec<Profile>>, Condvar)>,
    );
    async fn send_store_profile_event(
        tx: tokio::sync::mpsc::Sender<AgentEvent>,
        profile: Arc<Profile>,
    );
    async fn send_quit_event(tx: tokio::sync::mpsc::Sender<AgentEvent>);
}

pub struct Controller;

impl ModelMessageBroker for Controller {
    async fn send_write_event(tx: tokio::sync::oneshot::Sender<ModelEvent>) {
        if tx.send(ModelEvent::WriteEvent).is_err() {
            panic!("Receiver dropped before message [DatabaseEvent::WriteEvent] could be sent",);
        }
    }

    async fn send_read_event(tx: tokio::sync::oneshot::Sender<ModelEvent>) {
        if tx.send(ModelEvent::ReadEvent).is_err() {
            panic!("Receiver dropped before message [DatabaseEvent::ReadEvent] could be sent",);
        }
    }
}

impl AgentMessageBroker for Controller {
    async fn send_get_column_event(
        tx: tokio::sync::mpsc::Sender<AgentEvent>,
        column: Arc<(Mutex<Vec<String>>, Condvar)>,
        kind: String,
    ) {
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
    }

    async fn send_get_profiles_event(
        tx: tokio::sync::mpsc::Sender<AgentEvent>,
        profile: Arc<(Mutex<Vec<Profile>>, Condvar)>,
    ) {
        if (tx
            .send(AgentEvent::ReadEvent(ReadEvent::GetProfiles(Arc::clone(
                &profile,
            ))))
            .await)
            .is_err()
        {
            panic!("Receiver dropped before message [AgentEvent::ReadEvent(ReadEvent::GetProfiles(arc))] could be sent");
        }
    }

    async fn send_store_profile_event(
        tx: tokio::sync::mpsc::Sender<AgentEvent>,
        profile: Arc<Profile>,
    ) {
        if (tx
            .send(AgentEvent::WriteEvent(WriteEvent::StoreProfile(Some(
                profile,
            ))))
            .await)
            .is_err()
        {
            panic!("Receiver dropped before message [AgentEvent::WriteEvent(WriteEvent::SaveProfile(arc))] could be sent");
        }
    }

    async fn send_quit_event(tx: tokio::sync::mpsc::Sender<AgentEvent>) {
        if (tx.send(AgentEvent::Quit).await).is_err() {
            println!("Receiver dropped before message [AgentEvent::Quit] could be sent");
        }
    }
}

#[derive(Debug)]
pub struct Profile {
    pub engine: String,
    pub source_root: String,
    pub source_file_extension: String,
    pub output_root: String,
    pub output_file_extension: String,
    pub options: Option<Vec<String>>,
    pub ignore_patterns: Option<Vec<String>>,
    pub metadata: Option<BTreeMap<String, i32>>,
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

static ASYNC_RUNTIME_HANDLE: LazyLock<Handle> = LazyLock::new(|| Handle::current());

pub fn get_runtime_handle() -> Handle {
    ASYNC_RUNTIME_HANDLE.clone()
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
