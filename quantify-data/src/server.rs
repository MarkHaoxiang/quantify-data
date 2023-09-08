#![allow(non_snake_case)]

use std::sync::Arc;

use executor::Executor;
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
pub struct QuantifyDataImpl {
    pub executor: executor::Executor
}

impl QuantifyDataImpl {
    pub async fn build(uri: &str) -> QuantifyDataImpl {
        let executor = Executor::build(uri).await.unwrap();
        QuantifyDataImpl { executor }
    }
}

#[tonic::async_trait]
impl QuantifyData for QuantifyDataImpl {

    async fn add_ticker(
        &self,
        request: Request<AddTickerRequest>
    ) -> Result<Response<StatusResponse>, Status> {
        println!("Adding ticker {:?}", request);

        let ticker = match &request.get_ref().ticker {
            Some(t) => &t.name,
            None => 
                return Ok(Response::new(StatusResponse {
                    success: false,
                    info: Some(String::from("Ticker not provided"))
                })),
        };

        let task = Arc::new(executor::AddTickerTask::new(ticker));

        match __self.executor.execute(&task).await {
            Ok(_) => {},
            Err(_) => 
                return Ok(Response::new(StatusResponse {
                    success: false,
                    info: Some(String::from("Ticker subscription failed"))
                })),
        };

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
    let mongo_addr = std::env::var("QUANTIFY_DATABASE_URI").expect("You must set the QUANTIFY_DATABASE_URI environment var!");
    let server_addr = "[::1]:50051".parse()?;
    let server = QuantifyDataImpl::build(&mongo_addr).await;

    Server::builder()
        .add_service(QuantifyDataServer::new(server))
        .serve(server_addr)
        .await?;

    Ok(())   
}

// Polling / automatic behavior
// TODO