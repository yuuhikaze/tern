mod converter;
mod database;
mod interface;
mod config;

use clap::Parser;
use interface::InterfaceBuilder;
use database::Database;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct ArgParser {
    #[arg(long, action)]
    tui: bool,
}

#[tokio::main]
async fn main() {
    config::create_data_dir();
    let args = ArgParser::parse();
    let mut db = Database::new();
    let profiles = match db.init().await {
        database::Protocol::Creation => {
            db.connect().await;
            db.setup().await;
            let interface = InterfaceBuilder::new(args.tui);
            interface.spawn();
            // let config = interface.retrieve_data();
            // db.store(config);
            // config
            // db.load()
            vec![]
        },
        database::Protocol::Access => {
            db.connect().await;
            db.fetch_profiles().await
        }
    };
    for profile in profiles {
        println!("{:#?}", profile);
    }
    // let converter = Converter::new(config);
    // converter.run();
}
