#![feature(async_closure)]
#![allow(incomplete_features)]
#![feature(unsized_fn_params)]

mod controller;
mod converter;
mod database;
mod interface;

use clap::Parser;
use controller::{ArgParser, DatabaseEvent, InterfaceEvent};
use database::Database;
use interface::InterfaceBuilder;
use tokio::{sync::mpsc, task};

#[tokio::main]
async fn main() {
    controller::create_data_dir();
    let args = ArgParser::parse();
    let mut db = Database::new();
    let mut database_event = db.init().await;
    if args.profile_manager {
        database_event = DatabaseEvent::Write;
    }
    match database_event {
        DatabaseEvent::Write => {
            let (tx, mut rx) = mpsc::channel(1);
            let db_handle = async {
                db.connect().await;
                db.setup().await;
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
                        InterfaceEvent::Save(profile) => println!("{:#?}", profile),
                        InterfaceEvent::Quit => break,
                    };
                }
            };
            futures::future::join3(db_handle, ui_handle, interface_event_reader_handle).await;
        }
        DatabaseEvent::Read => {
            db.connect().await;
            let profiles = db.fetch_profiles().await;
        }
    };
    // let converter = Converter::new(config);
    // converter.run();
}
