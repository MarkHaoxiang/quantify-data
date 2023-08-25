use std::env;
use eod::get_eod;
use meta::get_metadata;
use reqwest::Client;
use chrono::NaiveDate;

mod eod;
mod meta;

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
    pub fn new(web_client: Client) -> TiingoRESTClient{
        TiingoRESTClient {web_client, api_key: get_api_key()}
    }

    pub async fn get_metadata(
        &self,
        ticker: &str) -> meta::Metadata
    {
        get_metadata(ticker, &self.web_client, &self.api_key).await
    }

    pub async fn get_eod(
        &self,
        ticker: &str,
        start_date: &Option<NaiveDate>,
        end_date: &Option<NaiveDate>,
        resample_freq: &Option<eod::ResampleFreq>) -> Vec<eod::EoD> 
    {
        get_eod(ticker, &self.web_client, &self.api_key, start_date, end_date, resample_freq).await
    }
}

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
        let fetched_result = get_metadata("GOOGL", &client, &get_api_key()).await;
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
        ).await;

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