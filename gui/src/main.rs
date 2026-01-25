use tern_core::run_app;
use tern_core::interface::{Interface, CommandLineInterface};
use tern_core::controller::{AgentEvent, InterfaceArgs};
use tokio::sync::mpsc::Sender;

mod interface;
use interface::GraphicalInterface;

fn build_interface(tx: Sender<AgentEvent>, args: InterfaceArgs) -> Box<dyn Interface> {
    if args.tui {
        Box::new(CommandLineInterface)
    } else {
        Box::new(GraphicalInterface {
            tx: Some(tx),
            app: None,
        })
    }
}

#[tokio::main]
async fn main() {
    run_app(build_interface).await;
}
