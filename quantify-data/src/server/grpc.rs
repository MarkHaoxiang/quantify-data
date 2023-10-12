use tonic::{Request, Response, Status};
use std::sync::Arc;

use quantify::{
    Ticker,
    CandleData,
    AddTickerRequest,
    RemoveTickerRequest,
    UpdateCandleDataRequest,
    GetCandleDataRequest,
    StatusResponse,
    GetCandleDataResponse
};

pub mod quantify {
    tonic::include_proto!("quantify");
}

use quantify::quantify_data_server::QuantifyData;
use quantifylib::executor;

#[tonic::async_trait]
impl QuantifyData for crate::QuantifyDataImpl {

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

        let task = Arc::new(executor::tasks::AddTickerTask::new(ticker));
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
