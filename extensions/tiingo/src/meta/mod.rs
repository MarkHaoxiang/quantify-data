use core::fmt;

use reqwest::Client;
use chrono::NaiveDate;
use serde_json::Value;

pub struct Metadata {
    pub ticker: String,
    pub name: String,
    pub exchange_code: String,
    pub description: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Metadata: {}{}{}", self.ticker, self.name, self.exchange_code)
    }
}

impl PartialEq for Metadata {
    fn eq(&self, other: &Self) -> bool {
        return self.ticker == other.ticker
            && self.name == other.name
            && self.exchange_code == other.exchange_code
            && self.description == other.description
            && self.start_date == other.start_date
            && self.end_date == other.end_date
    }
}

pub(super) async fn get_metadata(
    ticker: &str,
    client: &Client,
    api_key: &str
) -> Metadata {
    // Construct request
    let request: String = String::from(format!("https://api.tiingo.com/tiingo/daily/{}?token={}", ticker, api_key));

    // Send request
    let response: String = client
        .get(request)
        .header("Content-Type", "application/json")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    // Parse response
    let v: Value = serde_json::from_str(&response).unwrap();
    let start_date = v["startDate"].to_string()
        .trim_matches('"').parse::<NaiveDate>().unwrap();
    let end_date = v["endDate"].to_string()
        .trim_matches('"').parse::<NaiveDate>().unwrap();

    Metadata
    {
        ticker: v["ticker"].to_string().trim_matches('"').to_string(),
        name: v["name"].to_string().trim_matches('"').to_string(),
        exchange_code: v["exchangeCode"].to_string().trim_matches('"').to_string(),
        description: v["description"].to_string().trim_matches('"').to_string(),
        start_date: start_date,
        end_date: end_date
    }
}