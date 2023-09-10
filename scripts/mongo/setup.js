const DB_NAME = "quantify"
const DELETE_DB = true

function main() {
    // Create Mongo Client
    client = Mongo();
    db = client.getDB(DB_NAME);
    // Potentially delete
    if (DELETE_DB) {
        db.dropDatabase();
    }
    else if (client.getDBNames().indexOf("DB_NAMES") != -1) {
        console.warn("Database already exists");
        return
    }
    // Create database
    
        // Ticker metadata
    db.createCollection("tickers", {
        validator: {
            $jsonSchema: {
                bsonType: "object",
                title: "Ticker Object Validation",
                required: ["ticker", "exchange"],
                properties: {
                    ticker: {
                        bsonType: "string",
                        description: "The ticker name for this asset"
                    },
                    company: {
                        bsonType: "string",
                        description: "The company name for this asset"
                    },
                    exchange: {
                        bsonType: "string",
                        description: "The exchange on which the ticker is traded"
                    }
                }
            }
        }
    });

        // Candle Data
    db.createCollection("minute_candle", {
        timeseries: {
            timeField: "timestamp",
            metaField: "ticker",
            granularity: "minutes"
        }
    });

    db.createCollection("hour_candle", {
        timeseries: {
            timeField: "timestamp",
            metaField: "ticker",
            granularity: "hours" 
        }
    });

    db.createCollection("day_candle", {
    });

        // Fundamentals
    db.createCollection("fundamentals", {       
    });

    console.log("Success");
}

main()