use crate::controller::{AgentEvent, InterfaceArgs};
use tokio::sync::mpsc::Sender;

pub trait Interface: Send {
    fn spawn_and_run(&mut self);
}

pub struct CommandLineInterface;

impl Interface for CommandLineInterface {
    fn spawn_and_run(&mut self) {
        println!("Tern Core: Batch conversion complete (TUI not yet implemented).");
    }
}
