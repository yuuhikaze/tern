#![feature(async_closure)]

mod converter;
mod database;
mod interface;
mod utils;

use clap::Parser;
use converter::Converter;
use database::Database;
use interface::InterfaceBuilder;
use std::sync::Arc;
use tokio::{
    sync::{mpsc, oneshot, Mutex},
    task,
};
use utils::{ArgParser, ControlEvent, DatabaseArgs, InterfaceArgs, ReadEvent, WriteEvent};

#[tokio::main]
async fn main() {
    // data directory management
    utils::create_data_dir();
    // CLI options management
    let args = ArgParser::parse();
    // oneshot channel setup
    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    // database management
    let db_args = DatabaseArgs {
        profile_manager: args.profile_manager,
    };
    let db_arc_mutex = Arc::new(Mutex::new(Database::new(oneshot_tx, db_args)));
    let db = Arc::clone(&db_arc_mutex);
    db.lock().await.connect().await;
    db.lock().await.migrate().await;
    // mpsc channel setup
    let (mpsc_tx, mut mpsc_rx) = mpsc::channel(1);
    // database status receiver
    let agent_handle = async {
        if let Ok(database_event) = oneshot_rx.await {
            match database_event {
                utils::DatabaseEvent::ReadEvent => {
                    Converter::build();
                }
                utils::DatabaseEvent::WriteEvent => {
                    let interface_args = InterfaceArgs { tui: args.tui };
                    let mpsc_tx = mpsc_tx.clone();
                    task::spawn_blocking(|| {
                        InterfaceBuilder::build(mpsc_tx, interface_args).spawn_and_run();
                    })
                    .await
                    .unwrap();
                }
            }
        } else {
            panic!("Database status was not received");
        }
    };
    let controller_handle = async {
        while let Some(control_event) = mpsc_rx.recv().await {
            match control_event {
                ControlEvent::ReadEvent(read_event) => match read_event {
                    ReadEvent::GetColumn(arc, col) => {
                        db.lock().await.get_column(arc, &col).await;
                    }
                    ReadEvent::GetProfiles(arc) => {
                        db.lock().await.get_profiles(arc).await;
                    }
                },
                ControlEvent::WriteEvent(write_event) => match write_event {
                    WriteEvent::SaveProfile(arc) => {
                        println!("{:#?}", arc);
                        db.lock().await.save_profile(arc).await;
                    }
                },
                ControlEvent::Quit => break,
            };
        }
    };
    futures::future::join(agent_handle, controller_handle).await;
}
