use std::sync::Arc;

use chrono::{NaiveDateTime, Utc, Duration};
use futures::TryStreamExt;
use mongodb::{Database, options::FindOptions, bson::doc};
use polygon::{PolygonRESTClient, Interval, DEFAULT_POLYGON_MAX_HISTORICAL_WEEKS};
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
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    granularity: Granularity,
    max_historical_duration: Duration // Maximum time in the past from which data can be pulled
}

impl UpdateCandleDataTask {
    pub fn new(ticker: &str, granularity: Granularity, max_historical_duration: Option<Duration>) -> UpdateCandleDataTask{
        let t = String::from(ticker);
        let duration = match max_historical_duration {
            Some(duration) => duration,
            None => Duration::weeks(DEFAULT_POLYGON_MAX_HISTORICAL_WEEKS)
        };
        UpdateCandleDataTask{ticker: t, granularity, max_historical_duration: duration}
    }
}

impl TaskFactory for UpdateCandleDataTask {
    fn init (this: Arc<Self>, _executor: Arc<Executor>, db_ref: Database, client: reqwest::Client) -> Task {
        Box::new(async move {
            // Initialize clients
            println!("Running");
            let polygon_client = PolygonRESTClient::new(client);

            // Get collection handle and granularity
            let col_name = match this.granularity {
                Granularity::Days(_) => DAY_CANDLE_COLLECTION,
                Granularity::Hours(_) => HOUR_CANDLE_COLLECTION,
                Granularity::Minutes(_) => MINUTE_CANDLE_COLLECTION
            };
            let col_ref = db_ref.collection::<CandleData>(col_name);

            // Get latest entry
            let find_options = FindOptions::builder()
                .sort(doc! { "timestamp": -1 })
                .limit(1)
                .build();
            let data_result = &col_ref
                .find(None, find_options)
                .await?
                .try_next()
                .await?;
            
            // Calculate the default start date (if no data is found, pull all available data)
            let default_start_date = Utc::now().naive_utc().date().checked_sub_signed(this.max_historical_duration);
            if default_start_date.is_none() {
                return Err("default start date is out of range".into());
            }

            // Check the current data (and the start date)
            let start_date = &match data_result {
                Some(data) => data.timestamp.date(),
                None => default_start_date.unwrap() // NOTE: Using UTC as standard
            };

            // Retrieve from PolygonIO
            let end_date = &Utc::now().naive_utc().date(); // NOTE: Using UTC as standard
            let interval = &match this.granularity {
                Granularity::Days(m) => Interval::Days(m),
                Granularity::Hours(m) => Interval::Hours(m),
                Granularity::Minutes(m) => Interval::Minutes(m)
            };
            let adjusted = &false;
            let agg_data = polygon_client.get_aggs(
                this.ticker.as_str(),
                start_date,
                end_date,
                interval,
                adjusted).await?;

            // Convert PolygonIO Aggregate Data into Candle Data
            let mut candle_data: Vec<CandleData> = Vec::new();
            for agg in &agg_data {
                if data_result.is_some() {
                    if agg.datetime.timestamp() <= data_result.as_ref().unwrap().timestamp.timestamp() {
                        continue;
                    }
                }
                candle_data.push(CandleData {
                    ticker: this.ticker.clone(),
                    timestamp: agg.datetime.naive_utc(),
                    open: agg.open,
                    close: agg.close,
                    high: agg.high,
                    low: agg.low,
                    volume: agg.volume as i64,
                    num_transactions: agg.num_transactions as i64
                })
            }

            // Add retrieved data to MongoDB Collection
            col_ref.insert_many(candle_data, None).await?;

            Ok(())
        })
    }
}
