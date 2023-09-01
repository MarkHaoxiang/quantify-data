use core::future::Future;
use std::{sync::Arc, ops::Deref};

use mongodb::{self, bson::doc, options::ClientOptions, Client, bson::Document, Collection};
use tokio::{sync::Mutex, spawn, task::JoinHandle};

use tiingo::{self, TiingoRESTClient, meta::Metadata};

/**
 * Executor manages pool of tasks running on Tokio threads
 * Locking on a single mongodb::Client
 */

pub struct Executor {
    locked_mongo_client: Arc<Mutex<Client>>,
}

impl Executor {
    /**
     * Create a new Executor
     */
    pub async fn build(uri: &str) -> Result<Executor, mongodb::error::Error> {
        let mut client_options = ClientOptions::parse(uri).await?;
        client_options.app_name = Some("Quantify".to_string());
        let mongo_client = Client::with_options(client_options)?;
        let locked_mongo_client = Arc::new(Mutex::new(mongo_client));
        Ok(Executor {locked_mongo_client})
    }

    /**
     * Runs a task
     */
    pub fn execute(&self, task: &Arc<impl TaskFactory>) -> JoinHandle<()>
    {
        let task: Task = TaskFactory::init(task.clone(), self.locked_mongo_client.clone());
        spawn(Box::into_pin(task))
    }
}


type Task = Box<dyn Future<Output=()> + Send +'static>;
pub trait TaskFactory
{
    fn init(this: Arc<Self>, lmc: Arc<Mutex<Client>>) -> Task;
}

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
    fn init (this: Arc<Self>, lmc: Arc<Mutex<Client>>) -> Task {
        Box::new(async move {
            // Initialize clients
            let client = reqwest::Client::new();
            let tiingo_client = TiingoRESTClient::new(client);

            // Get data
            let ticker_data: Metadata = tiingo_client.get_metadata(&this.ticker).await;
                // TODO: Polygon data with check

            // Update meta table
            {
                // Get the mongo client lock
                let mg = lmc.lock().await;
                let mc: &Client = mg.deref();
                let db = mc.database("quantify");
                let collection: Collection<Document> = db.collection::<Document>("tickers");
                // Update the document
                    // TODO: Remaining fields of document
                let ticker_document = doc! {
                    "ticker": ticker_data.name.to_lowercase(),
                    "exchange": ticker_data.exchange_code.to_lowercase()
                };
                    // TODO: Error handling
                collection.delete_many(
                    doc! {
                        "ticker": ticker_data.name.to_lowercase(),
                    } ,
                    None
                ).await.unwrap();
                collection.insert_one(ticker_document, None).await.unwrap();
            }      
        })
    }
}

// Tests
#[cfg(test)]
mod tests {
    use tokio::task::JoinHandle;
    use std::{env, sync::{Arc, Mutex}};
    use std::io::{self,Write};
    use super::{Executor, TaskFactory, Task};

    #[tokio::test]
    async fn test_executor() {
        // Create executor
        let client_uri =
            env::var("QUANTIFY_DATABASE_URI").expect("You must set the QUANTIFY_DATABASE_URI environment var!");
        let exec = match Executor::build(&client_uri).await {
            Ok(exec) => exec,
            Err(_) => {
                write!(&mut io::stdout(), "Skipping test: MongoDB cannot be accessed \n").unwrap();
                return;
            }
        };
        // Create sample task
        struct ExampleTask {
            count: Mutex<i32>
        }

        impl TaskFactory for ExampleTask {
            fn init(this: Arc<Self>, _lmc: Arc<tokio::sync::Mutex<mongodb::Client>>) -> Task {
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