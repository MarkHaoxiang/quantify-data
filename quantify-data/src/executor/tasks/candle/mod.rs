use std::sync::Arc;

use chrono::{NaiveDateTime, Utc};
use futures::TryStreamExt;
use mongodb::{Database, options::FindOptions, bson::doc};
use polygon::{PolygonRESTClient, Interval};
use serde::{Serialize, Deserialize};

use crate::executor::{Executor, Task, TaskFactory};

// MongoDB constants
const DAY_CANDLE_COLLECTION: &str = "day_candle";
const HOUR_CANDLE_COLLECTION: &str = "hour_candle";
const MINUTE_CANDLE_COLLECTION: &str = "minute_candle";

pub enum Granularity {
    Days(i32),
    Hours(i32),
    Minutes(i32)
}

// Definitions
#[derive(Debug, Serialize, Deserialize)]
struct CandleData {
    ticker: String,
    timestamp: NaiveDateTime,
    open: f64,
    close: f64,
    high: f64,
    low: f64,
    volume: i64,
    num_transactions: i64
}

pub struct UpdateCandleDataTask {
    ticker: String,
    granularity: Granularity
}

impl UpdateCandleDataTask {
    pub fn new(ticker: &str, granularity: Granularity) -> UpdateCandleDataTask{
        let t = String::from(ticker);
        UpdateCandleDataTask{ticker: t, granularity}
    }
}

impl TaskFactory for UpdateCandleDataTask {
    fn init (this: Arc<Self>, _executor: Arc<Executor>, db_ref: Database, client: reqwest::Client) -> Task {
        Box::new(async move {
            // Initialize clients
            println!("Running");
            let client = reqwest::Client::new();
            let polygon_client = PolygonRESTClient::new(client);

            // Get collection handle and granularity
            let col_name: &str;
            let granularity: &Interval;
            match this.granularity {
                Granularity::Days(m) => {
                    col_name = DAY_CANDLE_COLLECTION;
                    granularity = &Interval::Days(m);
                },
                Granularity::Hours(m) => {
                    col_name = HOUR_CANDLE_COLLECTION;
                    granularity = &Interval::Hours(m);
                },
                Granularity::Minutes(m) => {
                    col_name = MINUTE_CANDLE_COLLECTION;
                    granularity = &Interval::Minutes(m);
                }
            };
            let col_ref = db_ref.collection::<CandleData>(col_name);

            // Get latest entry
            let find_options = FindOptions::builder()
                .sort(doc! { "timestamp": -1 })
                .limit(1)
                .build();
            let data_result = col_ref
                .find(None, find_options)
                .await?
                .try_next()
                .await?;
            
            if data_result.unwrap().is_none() {
                // None retrieved
                println!("latest candle data not found");
                return Err("latest candle data not found");
            }
            
            // Latest data found
            let Some(latest_data) = data_result.unwrap();

            // Get timestamp of latest data and use it as the start_date of retrieval
            let start_date = &latest_data.timestamp.date();

            // Retrieve from PolygonIO
            let end_date = &Utc::now().naive_utc().date(); // NOTE: Using UTC as standard
            let adjusted = &false;
            let data = polygon_client.get_aggs(
                this.ticker.as_str(),
                start_date,
                end_date,
                granularity,
                adjusted).await?;

            // TODO: Add data to MongoDB

            Ok(())
        })
    }
}