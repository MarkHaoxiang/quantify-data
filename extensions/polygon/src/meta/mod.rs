use core::fmt;
use std::error::Error;

use chrono::NaiveDate;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Metadata {
    /// Whether the asset is actively traded
    pub active: bool,
    /// Address of headquarters
    pub address: Address,
    /// Central Index Key
    pub cik: String,
    /// Composite OpenFIGI number
    pub composite_figi: String,
    /// Currency that this asset is trade with
    #[serde(alias="currency_name")]
    pub currency: String,
    /// Last date the asset was traded if it had been delisted
    pub delisted_utc: String,
    /// Description of company
    pub description: String,
    /// Homepage URL of company website
    pub homepage_url: String,
    /// Date of first public listing of symbol
    pub list_date: NaiveDate,
    /// Locale of asset
    pub locale: Locale,
    /// Market type of asset
    #[serde(alias="market")]
    pub market_type: MarketType,
    /// The most recent close price of the ticker multiplied by weighted outstanding shares
    pub market_cap: f64,
    /// Company name
    pub name: String,
    /// Phone number of company
    pub phone_number: String,
    /// The ISO code of the primary listing exchange for the asset
    pub primary_exchange: String,
    /// Round lot size of security
    pub round_lot: f64,
    /// The share Class OpenFIGI number for the ticker
    pub share_class_figi: String,
    /// Standard industrial classification (SIC) code for the ticker
    pub sic_code: String,
    /// SIC description for the ticker
    pub sic_description: String,
    /// Exchange symbol (ticker) that this item is traded under
    pub ticker: String,
    /// The root of the ticker
    pub ticker_root: String,
    /// The suffix of the ticker
    pub ticker_suffix: String,
    // Total number of employees
    pub total_employees: f64,
    // Type of asset
    pub asset_type: String,
    // The shares outstanding calculated assuming all shares of other share classes are converted to this share class
    pub weighted_shares_outstanding: f64,
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Metadata: {}{}{}", self.ticker, self.name, self.primary_exchange)
    }
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Address {
    pub address1: String,
    pub city: String,
    pub postal_code: String,
    pub state: String,
}

// Enum for locale
#[derive(Deserialize, Debug)]
pub enum Locale {
    #[serde(alias="us")]
    US,
    #[serde(alias="global")]
    Global,
    Unknown, // To indicate a missing Locale. Not part of the API definition
}

impl Default for Locale {
    fn default() -> Self {
        Locale::Unknown
    }
}

// Enum for market type
#[derive(Deserialize, Debug)]
pub enum MarketType {
    #[serde(alias="stocks")]
    Stocks,
    #[serde(alias="crypto")]
    Crypto,
    #[serde(alias="fx")]
    FX,
    #[serde(alias="otc")]
    OTC,
    #[serde(alias="indices")]
    Indices,
    Unknown, // To indicate a missing MarketType. Not part of the API definition
}

impl Default for MarketType {
    fn default() -> Self {
        MarketType::Unknown
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct PolygonMetaResponse {
    // Metadata
    results: Metadata,
    // Status of the request
    status: String,
    // Any errors thrown
    error: String,
}

/// Get ticker metadata
pub(super) async fn get_meta (
    ticker: &str,
    client: &Client,
    api_key: &str,
    date: Option<&NaiveDate>,
) -> Result<Metadata, Box<dyn Error + Send + Sync>> {

    // Construct request
    let mut request = String::from(format!("https://api.polygon.io/v3/reference/tickers/{}?", ticker));
    if date.is_some() {
        request.push_str(date.unwrap().format("%Y-%m-%d&").to_string().as_str());
    }
    request.push_str(format!("apiKey={}", api_key).as_str());

    println!("{}", request);
    
    // Send request. Await response
    let response = client
        .get(request)
        .send()
        .await?
        .text()
        .await?;

    // Parse response
    let res: PolygonMetaResponse = serde_json::from_str(&response)?;

    // Check if there is an error. If there is, return it
    if res.status == "ERROR" {
        return Err(res.error.into());
    }

    return Ok(res.results);
}