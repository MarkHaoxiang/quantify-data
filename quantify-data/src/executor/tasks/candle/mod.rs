// MongoDB constants
const DAY_CANDLE_COLLECTION: &str = "day_candle";
const HOUR_CANDLE_COLLECTION: &str = "hour_candle";
const MINUTE_CANDLE_COLLECTION: &str = "minute_candle";

pub enum Granularity {
    Days(i32),
    Hours(i32),
    Minutes(i32)
}

// Definitions
#[derive(Debug, Serialize, Deserialize)]
struct CandleData {
    ticker: String,
    timestamp: NaiveDateTime,
    open: f64,
    close: f64,
    high: f64,
    low: f64,
    volume: i64,
    num_transactions: i64
}

pub struct UpdateCandleDataTask {
    ticker: String,
    granularity: Granularity
}

impl UpdateCandleDataTask {
    pub fn new(ticker: &str, granularity: Granularity) -> UpdateCandleDataTask{
        let t = String::from(ticker);
        UpdateCandleDataTask{ticker: t, granularity}
    }
}

impl TaskFactory for UpdateCandleDataTask {
    fn init (this: Arc<Self>, executor: &Executor, db_ref: Database) -> Task {
        Box::new(async move {
            // Initialize clients
            println!("Running");
            let client = reqwest::Client::new();
            let polygon_client = PolygonRESTClient::new(client);

            // Get collection handle and granularity
            let col_name: &str;
            let granularity: i32;
            match this.granularity {
                Granularity::Days(m) => {
                    col_name = DAY_CANDLE_COLLECTION;
                    granularity = m;
                },
                Granularity::Hours(m) => {
                    col_name = HOUR_CANDLE_COLLECTION;
                    granularity = m;
                },
                Granularity::Minutes(m) => {
                    col_name = MINUTE_CANDLE_COLLECTION;
                    granularity = m;
                }
            };
            let col_ref = db_ref.collection::<CandleData>(col_name);

            // Get latest entry
            let find_options = FindOptions::builder()
                .sort(doc! { "timestamp": -1 })
                .limit(1)
                .build();
            let mut cursor = col_ref.find(None, find_options).await.unwrap();
            
            if let Some(latest_data) = cursor.try_next().await.unwrap() {
                // Latest data found

            } else {
                // Latest data not found
            }
        })
    }
}