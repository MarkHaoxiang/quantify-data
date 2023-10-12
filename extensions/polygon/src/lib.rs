use std::{env, error::Error};
use reqwest::Client;
use chrono::NaiveDate;
use agg::get_aggs;
use meta::get_meta;

// Re-exporting
pub use agg::{AggregateData, Interval};
pub use meta::{Metadata, Address, Locale, MarketType};

mod agg;
mod meta;

// Constants
pub const DEFAULT_POLYGON_MAX_HISTORICAL_WEEKS: i64 = 260; // Maximum time in the past from which data can be pulled

pub struct PolygonRESTClient {
    web_client: Client,
    api_key: String,
}

impl Default for PolygonRESTClient {
    fn default() -> Self {
        PolygonRESTClient { web_client: Client::new(), api_key: get_api_key()}
    }
}

impl PolygonRESTClient {
    pub fn new(web_client: Client) -> PolygonRESTClient {
        return PolygonRESTClient { web_client: web_client, api_key: get_api_key() };
    }

    pub async fn get_aggs(
        &self,
        ticker: &str,
        start_date: &NaiveDate,
        end_date: &NaiveDate,
        interval: &Interval,
        adjusted: &bool,) -> Result<Vec<AggregateData>, Box<dyn Error + Send + Sync>>
    {
        get_aggs(ticker, &self.web_client, &self.api_key, start_date, end_date, interval, adjusted).await
    }

    pub async fn get_meta (
        &self,
        ticker: &str,
        date: Option<&NaiveDate>,
    ) -> Result<Metadata, Box<dyn Error + Send + Sync>> {
        get_meta(ticker, &self.web_client, &self.api_key, date).await
    }
}

pub fn get_api_key() -> String {
    let key = "POLYGON_API_KEY";
    match env::var(key) {
        Ok(v) => return v,
        Err(e) => panic!("${} is not set - {}$", key, e.to_string())
    }
}

// Tests
#[cfg(test)]
mod tests {
    use crate::{get_api_key, Interval, AggregateData};
    use crate::agg::get_aggs;
    use crate::meta::get_meta;
    use reqwest::Client;
    use chrono::{NaiveDate, Utc, TimeZone};

    #[test]
    fn test_get_api_key() {
        let k: String = get_api_key();
        let l = k.len();
        assert_eq!(l, 32);
    }

    #[tokio::test]
    async fn test_get_meta() {
        let fetched_result = get_meta(
            "NFLX",
            &Client::new(),
            &get_api_key(),
            Some(&NaiveDate::from_ymd_opt(2023, 8, 1).unwrap()),
        ).await.unwrap();

        assert_eq!(fetched_result.ticker, "NFLX");
        assert_eq!(fetched_result.market_cap, 183094943110.67);
    }

    #[tokio::test]
    async fn test_get_aggs() {
        let fetched_result = get_aggs(
            "NFLX",
            &Client::new(),
            &get_api_key(),
            &NaiveDate::from_ymd_opt(2022, 8, 1).unwrap(),
            &NaiveDate::from_ymd_opt(2023, 8, 1).unwrap(),
            &Interval::Minutes(5),
            &true,
        ).await.unwrap();

        let correct_first = AggregateData {
            datetime: Utc.with_ymd_and_hms(2022, 8, 1, 8, 10, 0).unwrap(),
            open: 224.7,
            high: 224.7,
            low: 223.25,
            close: 223.25,
            volume: 1901.0,
            num_transactions: 83.0,
            otc: false,
            vwap: 223.7848,
        };

        let correct_last = AggregateData {
            datetime: Utc.with_ymd_and_hms(2023, 8, 1, 23, 55, 0).unwrap(),
            open: 437.06,
            high: 437.06,
            low: 437.06,
            close: 437.06,
            volume: 279.0,
            num_transactions: 19.0,
            otc: false,
            vwap: 437.3132,
        };

        assert_eq!(fetched_result[0], correct_first);
        assert_eq!(fetched_result[fetched_result.len()-1], correct_last);
    }
}