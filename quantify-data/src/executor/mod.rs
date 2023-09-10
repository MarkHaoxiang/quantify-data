#![warn(missing_docs)]

use core::future::Future;
use std::{sync::Arc, error::Error};

use mongodb::{self, options::ClientOptions, Database};
use tokio::{spawn, task::JoinHandle};

const QUANTIFY_DATABASE: &str = "quantify";

pub mod tasks;

/// Asynchronously manages execution of tasks
/// 
/// Handles on the quantify database and scheduler(TODO).
pub struct Executor {
    db_ref: Database,
    client: reqwest::Client
}
impl Executor {
    /// Constructs a new executor
    ///
    /// # Arguments
    /// 
    /// * 'uri' - A string slice that represents the mongo database connection
    ///
    pub async fn build(uri: &str) -> Result<Executor, mongodb::error::Error>
    {
        let mut client_options = ClientOptions::parse(uri).await?;
        client_options.app_name = Some("Quantify".to_string());
        let mongo_client = mongodb::Client::with_options(client_options)?; 
        let db_ref = mongo_client.database(QUANTIFY_DATABASE);
        let client = reqwest::Client::new();

        Ok(Executor {db_ref, client})
    }

    /// Runs a task
    /// 
    /// Calls tokio spawn internally
    /// 
    /// # Arguments
    /// 
    /// * 'self' - a reference counted Executor, to ensure lifespan is above all tasks
    /// * 'task' - the task to execute, in the form of a task factory
    pub fn execute(self: &Arc<Self>, task: &Arc<impl TaskFactory>) -> JoinHandle<Result<(), Box<dyn Error + Send + Sync>>>
    {
        let task: Task = TaskFactory::init(
            task.clone(),
            self.clone(), 
            self.db_ref.clone(),
            self.client.clone()
        );
        spawn(Box::into_pin(task))
    }
}

/// An spawnable function
type Task = Box<dyn Future<Output=Result<(),Box<dyn Error + Send + Sync>>> + Send +'static>;
/// An executable task
pub trait TaskFactory
{
    /// Initializes the task based on the context
    /// 
    /// # Arguments
    /// 
    /// * 'this'
    /// * 'executor' - For use in recursive calls
    /// * 'db_ref' - Mongo database handle for quantify
    /// * 'client' - reqwest client
    /// 
    /// # Examples
    /// 
    /// ```
    /// struct ExampleTask {
    ///     count: Mutex<i32>
    /// }
    /// impl TaskFactory for ExampleTask {
    ///     fn init(this: Arc<Self>, _executor: Arc<Executor>, _db_ref: Database) -> Task {
    ///         Box::new(async move {
    ///             let mut count = this.count.lock().unwrap();
    ///             *count += 1;
    ///         })
    ///     }
    /// }
    /// ```
    fn init(this: Arc<Self>, executor: Arc<Executor>, db_ref: Database, client: reqwest::Client) -> Task;
}

// Tests
#[cfg(test)]
mod tests {
    use mongodb::Database;
    use std::{env, sync::{Arc, Mutex}};
    use std::io::{self,Write};
    use super::{Executor, TaskFactory, Task};

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

    // Create simple task (Test basic execution)// Test basic task execution
    #[tokio::test]
    async fn test_executor_simple() {
        // Create executor
        let exec = match create_executor().await {
            Some(exec) => exec,
            None => return,
        };
        struct ExampleTask {
            count: Mutex<i32>
        }
        impl TaskFactory for ExampleTask {
            fn init(this: Arc<Self>, _executor: Arc<Executor>, _db_ref: Database, _client: reqwest::Client) -> Task {
                Box::new(async move {
                    let mut count = this.count.lock().unwrap();
                    *count += 1;
                    Ok(())
                })
            }
        }
        let mut handles = Vec::new();
        let example = Arc::new(ExampleTask {count: Mutex::new(0)});
        for _ in 0..10 {
            handles.push(exec.execute(&example))
     
        }
        for handle in handles {
            let _ = handle.await;
        }
        let final_count = example.count.lock().unwrap();
        assert_eq!(*final_count, 10);
    
    }

    // Create fibonacci task (Test recursive)
    #[tokio::test]
    async fn test_executor_recursive() {
        let exec = match create_executor().await {
            Some(exec) => exec,
            None => return,
        };
        struct FibonacciTask {
            n: i32,
            v: Mutex<i32>
        }
        impl TaskFactory for FibonacciTask {
            fn init(this: Arc<Self>, executor: Arc<Executor>, _db_ref: Database, _client: reqwest::Client) -> Task {
                Box::new(async move {
                    if this.n == 0 {
                        *this.v.lock().unwrap() = 0;
                    }
                    else if this.n == 1 {
                        *this.v.lock().unwrap() = 1;  
                    }
                    else {
                        let fib_1 = Arc::new(FibonacciTask {n: this.n-1, v: Mutex::new(0)});
                        let fib_2 = Arc::new(FibonacciTask {n: this.n-2, v: Mutex::new(0)});
                        // What we are testing: recursive call on executor
                        let handle_1 = executor.execute(&fib_1);
                        let handle_2 = executor.execute(&fib_2);
                        let _ = handle_1.await; let _ = handle_2.await;
                        *this.v.lock().unwrap() = *fib_1.v.lock().unwrap() + *fib_2.v.lock().unwrap();
                    }
                    Ok(())
                })
            }
        }

        let example = Arc::new(FibonacciTask {n: 6, v: Mutex::new(0)});
        let _ = exec.execute(&example).await;
        assert_eq!(*example.v.lock().unwrap(), 8);
    }
}