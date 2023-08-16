#![allow(non_snake_case)]
use tonic::{transport::Server, Request, Response, Status};

use quantify::{AddTickerRequest, RemoveTickerRequest, StatusResponse};
use quantify::quantify_data_server::{QuantifyData, QuantifyDataServer};

pub mod quantify {
    tonic::include_proto!("quantify");
}

#[derive(Debug, Default)]
pub struct QuantifyDataImpl {}

#[tonic::async_trait]
impl QuantifyData for QuantifyDataImpl {
    async fn add_ticker(
        &self,
        request: Request<AddTickerRequest>
    ) -> Result<Response<StatusResponse>, Status> {
        println!("Adding ticker {:?}", request);

        let reply = StatusResponse {
            success: true,
            info: Some(String::from("Subscribed to ticker"))
        };

        Ok(Response::new(reply))
    }

    async fn remove_ticker(
        &self,
        request: Request<RemoveTickerRequest>
    ) -> Result<Response<StatusResponse>, Status> {
        println!("Removing ticker {:?}", request);

        let reply = StatusResponse {
            success: true,
            info: Some(String::from("Removed ticker"))
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let server = QuantifyDataImpl::default();

    Server::builder()
        .add_service(QuantifyDataServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}