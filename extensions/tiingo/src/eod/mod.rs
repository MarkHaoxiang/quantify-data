use core::fmt;
use std::option::Option;

use chrono::{prelude::DateTime, Utc};
use reqwest::Client;

// TODO
// Avoid panicking on fail

// An object created from the Tiingo End-of-Day Endpoint API
// https://www.tiingo.com/documentation/end-of-day
pub struct EoD {
    // The date this data pertains to
    pub date: DateTime<Utc>,
    // The opening price for the asset on the given day
    pub open: f64,
    // The high price for the asset on the given date
    pub high: f64,
    // Thh low price for the asset on the given date
    pub low: f64,
    // The closing price for the asset on the given date
    pub close: f64,
    // The number of shares traded for the asset
    pub volume: u64,
    // The adjusted opening price for the asset on the given date
    pub adj_open: f64,
    // The adjusted high price for the asset on the given date
    pub adj_high: f64,
    // The adjusted high price for the asset on the given date
    pub adj_low: f64,
    // The adjusted opening price for the asset on the given date
    pub adj_close: f64, 
    // The adjusted opening price for the asset on the given date
    pub adj_volume: u64, 
    // The dividend paid out on the date (ex-dividend date)
    pub dividend: f64,
    // The factor used to adjust prices
    pub split: f64,
}

impl fmt::Display for EoD {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "End Of Day: {}{}{}{}{}", self.date, self.open, self.high, self.low, self.close)
    }
}
impl fmt::Debug for EoD {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl PartialEq for EoD {
    fn eq(&self, other: &Self) -> bool {
        return self.date == other.date
            && self.open == other.open
            && self.close== other.close
            && self.high == other.high
            && self.low == other.low
            && self.volume == other.volume
            && self.adj_open == other.adj_open
            && self.adj_high == other.adj_high
            && self.adj_low == other.adj_low
            && self.adj_close == other.adj_close
            && self.volume == other.volume
            && self.dividend == other.dividend
            && self.split == other.split 
    }
}

// Resample frequency choices
pub enum ResampleFreq {
    DAILY,
    WEEKLY,
    MONTHLY,
    ANNUALLY
}
impl fmt::Display for ResampleFreq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResampleFreq::DAILY => write!(f, "daily"),
            ResampleFreq::WEEKLY => write!(f, "weekly"),
            ResampleFreq::MONTHLY => write!(f, "monthly"),
            ResampleFreq::ANNUALLY => write!(f, "annually"),
        }
    }
}

pub(super) async fn get_eod (
    ticker: &str,
    client: &Client,
    api_key: &str,
    start_date: &Option<DateTime<Utc>>,
    end_date: &Option<DateTime<Utc>>,
    resample_freq: &Option<ResampleFreq>
) -> Vec<EoD> {
    // Construct request
    let mut request = String::from(format!("https://api.tiingo.com/tiingo/daily/{}/prices?format=csv&token={}", ticker, api_key));
    if let Some(start_date) = start_date {
        request.push_str(format!("&startDate={}", start_date.format("%F")).as_str());
    }
    if let Some(end_date) = end_date {
        request.push_str(format!("&endDate={}", end_date.format("%F")).as_str());
    }
    if let Some(resample_freq) = resample_freq {
        request.push_str(format!("&resampleFreq={}", resample_freq).as_str());
    }
    // Send request. Await response
    let response = client
        .get(request)
        .header("Content-Type", "application/json")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    // Parse response
    let mut result: Vec<EoD> = Vec::new();
    let mut count = 0;
    let (mut date, mut open, mut high, mut low, mut close, mut volume, mut adj_open, 
         mut adj_high, mut adj_low, mut adj_close, mut adj_volume, mut dividend, mut split) =
    (0,0,0,0,0,0,0,0,0,0,0,0,0);

    for line in response.lines() {
        count += 1;
        let data= line.split(',');
        if count == 1 {
            for (i, column) in data.enumerate() {
                match column {
                    "date"        => date = i,
                    "close"       => close = i,
                    "high"        => high = i,
                    "low"         => low = i,
                    "open"        => open = i,
                    "volume"      => volume = i,
                    "adjClose"    => adj_close = i,
                    "adjHigh"     => adj_high = i,
                    "adjLow"      => adj_low = i,
                    "adjOpen"     => adj_open = i,
                    "adjVolume"   => adj_volume = i,
                    "divCash"     => dividend = i,
                    "splitFactor" => split = i,
                    &_ => panic!("Unknown data in response. Check Tiingo API reference.")
                }
            }
        }
        else {
            let data: Vec<&str> = data.collect();

            let date_string = data[date];
            let date_parsed = match format!("{date_string}T00:00:00.000Z").parse::<DateTime<Utc>>() {
                Ok(value) => value,
                Err(_error) => panic!("Parse Error {date_string}"),
            };

            result.push
            (
                EoD
                {
                    date: date_parsed,
                    open: data[open].parse().unwrap(),
                    high: data[high].parse().unwrap(),
                    low: data[low].parse().unwrap(),
                    close: data[close].parse().unwrap(),
                    volume: data[volume].parse().unwrap(),
                    adj_open: data[adj_open].parse().unwrap(),
                    adj_high: data[adj_high].parse().unwrap(),
                    adj_low: data[adj_low].parse().unwrap(),
                    adj_close: data[adj_close].parse().unwrap(),
                    adj_volume: data[adj_volume].parse().unwrap(),
                    dividend: data[dividend].parse().unwrap(),
                    split: data[split].parse().unwrap()
                }
            )
        }
    }
    return result;
}
