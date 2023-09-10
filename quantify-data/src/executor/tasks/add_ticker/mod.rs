use std::{sync::Arc, process::CommandArgs};

use mongodb::{Database, Collection, bson::{Document, doc}};
use reqwest::Client;

use crate::executor::{TaskFactory, Executor, Task};

use super::resolver::mode;

/// Registers a ticker into the database
pub struct AddTickerTask {
    ticker: String
}
impl AddTickerTask {
    /// Constructs a new instance of AddTickerTask
    /// 
    /// # Arguments
    /// 
    /// * 'ticker' - The financial ticker
    pub fn new(ticker: &str) -> AddTickerTask{
        let t = String::from(ticker);
        AddTickerTask{ticker: t}
    }
}

impl TaskFactory for AddTickerTask {
    /// [AddTickerTask]
    fn init (this: Arc<Self>, _executor: Arc<Executor>, db_ref: Database, client: Client) -> Task {
        Box::new(async move {
            let ticker: &String = &this.ticker.to_lowercase();
            // Initialize clients
            let tiingo_client = tiingo::TiingoRESTClient::new(client.clone());
            let polygon_client = polygon::PolygonRESTClient::new(client.clone());
            
            // Get data
            let ticker_data_tiingo = tiingo_client.get_metadata(&this.ticker).await;
            let ticker_data_polygon = polygon_client.get_meta(&this.ticker, None).await;

            let mut company: Vec<String> = Vec::new();
            let mut exchange: Vec<String> = Vec::new();

            match ticker_data_tiingo {
                Ok(tiingo_metadata) => {
                    company.push(tiingo_metadata.name);
                    exchange.push(tiingo_metadata.exchange_code)
                },
                Err(_) => ()
            }
            match ticker_data_polygon {
                Ok(polygon_metadata) => {
                    company.push(polygon_metadata.name);
                    exchange.push(polygon_metadata.primary_exchange)
                },
                Err(_) => ()
            }

            if company.len() == 0 || exchange.len() == 0 {
                return Err(format!("No data found for {ticker}"))?;
            }
            let company = mode::<String>(&company).unwrap().to_lowercase();
            let exchange = mode::<String>(&exchange).unwrap().to_lowercase();

            // Update meta table
            let collection: Collection<Document> = db_ref.collection::<Document>("tickers");
            let ticker_document = doc! {
                "ticker": ticker,
                "company": company,
                "exchange": exchange
            };
                // TODO: Error handling
            collection.delete_many(
                doc! {
                    "ticker": ticker
                } ,
                None
            ).await.unwrap();
            collection.insert_one(ticker_document, None).await.unwrap();
            Ok(())
        })
    }
}