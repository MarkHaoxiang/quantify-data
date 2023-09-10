// Add tickers
mod add_ticker;
pub use add_ticker::AddTickerTask;
// Candle data control
mod candle;
pub use candle::{UpdateCandleDataTask, Granularity};


/// This module provides utility functions to resolve
/// discrepancies involving multiple data sources
mod resolver {
    use std::{hash::Hash, collections::HashMap};

    /// Returns the most common value. The tiebreaker is the index of the value.
    pub fn mode<T: Eq + Hash>(values: &Vec<T>) -> Option<&T> {
        let mut map: HashMap<&T, u32> = HashMap::new();
        let mut maximum: u32 = 0;
        for v in values {
            let count = map.entry(v).or_insert(0);
            *count += 1;
            maximum = if maximum >= *count {maximum} else {*count};
        }
        let map_iter = map.into_iter();
        for (k, v)  in map_iter {
            if v == maximum {
                return Some(k);
            }
        }
        None
    }

    /// Returns the median value. The tiebreaker (even collection) is the smaller value.
    pub fn median<T: Ord + Hash>(_values: &Vec<T>) -> Option<&T> {
        //TODO
        None
    }
}