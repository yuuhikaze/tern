mod converter;
mod database;
mod interface;

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
    let args = ArgParser::parse();
    let db = Database::new();
    /* let config = */ match db.init().await {
        database::Protocol::Creation => {
            let interface = InterfaceBuilder::new(args.tui);
            interface.spawn();
            // interface.store_data();
            // db.load()
        },
        database::Protocol::Access => {
            // db.load()
        }
    };
    // let converter = Converter::new(config);
    // converter.run();
}
