#![allow(non_snake_case)]

use tonic::{transport::Server, Request, Response, Status};

// gRPC
use quantify::{
    Ticker,
    CandleData,
    AddTickerRequest,
    RemoveTickerRequest,
    UpdateCandleDataRequest,
    GetCandleDataRequest,
    StatusResponse,
    GetCandleDataResponse};
use quantify::quantify_data_server::{QuantifyData, QuantifyDataServer};

// Library
mod executor;

pub mod quantify {
    tonic::include_proto!("quantify");
}

// gRPC Entry Points
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

    async fn update_candle_data(
        &self,
        request: Request<UpdateCandleDataRequest>
    ) -> Result<Response<StatusResponse>, Status> {
        println!("Adding candle data {:?}", request);

        let reply = StatusResponse {
            success: true,
            info: Some(String::from("Added candle data"))
        };

        Ok(Response::new(reply))
    }

    async fn get_candle_data(
        &self,
        request: Request<GetCandleDataRequest>
    ) -> Result<Response<GetCandleDataResponse>, Status> {
        println!("Retrieving candle data {:?}", request);

        let reply = GetCandleDataResponse {
            candle_data: vec![CandleData{
                ticker: Some(Ticker{name: "test".to_string()}),
                timestamp: 100,
                open: 5.0,
                close: 5.0,
                high: 5.0,
                low: 5.0,
                volume: 10,
                num_transactions: 20,
            }]
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Run gRPC server
    let addr = "[::1]:50051".parse()?;
    let server = QuantifyDataImpl::default();

    Server::builder()
        .add_service(QuantifyDataServer::new(server))
        .serve(addr)
        .await?;

    Ok(())   
}

// Polling / automatic behavior
// TODO