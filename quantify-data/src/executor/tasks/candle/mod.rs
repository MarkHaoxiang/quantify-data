use std::sync::Arc;
use std::collections::HashMap;

use chrono::{DateTime, Utc, offset::Local, Duration};
use futures::TryStreamExt;
use mongodb::{Database, options::FindOptions, bson::doc};
use polygon::DEFAULT_POLYGON_MAX_HISTORICAL_WEEKS;
use serde::{Serialize, Deserialize};

use crate::executor::{Executor, Task, TaskFactory, tasks::api_managers::get_managers_in_priority};

// Rexporting for executor::tasks::data_managers.rs
pub mod aggregate_data_interface;

// MongoDB constants
const DAY_CANDLE_COLLECTION: &str = "day_candle";
const HOUR_CANDLE_COLLECTION: &str = "hour_candle";
const MINUTE_CANDLE_COLLECTION: &str = "minute_candle";

#[allow(dead_code)]
pub enum Granularity {
    Days(i32),
    Hours(i32),
    Minutes(i32)
}

// Definitions
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CandleData {
    ticker: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    timestamp: DateTime<Utc>, // MongoDB requires a "valid BSON UTC datetime value"
    open: f64,
    close: f64,
    high: f64,
    low: f64,
    volume: i64,
    num_transactions: i64
}

/// Task that checks the local database for the latest data entry for a given ticker and updates itwith the data updated since.
pub struct UpdateCandleDataTask {
    ticker: String,
    granularity: Granularity,
    max_historical_duration: Duration // Maximum time in the past from which data can be pulled
}

impl UpdateCandleDataTask {
    /// Constructs a new Update Candle Data task.
    /// 
    /// # Arguments
    /// 
    /// * 'ticker' - Ticker of target asset.
    /// * 'granularity' - The granularity of the data to be retrieved. Defined by the enum Granularity, which defines both the multiplier (eg. 5) and the interval (eg. minutes)
    /// * 'max_historical_duration' - The fallback historical duration (i.e. retrieve data from [now - duration] to now) if no data is found locally.
    #[allow(dead_code)]
    pub fn new(ticker: &str, granularity: Granularity, max_historical_duration: Option<Duration>) -> UpdateCandleDataTask{
        let t = String::from(ticker);

        // Defaults to the maximum allowable duration by Polygon (defined as DEFAULT_POLYGON_MAX_HISTORICAL_WEEKS)
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
            let default_start_date = Local::now().date_naive().checked_sub_signed(this.max_historical_duration);
            if default_start_date.is_none() {
                return Err("default start date is out of range".into());
            }

            // Check the current data (and the start date)
            let start_date = &match data_result {
                Some(data) => data.timestamp.date_naive(),
                None => default_start_date.unwrap()
            };

            let end_date = &Local::now().date_naive();

            let managers = get_managers_in_priority(client.clone());
            let mut retrieved_data: HashMap<i64, CandleData> = HashMap::new(); // HashMap<timestamp as i64, CandleData>
            for manager  in managers {
                // Get data from manager
                let res = manager.get_agg_candle_data(
                    this.ticker.as_str(),
                    start_date,
                    end_date,
                    &this.granularity,
                ).await?;

                for entry in res {
                    let timestamp = entry.timestamp.timestamp();
                    if data_result.is_some() {
                        if timestamp <= data_result.as_ref().unwrap().timestamp.timestamp() {
                            continue;
                        }
                    }

                    // If the timestamp does not yet exist in the data HashMap (e.g. due to missing data from a higher-priority source), insert it
                    if !retrieved_data.contains_key(&timestamp) {
                        retrieved_data.insert(timestamp, entry);
                    }
                }
                break;
            }

            let candle_data = retrieved_data.values().cloned().collect::<Vec<CandleData>>();

            if candle_data.len() <= 0 {
                // Everything is up-to-date or there is no data to be retrieved from the server
                return Ok(());
            }

            // Add retrieved data to MongoDB Collection
            col_ref.insert_many(candle_data, None).await?;

            Ok(())
        })
    }
}

// Tests
#[cfg(test)]
mod tests {
    // Ensure that the MongoDB database is running and the following environment variables are set:
    // - POLYGON_API_KEY
    // - QUANTIFY_DATABASE_URI

    use super::{UpdateCandleDataTask, Granularity};
    use crate::executor::Executor;
    use chrono::Duration;
    use std::{env, sync::Arc};
    use std::io::{self,Write};

    // Creates an executor based on environment variables
    async fn create_executor() -> Option<Arc<Executor>> {
        let client_uri =
            env::var("QUANTIFY_DATABASE_URI").expect("You must set the QUANTIFY_DATABASE_URI environment var!");
        let exec = Arc::new(match Executor::build(&client_uri).await {
            Ok(exec) => exec,
            Err(_) => {
                write!(&mut io::stdout(), "Skipping test: MongoDB cannot be accessed \n").unwrap();
                return None;
            }
        });
        return Some(exec);
    }
    
    #[tokio::test]
    async fn UpdateCandleDataTaskTest() {
        let exec = match create_executor().await {
            Some(exec) => exec,
            None => return,
        };

        let ticker = "GOOGL";
        let granularity = Granularity::Hours(1);
        let max_historical_duration = Some(Duration::weeks(4));
        let updateTask = Arc::new(UpdateCandleDataTask::new(
            ticker,
            granularity,
            max_historical_duration));
        
        let ok = exec.execute(&updateTask).await;
        match ok {
            Ok(res) => match res {
                Ok(_) => assert!(true),
                Err(err) => panic!("{}", err)
            },
            Err(err) => panic!("{}", err)
        }

        // Lastly, check the database to see if it is populated
    }
}