#![allow(non_snake_case)]

use log::info;
use clap::Parser;
use server::grpc::QuantifyDataServerImpl;
use tokio::runtime::Runtime;
// Library
mod gui;
mod server;

// Arguments
#[derive(Parser)]
#[command(name = "Quantify")]
struct Cli {
    #[arg(long)]
    headless: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get Arguments and initalise logger
    let cli = Cli::parse();
    env_logger::init();

    // Construct tokio runtime
    let runtime = Runtime::new().unwrap();

    // Construct Executor Service
    let mongo_addr = std::env::var("QUANTIFY_DATABASE_URI").expect("You must set the QUANTIFY_DATABASE_URI environment var!");
    let server_addr = "[::1]:50051".parse()?;
    let server = QuantifyDataServerImpl::build(&mongo_addr);
    let server = runtime.block_on(server);

    // Start application
    if cli.headless {
        info!("Running in headless mode");
        runtime.block_on(server.start_service(server_addr))?;
    }
    else {
        info!("Running in GUI mode");
        let app = gui::QuantifyApp {server};
        app.run()?;
    }
    Ok(())   
}
