#![allow(non_snake_case)]

use std::sync::Arc;
use log::info;
use clap::Parser;
use tonic::transport::Server;
use quantifylib::executor::Executor;

use quantify::quantify_data_server::QuantifyDataServer;
// Library
mod server;

pub mod quantify {
    tonic::include_proto!("quantify");
}

// Binary Representation
pub struct QuantifyDataImpl {
    pub executor: Arc<Executor>
}

impl QuantifyDataImpl {
    pub async fn build(uri: &str) -> QuantifyDataImpl {
        let executor = Executor::build(uri).await.unwrap();
        QuantifyDataImpl { executor: Arc::new(executor) }
    }
}

#[derive(Parser)]
#[command(name = "Quantify")]
struct Cli {
    #[arg(long)]
    headless: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get Arguments
    let cli = Cli::parse();
    env_logger::init();
    if cli.headless {
        info!("Running in headless mode");
    }
    else {
        info!("Running in GUI mode");
    }

    // Construct Executor Service
    let mongo_addr = std::env::var("QUANTIFY_DATABASE_URI").expect("You must set the QUANTIFY_DATABASE_URI environment var!");
    let server_addr = "[::1]:50051".parse()?;
    let server = QuantifyDataImpl::build(&mongo_addr).await;

    Server::builder()
        .add_service(QuantifyDataServer::new(server))
        .serve(server_addr)
        .await?;

    Ok(())   
}
