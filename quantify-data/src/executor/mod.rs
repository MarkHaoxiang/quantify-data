use core::future::Future;
use std::sync::Arc;

use mongodb::{self, options::{ClientOptions, ServerApi, ServerApiVersion}, Client};
use tokio::{sync::Mutex, spawn, task::JoinHandle};

use tiingo::{self, TiingoRESTClient};

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
    pub async fn build(uri: &str) -> Result<Executor, mongodb::error::Error>{
        let mut client_options = ClientOptions::parse(uri).await?;
        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
            client_options.server_api = Some(server_api);
        let mongo_client = Client::with_options(client_options)?;
        let locked_mongo_client = Arc::new(Mutex::new(mongo_client));

        Ok(Executor {locked_mongo_client})
    }

    /**
     * Runs a task
     */
    pub fn execute(&self, task: &Arc<impl TaskFactory>) -> JoinHandle<()>
    {
        let task: Task = TaskFactory::init(task.clone(), &self.locked_mongo_client);
        spawn(Box::into_pin(task))
    }
}


type Task = Box<dyn Future<Output=()> + Send +'static>;
pub trait TaskFactory
{
    fn init(this: Arc<Self>, lmc: &Arc<Mutex<Client>>) -> Task;
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
    fn init (this: Arc<Self>, lmc: &Arc<Mutex<Client>>) -> Task {
        Box::new(async move {
            // Initialize clients
            println!("Running");
            let client = reqwest::Client::new();
            let tiingo_client = TiingoRESTClient::new(client);
        })
    }
}

// Tests
#[cfg(test)]
mod tests {
    use log::warn;
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
            fn init(this: Arc<Self>, lmc: &Arc<tokio::sync::Mutex<mongodb::Client>>) -> Task {
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