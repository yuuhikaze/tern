use tern_core::interface::CommandLineInterface;
use tern_core::run_app;

#[tokio::main]
async fn main() {
    run_app(|_, _| Box::new(CommandLineInterface)).await;
}
