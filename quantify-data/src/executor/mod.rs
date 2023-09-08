use core::future::Future;
use std::sync::Arc;

use mongodb::{self, options::{ClientOptions, FindOptions, ServerApi, ServerApiVersion}, bson::doc, Client, Database};
use tokio::{spawn, task::JoinHandle};
use futures::stream::TryStreamExt;

use tiingo::{self, TiingoRESTClient};
use polygon::{self, PolygonRESTClient};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

// MongoDB constants
const QUANTIFY_DATABASE: &str = "quantify";
const DAY_CANDLE_COLLECTION: &str = "day_candle";
const HOUR_CANDLE_COLLECTION: &str = "hour_candle";
const MINUTE_CANDLE_COLLECTION: &str = "minute_candle";

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

/**
 * Executor manages pool of tasks running on Tokio threads
 * Locking on a single mongodb::Client
 */
pub struct Executor {
    db_ref: Database
}

impl Executor {
    /**
     * Create a new Executor
     */
    pub async fn build(uri: &str) -> Result<Executor, mongodb::error::Error>{
        let mut client_options = ClientOptions::parse(uri).await?;
        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
            client_options.server_api = Some(server_api);
        let mongo_client = Client::with_options(client_options)?;
        let db_ref = mongo_client.database(QUANTIFY_DATABASE);

        Ok(Executor {db_ref})
    }

    /**
     * Runs a task
     */
    pub fn execute(&self, task: &Arc<impl TaskFactory>) -> JoinHandle<()>
    {
        let task: Task = TaskFactory::init(task.clone(), self.db_ref.clone());
        spawn(Box::into_pin(task))
    }
}


type Task = Box<dyn Future<Output=()> + Send +'static>;
pub trait TaskFactory
{
    fn init(this: Arc<Self>, db_ref: Database) -> Task;
}

// Tasks

pub struct AddTickerTask {
    ticker: String
}

impl AddTickerTask {
    pub fn new(ticker: &str) -> AddTickerTask{
        let t = String::from(ticker);
        AddTickerTask{ticker: t}
    }
}

impl TaskFactory for AddTickerTask {
    fn init (this: Arc<Self>, db_ref: Database) -> Task {
        Box::new(async move {
            // Initialize clients
            println!("Running");
            let client = reqwest::Client::new();
            let tiingo_client = TiingoRESTClient::new(client);
        })
    }
}

pub enum Granularity {
    Days(i32),
    Hours(i32),
    Minutes(i32)
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
    fn init (this: Arc<Self>, db_ref: Database) -> Task {
        Box::new(async move {
            // Initialize clients
            println!("Running");
            let client = reqwest::Client::new();
            let polygon_client = PolygonRESTClient::new(client);

            // Get collection handle and granularity
            let col_name: &str;
            let granularity: i32;
            match this.granularity {
                Granularity::Days(m) => {
                    col_name = DAY_CANDLE_COLLECTION;
                    granularity = m;
                },
                Granularity::Hours(m) => {
                    col_name = HOUR_CANDLE_COLLECTION;
                    granularity = m;
                },
                Granularity::Minutes(m) => {
                    col_name = MINUTE_CANDLE_COLLECTION;
                    granularity = m;
                }
            };
            let col_ref = db_ref.collection::<CandleData>(col_name);

            // Get latest entry
            let find_options = FindOptions::builder()
                .sort(doc! { "timestamp": -1 })
                .limit(1)
                .build();
            let cursor = col_ref.find(None, find_options).await?;
            
            if let Some(latest_data) = cursor.try_next().await? {
                // Latest data found

            } else {
                // Latest data not found
            }
        })
    }
}

// Tests
#[cfg(test)]
mod tests {
    use log::warn;
    use mongodb::Database;
    use tokio::task::JoinHandle;
    use std::sync::{Arc, Mutex};
    use super::{Executor, TaskFactory, Task};

    #[tokio::test]
    async fn test_executor() {
        // Create executor
        let exec = match Executor::build("localhost:27017").await {
            Ok(exec) => exec,
            Err(_) => {
                warn!("MongoDB is not available");
                return;
            }
        };
        // Create sample task
        struct ExampleTask {
            count: Mutex<i32>
        }

        impl TaskFactory for ExampleTask {
            fn init(this: Arc<Self>, db_ref: Database) -> Task {
                Box::new(async move {
                    let mut count = this.count.lock().unwrap();
                    *count += 1;
                })
            }
        }
        let mut handles: Vec<JoinHandle<()>> = Vec::new();
        let example = Arc::new(ExampleTask {count: Mutex::new(0)});
        for _ in 0..10 {
            handles.push(exec.execute(&example))
     
        }
        for handle in handles {
            let _ = handle.await;
        }
        let final_count = example.count.lock().unwrap();
        assert_eq!(*final_count, 10)
    }
}