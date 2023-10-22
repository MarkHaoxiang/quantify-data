//! API Managers are wrappers around API clients that can be extended as needed to include different functionality
//! as needed. Simply create and implement a new trait, then add it to the APIManager trait as a supertrait.
//! 
//! This file also includes the manager priority list (from first to last) of data resolution.
//! It defines which data source is the primary one (the first) and which are the subsequent backups.
//! The definition can be found (and edited) in `get_managers_in_priority(web_client)` below;
//! 
//! To use the API Managers,
//! 1. Create a trait
//! 2. Implement the trait for each Manager
//! 3. Import the trait to this file and add it to the APIManager trait as a supertrait.
//! 4. Call `get_managers_in_priority(web_client)` to obtain list of managers in order of priority
//! 5. Loop through
//! 
//! # Examples
//! 
//! ```
//! // trait_impl.rs
//! 
//! // Define trait and functions
//! pub trait DataProcesser {
//!     fn get_and_process_data() -> Option<Vec<Data>>;
//! }
//! 
//! // Implement trait for all managers
//! impl DataProcesser for PolygonAPIManager {
//!     fn get_and_process_data() -> Option<Vec<Data>> {
//!         ...
//!     }
//! }
//! ...
//! ```
//! 
//! ```
//! // api_managers.rs
//! 
//! pub trait APIManagers : Sync + Send
//!     ...
//!     + DataProcessor {}
//! ```
//! 
//! ```
//! // trait_use.rs
//! 
//! // Get and loop through managers
//! fn main() {
//!     let managers: Vec<Box<dyn APIManager>> = get_managers_in_priority(web_client.clone());
//!     let data: Vec<Vec<Data>>;
//!     for manager in managers {
//!         let res = manager.get_and_process_data();
//!         if res.is_some() {
//!             data.push(res);
//!         }
//!     }
//!     ...
//! }
//! ```

use polygon::PolygonRESTClient;
use tiingo::TiingoRESTClient;

// Traits to extend APIManager with as supertraits (implemented elsewhere)
use crate::executor::tasks::candle::aggregate_data_interface::AggregateDataInterface;

// APIManager trait. Inherits from all supertraits (as above)
// Add all implemented traits here to integrate functionality
pub trait APIManager : Sync + Send
    + AggregateDataInterface {}

// API Managers

// Polygon
pub struct PolygonAPIManager {
    pub client: PolygonRESTClient,
}
impl PolygonAPIManager {
    fn boxed(web_client: reqwest::Client) -> Box<PolygonAPIManager> {
        Box::new(PolygonAPIManager {
            client: PolygonRESTClient::new(web_client)
        })
    }
}
impl APIManager for PolygonAPIManager {}

// Tiingo
pub struct TiingoAPIManager {
    pub client: TiingoRESTClient
}
impl TiingoAPIManager {
    fn boxed(web_client: reqwest::Client) -> Box<TiingoAPIManager> {
        Box::new(TiingoAPIManager {
            client: TiingoRESTClient::new(web_client)
        })
    }
}
impl APIManager for TiingoAPIManager {}

// Returns API Managers in order of the priority list (top priority to bottom priority)
pub fn get_managers_in_priority(web_client: reqwest::Client) -> Vec<Box<dyn APIManager>> {
    // Priority list
    return vec!(
        PolygonAPIManager::boxed(web_client.clone()), // Primary source
        TiingoAPIManager::boxed(web_client.clone())
    );
}