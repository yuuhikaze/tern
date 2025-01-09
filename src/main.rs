#![feature(try_blocks)]

mod controller;
mod converter;
mod database;
mod interface;

use clap::Parser;
use controller::{
    AgentEvent, ArgParser, ConverterArgs, DatabaseArgs, InterfaceArgs, ReadEvent, WriteEvent,
};
use converter::ConverterFactory;
use database::Database;
use interface::InterfaceBuilder;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};

#[tokio::main]
async fn main() {
    // data directory management
    controller::create_data_dir();
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
                controller::ModelEvent::ReadEvent => {
                    let converter_args = ConverterArgs {
                        hidden: args.ignore_hidden_files,
                        force: args.force,
                        concurrent_profiles: args.concurrent_profiles,
                    };
                    controller::get_runtime_handle().spawn(async {
                        ConverterFactory::build(mpsc_tx, converter_args).run().await
                    });
                }
                controller::ModelEvent::WriteEvent => {
                    let interface_args = InterfaceArgs { tui: args.tui };
                    let mpsc_tx = mpsc_tx.clone();
                    controller::get_runtime_handle().spawn_blocking(|| {
                        InterfaceBuilder::build(mpsc_tx, interface_args).spawn_and_run();
                    });
                }
            }
        } else {
            panic!("Database status was not received");
        }
    };
    let controller_handle = async {
        while let Some(control_event) = mpsc_rx.recv().await {
            match control_event {
                AgentEvent::ReadEvent(read_event) => match read_event {
                    ReadEvent::GetColumn(arc, col) => {
                        db.lock().await.get_column(arc, &col).await;
                    }
                    ReadEvent::GetProfiles(arc) => {
                        db.lock().await.get_profiles(arc).await;
                    }
                },
                AgentEvent::WriteEvent(write_event) => match write_event {
                    WriteEvent::StoreProfile(arc) => {
                        db.lock().await.store_profile(arc).await;
                    }
                    WriteEvent::UpdateMetadata(met) => {
                        db.lock().await.update_metadata(met.0, met.1).await;
                    }
                },
                AgentEvent::Quit => break,
            };
        }
    };
    futures::future::join(agent_handle, controller_handle).await;
}
