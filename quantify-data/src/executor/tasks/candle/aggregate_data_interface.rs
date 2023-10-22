use chrono::NaiveDate;
use std::error::Error;
use async_trait::async_trait;
use crate::executor::tasks::api_managers::{
    PolygonAPIManager,
    TiingoAPIManager
};
use polygon::Interval;

use crate::executor::tasks::candle::{CandleData, Granularity};

#[async_trait]
pub trait AggregateDataInterface {
    async fn get_agg_candle_data(
        &self,
        ticker: &str,
        start_date: &NaiveDate,
        end_date: &NaiveDate,
        granularity: &Granularity) -> Result<Vec<CandleData>, Box<dyn Error + Send + Sync>>;
}

// Implementations

#[async_trait]
impl AggregateDataInterface for PolygonAPIManager {
    async fn get_agg_candle_data(
        &self,
        ticker: &str,
        start_date: &NaiveDate,
        end_date: &NaiveDate,
        granularity: &Granularity) -> Result<Vec<CandleData>, Box<dyn Error + Send + Sync>> {
        
        // Retrieve from PolygonIO
        let interval = &match granularity {
            Granularity::Days(m) => Interval::Days(*m),
            Granularity::Hours(m) => Interval::Hours(*m),
            Granularity::Minutes(m) => Interval::Minutes(*m)
        };
        let adjusted = &false;
        let agg_data = self.client.get_aggs(
            ticker,
            start_date,
            end_date,
            interval,
            adjusted).await?;

        // Convert PolygonIO Aggregate Data into Candle Data
        let mut candle_data: Vec<CandleData> = Vec::new();
        for agg in &agg_data {
            candle_data.push(CandleData {
                ticker: ticker.to_string(),
                timestamp: agg.datetime,
                open: agg.open,
                close: agg.close,
                high: agg.high,
                low: agg.low,
                volume: agg.volume as i64,
                num_transactions: agg.num_transactions as i64
            });
        }
        return Ok(candle_data);
    }
}

#[async_trait]
impl AggregateDataInterface for TiingoAPIManager {
    async fn get_agg_candle_data(
        &self,
        ticker: &str,
        start_date: &NaiveDate,
        end_date: &NaiveDate,
        granularity: &Granularity) -> Result<Vec<CandleData>, Box<dyn Error + Send + Sync>> {
        
        // TODO
        return Ok(Vec::new());
    }
}