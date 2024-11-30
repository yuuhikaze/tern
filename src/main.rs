#![feature(async_closure)]

mod controller;
mod converter;
mod database;
mod interface;

use clap::Parser;
use controller::{ArgParser, DatabaseEvent, InterfaceEvent};
use database::Database;
use interface::InterfaceBuilder;
use std::sync::Arc;
use tokio::{
    sync::{mpsc, Mutex},
    task,
};

#[tokio::main]
async fn main() {
    controller::create_data_dir();
    let args = ArgParser::parse();
    let db_arc_mutex = Arc::new(Mutex::new(Database::new()));
    let db = Arc::clone(&db_arc_mutex);
    let mut database_event = db.lock().await.init().await;
    if args.profile_manager {
        database_event = DatabaseEvent::Write;
    }
    match database_event {
        DatabaseEvent::Write => {
            let (tx, mut rx) = mpsc::channel(1);
            let db_handle = async {
                db.lock().await.connect().await;
                db.lock().await.setup().await;
            };
            let ui_handle = async {
                task::spawn_blocking(|| {
                    let mut interface = InterfaceBuilder::build(args, tx);
                    interface.spawn_and_run();
                })
                .await
                .unwrap();
            };
            let interface_event_reader_handle = async {
                while let Some(it) = rx.recv().await {
                    match it {
                        InterfaceEvent::Save(profile) => {
                            db.lock().await.save_profile(profile).await
                        }
                        InterfaceEvent::Quit => break,
                    };
                }
            };
            futures::future::join3(db_handle, ui_handle, interface_event_reader_handle).await;
        }
        DatabaseEvent::Read => {
            db.lock().await.connect().await;
            let profiles = db.lock().await.fetch_profiles().await;
            println!("{:#?}", profiles);
        }
    };
    // let converter = Converter::new(config);
    // converter.run();
}
