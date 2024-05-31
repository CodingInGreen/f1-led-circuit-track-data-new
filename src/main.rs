use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer};
use std::error::Error;
use tokio;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
struct LocationData {
    x: f64,
    y: f64,
    #[serde(deserialize_with = "deserialize_datetime")]
    date: DateTime<Utc>,
    driver_number: u32,
}

fn deserialize_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    DateTime::parse_from_rfc3339(&s)
        .map_err(de::Error::custom)
        .map(|dt| dt.with_timezone(&Utc))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let session_key = "9149";
    let driver_numbers = vec![
        1, 2, 4, 10, 11, 14, 16, 18, 20, 22, 23, 24, 27, 31, 40, 44, 55, 63, 77, 81
    ];

    let client = Client::new();
    let mut all_data: Vec<LocationData> = Vec::new();

    for driver_number in driver_numbers {
        let url = format!("https://api.openf1.org/v1/location?session_key={}&driver_number={}", session_key, driver_number);
        let resp = client.get(&url).send().await?;
        if resp.status().is_success() {
            let data: Vec<LocationData> = resp.json().await?;
            all_data.extend(data);
        } else {
            eprintln!("Failed to fetch data for driver {}: HTTP {}", driver_number, resp.status());
        }
    }

    // Sort the data by the date field
    all_data.sort_by_key(|d| d.date);

    // Write the collected data to a CSV file
    let file_path = "race_data.csv";
    let mut wtr = csv::Writer::from_path(file_path)?;

    for record in all_data {
        wtr.serialize(record)?;
    }

    wtr.flush()?;
    println!("Data successfully written to {}", file_path);

    Ok(())
}
