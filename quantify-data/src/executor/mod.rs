use core::future::Future;
use std::sync::Arc;

use mongodb::{self, options::{ClientOptions, ServerApi, ServerApiVersion}, Client};
use tokio::sync::Mutex;

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
    pub fn execute<T>(&self, task: &impl Task<T>)
        where T: Future + Send + 'static, T::Output: Send + 'static,
    {
        let task = task.init(&self.locked_mongo_client);
        let _handle = tokio::spawn(task);
        // TODO(mark): How do we handle errors - just log?
    }
}

pub trait Task<T> {
    fn init(&self, lmc: &Arc<Mutex<Client>>) -> T;
}