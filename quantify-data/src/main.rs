use executor::Executor;

mod executor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uri = std::env::var("QUANTIFY_DATABASE_URI").expect("You must set the QUANTIFY_DATABASE_URI environment var!");
    let executor = Executor::build(uri.as_str()).await.unwrap();

    Ok(())   
}