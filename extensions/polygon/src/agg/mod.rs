use core::fmt;
use std::error::Error;

use chrono::{DateTime, NaiveDate, Utc};
use chrono::serde::ts_milliseconds;
use reqwest::Client;
use serde::Deserialize;

use crate::PolygonResponseError;

const MAX_POLYGON_AGGS_LIMIT: i32 = 50000; // as defined by Polygon.io's API

#[derive(Deserialize, Default)]
pub struct AggregateData {
    // The date and time this data pertains to
    #[serde(with="ts_milliseconds", rename="t")]
    pub datetime: DateTime<Utc>,
    // The opening price for the asset in the given window
    #[serde(rename="o")]
    pub open: f64,
    // The high price for the asset in the given window
    #[serde(rename="h")]
    pub high: f64,
    // Thh low price for the asset in the given window
    #[serde(rename="l")]
    pub low: f64,
    // The closing price for the asset in the given window
    #[serde(rename="c")]
    pub close: f64,
    // The number of shares traded for the asset in the given widnow
    #[serde(rename="v")]
    pub volume: f64,
    // The number of transactions traded in the given window
    #[serde(rename="n")]
    pub num_transactions: f64,
    // Whether or not the aggregate is for an OTC ticker (NOTE: Polygon.io will leave out this field if false)
    #[serde(default)]
    pub otc: bool,
    // The volume weighted average price
    #[serde(rename="vw")]
    pub vwap: f64,
}

impl fmt::Display for AggregateData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Aggregate Bars: {}{}{}{}{}", self.datetime, self.open, self.high, self.low, self.close)
    }
}
impl fmt::Debug for AggregateData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl PartialEq for AggregateData {
    fn eq(&self, other: &Self) -> bool {
        return self.datetime == other.datetime
            && self.open == other.open
            && self.high == other.high
            && self.low == other.low
            && self.close== other.close
            && self.volume == other.volume
            && self.num_transactions == other.num_transactions
            && self.otc == other.otc
            && self.vwap == other.vwap
    }
}

// Enum for passing intervals
pub enum Interval {
    Seconds(i32),
    Minutes(i32),
    Hours(i32),
    Days(i32),
    Weeks(i32),
    Months(i32),
    Quarters(i32),
    Years(i32),
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
struct PolygonAggResponse {
    // The results of the request
    results: Vec<AggregateData>,
    // The url to call for the next page of results (pagination)
    next_url: String,
    // Status of the request
    status: String,
    // Any errors thrown
    error: String,
}

/// Get Aggregate (Bar) data. Uses pagination to deal with the limit on data points
/// 
/// # Arguments to pass into API
/// 
/// ticker - Target ticker
/// start_date - Start date of the data fetched
/// end_date - End date of the data fetched
/// interval - The granularity of the data. Defined by the enum Interval, which defines both the multiplier (eg. 5) and the interval (eg. minutes)
/// adjusted - Whether the data is adjusted for splits
/// limit - Limit to the number of data points fetched. Polygon.io defines the maximum limit to be 50000
pub(super) async fn get_aggs (
    ticker: &str,
    client: &Client,
    api_key: &str,
    start_date: &NaiveDate,
    end_date: &NaiveDate,
    interval: &Interval,
    adjusted: &bool,
) -> Result<Vec<AggregateData>, Box<dyn Error>> {

    // Construct request
    let mut request = String::from(format!("https://api.polygon.io/v2/aggs/ticker/{}/range", ticker));
    match interval {
        Interval::Seconds(m) => request.push_str(format!("/{}/second", m).as_str()),
        Interval::Minutes(m) => request.push_str(format!("/{}/minute", m).as_str()),
        Interval::Hours(m) => request.push_str(format!("/{}/hour", m).as_str()),
        Interval::Days(m) => request.push_str(format!("/{}/day", m).as_str()),
        Interval::Weeks(m) => request.push_str(format!("/{}/week", m).as_str()),
        Interval::Months(m) => request.push_str(format!("/{}/month", m).as_str()),
        Interval::Quarters(m) => request.push_str(format!("/{}/quarter", m).as_str()),
        Interval::Years(m) => request.push_str(format!("/{}/year", m).as_str()),
    }
    request.push_str(start_date.format("/%Y-%m-%d").to_string().as_str());
    request.push_str(end_date.format("/%Y-%m-%d").to_string().as_str());
    request.push_str(format!("?adjusted={}", adjusted).as_str());
    request.push_str(format!("&limit={}", MAX_POLYGON_AGGS_LIMIT).as_str()); // Use the maximum limit
    request.push_str(format!("&apiKey={}", api_key).as_str());

    let mut aggs: Vec<AggregateData> = Vec::new();
    
    // Send request. Await response
    let mut response = client
        .get(request)
        .send()
        .await?
        .text()
        .await?;

    // Parse response
    let mut res: PolygonAggResponse = serde_json::from_str(&response)?;

    // Check if there is an error. If there is, return it
    if res.status == "ERROR" {
        return Err(Box::new(PolygonResponseError{error: res.error}));
    }

    // Add the results to the output vector
    aggs.append(&mut res.results);
    
    // Pagination
    // For reference: https://polygon.io/blog/aggs-api-updates
    // Query the value of "next_url" to get the next page
    // The value under the "results" list shows the results
    while !res.next_url.is_empty() {
        res.next_url.push_str(format!("&apiKey={}", api_key).as_str());
        response = client
            .get(res.next_url)
            .send()
            .await?
            .text()
            .await?;

        res = serde_json::from_str(&response)?;

        aggs.append(&mut res.results);
    }

    return Ok(aggs);
}