#![allow(non_snake_case)]

use tonic::{Request, transport::Channel};

// gRPC
use quantify::{
    GranularityType,
    Ticker,
    UpdateCandleDataRequest};
use quantify::quantify_data_client::QuantifyDataClient;

pub mod quantify {
    tonic::include_proto!("quantify");
}

pub struct QuantifyDataPoller {
    client: QuantifyDataClient<Channel>,
}

impl QuantifyDataPoller {
    pub async fn connect(addr: &str) -> Result<QuantifyDataPoller, tonic::transport::Error> {
        let client = QuantifyDataClient::connect(String::from(addr)).await?;
        return Ok(QuantifyDataPoller{client});
    }

    pub async fn update_candle_data(&mut self, ticker: &str) -> Result<(), tonic::Status> {
        let request = Request::new(UpdateCandleDataRequest{
            ticker: Some(Ticker{name: ticker.to_string()}),
            granularity_type: GranularityType::Days as i32,
            granularity_value: 1,
        });
        let response = self.client.update_candle_data(request).await?;
        println!("{:?}", response.into_inner());
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Run gRPC server
    let addr = "http://[::1]:50051";
    let mut fetcher = QuantifyDataPoller::connect(addr).await?;

    let ticker = "test";
    fetcher.update_candle_data(ticker).await?;

    Ok(())   
}