use std::{env, error::Error};
use eod::get_eod;
use meta::get_metadata;
use reqwest::Client;
use chrono::NaiveDate;

pub mod eod;
pub mod meta;

/// A client to access Tiingo REST APIs
/// 
/// See https://www.tiingo.com/documentation/general/overview
pub struct TiingoRESTClient {
    web_client: Client,
    api_key: String
}
impl Default for TiingoRESTClient {
    fn default() -> Self {
        TiingoRESTClient { web_client: Client::new(), api_key: get_api_key()}
    }
}
impl TiingoRESTClient {
    /// Creates a new TiingoRESTClient
    /// 
    /// # Arguments
    /// 
    /// * `web_client`
    pub fn new(web_client: Client) -> TiingoRESTClient{
        TiingoRESTClient {web_client, api_key: get_api_key()}
    }

    /// Gets Metadata
    pub async fn get_metadata(
        &self,
        ticker: &str) -> Result<meta::Metadata, Box<dyn Error + Send + Sync>>
    {
        get_metadata(ticker, &self.web_client, &self.api_key).await
    }

    /// Gets end-of-day candle data
    pub async fn get_eod(
        &self,
        ticker: &str,
        start_date: &Option<NaiveDate>,
        end_date: &Option<NaiveDate>,
        resample_freq: &Option<eod::ResampleFreq>) -> Result<Vec<eod::EoD>, Box<dyn Error + Send + Sync>>
    {
        get_eod(ticker, &self.web_client, &self.api_key, start_date, end_date, resample_freq).await
    }
}

/// Returns the api key stored in env variable TIINGO_API_KEY
pub fn get_api_key() -> String {
    let key = "TIINGO_API_KEY";
    match env::var(key) {
        Ok(v) => return v,
        Err(e) => panic!("${} is not set - {}$", key, e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Client;
    use crate::{eod::{get_eod, ResampleFreq, EoD}, meta::get_metadata};
    use super::*;

    #[test]
    fn test_get_api_key() {
        let k: String = get_api_key();
        let l = k.len();
        assert_eq!(l, 40);
    }

    #[tokio::test]
    async fn test_rest_api() {
        let client = Client::new();
        // Metadata
        let fetched_result = get_metadata("GOOGL", &client, &get_api_key()).await.unwrap();
        assert_eq!(fetched_result.ticker, "GOOGL");
        assert_eq!(fetched_result.name, "Alphabet Inc - Class A");

        // EoD
        let test_start = NaiveDate::from_ymd_opt(2014, 1, 1).unwrap();
        let test_end = NaiveDate::from_ymd_opt(2014, 1, 2).unwrap(); 

        let fetched_result = get_eod(
            "GOOGL",
            &client,
            &get_api_key(),
            &Some(test_start), 
            &Some(test_end),
            &Some(ResampleFreq::DAILY)
        ).await.unwrap();

        let correct_result = EoD
            {
                date: NaiveDate::from_ymd_opt(2014, 1, 2).unwrap(),
                open: 1115.46,
                high: 1117.75,
                low: 1108.26,
                close: 1113.12,
                volume: 3639100,
                adj_open: 27.9728495066623,
                adj_high: 28.0302767791511,
                adj_low: 27.7922921433791,
                adj_close: 27.9141683635953,
                adj_volume: 145114657,
                dividend: 0.0,
                split: 1.0
            };

        assert_eq!(fetched_result[0], correct_result);

    }
}