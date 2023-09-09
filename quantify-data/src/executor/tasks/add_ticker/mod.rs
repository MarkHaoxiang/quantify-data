pub struct AddTickerTask {
    ticker: String
}

impl AddTickerTask {
    pub fn new(ticker: &str) -> AddTickerTask{
        let t = String::from(ticker);
        AddTickerTask{ticker: t}
    }
}

impl TaskFactory for AddTickerTask {
    fn init (this: Arc<Self>, executor: &Executor, db_ref: Database) -> Task {
        Box::new(async move {
            // Initialize clients
            let client = reqwest::Client::new();
            let tiingo_client = TiingoRESTClient::new(client);

            // Get data
            let ticker_data: Metadata = tiingo_client.get_metadata(&this.ticker).await;
                // TODO: Polygon data with check

            // Update meta table
            {
                // Get the mongo client lock
                let collection: Collection<Document> = db_ref.collection::<Document>("tickers");
                // Update the document
                    // TODO: Remaining fields of document
                let ticker_document = doc! {
                    "ticker": ticker_data.name.to_lowercase(),
                    "exchange": ticker_data.exchange_code.to_lowercase()
                };
                    // TODO: Error handling
                collection.delete_many(
                    doc! {
                        "ticker": ticker_data.name.to_lowercase(),
                    } ,
                    None
                ).await.unwrap();
                collection.insert_one(ticker_document, None).await.unwrap();
            }      
        })
    }
}