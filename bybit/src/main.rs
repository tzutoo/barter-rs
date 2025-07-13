use chrono::{DateTime, NaiveDate, Utc};
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BybitError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Date parsing failed: {0}")]
    DateParseError(String),
    #[error("API error: {msg}")]
    ApiError { msg: String },
}

#[derive(Debug, Serialize, Deserialize)]
struct BybitResponse {
    #[serde(rename = "retCode")]
    ret_code: i32,
    #[serde(rename = "retMsg")]
    ret_msg: String,
    result: Option<KlineResult>,
    time: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct KlineResult {
    symbol: String,
    category: String,
    list: Vec<Vec<String>>,
}

#[derive(Debug)]
struct Kline {
    start_time: u64,
    open_price: f64,
    high_price: f64,
    low_price: f64,
    close_price: f64,
    volume: f64,
    turnover: f64,
}

// Barter-compatible data structures
#[derive(Debug, Serialize, Deserialize)]
struct BarterCandle {
    pub close_time: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct BarterMarketEvent {
    pub time_exchange: DateTime<Utc>,
    pub time_received: DateTime<Utc>,
    pub exchange: String,
    pub instrument: usize,
    pub kind: BarterDataKind,
}

#[derive(Debug, Serialize, Deserialize)]
struct BarterDataKind {
    #[serde(rename = "Candle")]
    pub Candle: BarterCandle,
}

#[derive(Debug, Serialize, Deserialize)]
struct BarterMarketStreamEvent {
    #[serde(rename = "Item")]
    pub item: BarterMarketEventResult,
}

#[derive(Debug, Serialize, Deserialize)]
struct BarterMarketEventResult {
    #[serde(rename = "Ok")]
    pub ok: BarterMarketEvent,
}

impl Kline {
    fn from_vec(data: Vec<String>) -> Result<Self, BybitError> {
        if data.len() < 7 {
            return Err(BybitError::ApiError {
                msg: "Invalid kline data format".to_string(),
            });
        }

        Ok(Kline {
            start_time: data[0].parse().map_err(|_| BybitError::ApiError {
                msg: "Invalid start time".to_string(),
            })?,
            open_price: data[1].parse().map_err(|_| BybitError::ApiError {
                msg: "Invalid open price".to_string(),
            })?,
            high_price: data[2].parse().map_err(|_| BybitError::ApiError {
                msg: "Invalid high price".to_string(),
            })?,
            low_price: data[3].parse().map_err(|_| BybitError::ApiError {
                msg: "Invalid low price".to_string(),
            })?,
            close_price: data[4].parse().map_err(|_| BybitError::ApiError {
                msg: "Invalid close price".to_string(),
            })?,
            volume: data[5].parse().map_err(|_| BybitError::ApiError {
                msg: "Invalid volume".to_string(),
            })?,
            turnover: data[6].parse().map_err(|_| BybitError::ApiError {
                msg: "Invalid turnover".to_string(),
            })?,
        })
    }

    fn format_time(&self) -> String {
        let dt = DateTime::from_timestamp_millis(self.start_time as i64)
            .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());
        dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }

    fn to_barter_event(&self, instrument_index: usize, interval_minutes: u32, category: &str) -> BarterMarketStreamEvent {
        let start_time = DateTime::from_timestamp_millis(self.start_time as i64)
            .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());
        
        // Calculate close time by adding interval duration
        let close_time = start_time + chrono::Duration::minutes(interval_minutes as i64);
        let now = Utc::now();
        
        // Map category to exchange name
        let exchange_name = match category {
            "spot" => "bybit_spot",
            "linear" => "bybit_perpetuals_usd",
            "inverse" => "bybit_perpetuals_usd", // Using same as linear for now
            _ => "bybit_spot",
        };
        
        let candle = BarterCandle {
            close_time,
            open: self.open_price,
            high: self.high_price,
            low: self.low_price,
            close: self.close_price,
            volume: self.volume,
            trade_count: 0, // Bybit doesn't provide trade count in kline data
        };
        
        let market_event = BarterMarketEvent {
            time_exchange: start_time,
            time_received: now,
            exchange: exchange_name.to_string(),
            instrument: instrument_index,
            kind: BarterDataKind {
                Candle: candle,
            },
        };
        
        BarterMarketStreamEvent {
            item: BarterMarketEventResult {
                ok: market_event,
            },
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Symbol to fetch (e.g., BTCUSDT)
    #[arg(short, long, default_value = "BTCUSDT")]
    symbol: String,

    /// Kline interval in minutes (e.g., 15, 60, 240)
    #[arg(short, long, default_value = "15")]
    interval: String,

    /// Start date in YYYY/MM/DD format
    #[arg(long)]
    start_date: String,

    /// End date in YYYY/MM/DD format
    #[arg(long)]
    end_date: String,

    /// Category (spot, linear, inverse)
    #[arg(short, long, default_value = "linear")]
    category: String,

    /// Maximum number of records to fetch (program will automatically paginate)
    #[arg(short, long, default_value = "1000")]
    max_records: u32,

    /// Use testnet instead of mainnet
    #[arg(long)]
    testnet: bool,

    /// Output format: 'table' (default) or 'barter' (JSON format compatible with barter backtesting)
    #[arg(long, default_value = "table")]
    output_format: String,

    /// Instrument index for barter format (default: 0)
    #[arg(long, default_value = "0")]
    instrument_index: usize,
}

struct BybitClient {
    client: Client,
    base_url: String,
}

impl BybitClient {
    fn new(testnet: bool) -> Self {
        let base_url = if testnet {
            "https://api-testnet.bybit.com".to_string()
        } else {
            "https://api.bybit.com".to_string()
        };

        Self {
            client: Client::new(),
            base_url,
        }
    }

    async fn get_kline_single(
        &self,
        symbol: &str,
        interval: &str,
        start: u64,
        end: u64,
        category: &str,
        limit: u32,
    ) -> Result<Vec<Kline>, BybitError> {
        let url = format!("{}/v5/market/kline", self.base_url);
        
        let response = self
            .client
            .get(&url)
            .query(&[
                ("category", category),
                ("symbol", symbol),
                ("interval", interval),
                ("start", &start.to_string()),
                ("end", &end.to_string()),
                ("limit", &limit.to_string()),
            ])
            .send()
            .await?
            .json::<BybitResponse>()
            .await?;

        if response.ret_code != 0 {
            return Err(BybitError::ApiError {
                msg: response.ret_msg,
            });
        }

        let result = response.result.ok_or_else(|| BybitError::ApiError {
            msg: "No result data".to_string(),
        })?;

        let mut klines = Vec::new();
        for kline_data in result.list {
            klines.push(Kline::from_vec(kline_data)?);
        }

        Ok(klines)
    }

    async fn get_kline(
        &self,
        symbol: &str,
        interval: &str,
        start: u64,
        end: u64,
        category: &str,
        max_records: u32,
        output_format: &str,
    ) -> Result<Vec<Kline>, BybitError> {
        let mut all_klines: Vec<Kline> = Vec::new();
        let mut current_start = start;
        let chunk_limit = 1000u32; // Bybit's max limit is 1000
        
        // Calculate interval duration in milliseconds
        let interval_ms = self.parse_interval_to_ms(interval)?;
        
        while current_start < end && (all_klines.len() as u32) < max_records {
            // Calculate how many more records we need
            let remaining_records = max_records - (all_klines.len() as u32);
            let current_chunk_limit = std::cmp::min(chunk_limit, remaining_records);
            
            // Calculate the end time for this chunk
            let chunk_end = std::cmp::min(
                current_start + (current_chunk_limit as u64 * interval_ms),
                end
            );
            
            // Only show progress for table format
            if output_format != "barter" {
                println!("Fetching data from {} to {} (chunk size: {})...", 
                    DateTime::from_timestamp_millis(current_start as i64)
                        .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap())
                        .format("%Y-%m-%d %H:%M:%S"),
                    DateTime::from_timestamp_millis(chunk_end as i64)
                        .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap())
                        .format("%Y-%m-%d %H:%M:%S"),
                    current_chunk_limit
                );
            }
            
            let mut chunk_klines = self.get_kline_single(
                symbol,
                interval,
                current_start,
                chunk_end,
                category,
                current_chunk_limit,
            ).await?;
            
            if chunk_klines.is_empty() {
                if output_format != "barter" {
                    println!("No more data available.");
                }
                break;
            }
            
            // Sort by start_time to ensure proper ordering
            chunk_klines.sort_by_key(|k| k.start_time);
            
            // Remove duplicates if any (based on start_time)
            if !all_klines.is_empty() {
                let last_time = all_klines.last().unwrap().start_time;
                chunk_klines.retain(|k| k.start_time > last_time);
            }
            
            // Limit the chunk to not exceed max_records
            let space_left = max_records as usize - all_klines.len();
            if chunk_klines.len() > space_left {
                chunk_klines.truncate(space_left);
            }
            
            if output_format != "barter" {
                println!("Retrieved {} records in this chunk. Total so far: {}", 
                    chunk_klines.len(), 
                    all_klines.len() + chunk_klines.len()
                );
            }
            
            all_klines.extend(chunk_klines);
            
            // Check if we've reached the max_records limit
            if (all_klines.len() as u32) >= max_records {
                if output_format != "barter" {
                    println!("Reached maximum record limit of {}.", max_records);
                }
                break;
            }
            
            // Move to next chunk - start from the last kline's time + interval
            if let Some(last_kline) = all_klines.last() {
                current_start = last_kline.start_time + interval_ms;
            } else {
                break;
            }
            
            // Add a small delay to avoid rate limiting
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        // Final sort and deduplication
        all_klines.sort_by_key(|k| k.start_time);
        all_klines.dedup_by_key(|k| k.start_time);
        
        Ok(all_klines)
    }
    
    fn parse_interval_to_ms(&self, interval: &str) -> Result<u64, BybitError> {
        match interval {
            "1" => Ok(60_000),           // 1 minute
            "3" => Ok(180_000),          // 3 minutes
            "5" => Ok(300_000),          // 5 minutes
            "15" => Ok(900_000),         // 15 minutes
            "30" => Ok(1_800_000),       // 30 minutes
            "60" => Ok(3_600_000),       // 1 hour
            "120" => Ok(7_200_000),      // 2 hours
            "240" => Ok(14_400_000),     // 4 hours
            "360" => Ok(21_600_000),     // 6 hours
            "720" => Ok(43_200_000),     // 12 hours
            "D" => Ok(86_400_000),       // 1 day
            "W" => Ok(604_800_000),      // 1 week
            "M" => Ok(2_592_000_000),    // 30 days (approximate)
            _ => Err(BybitError::ApiError {
                msg: format!("Unsupported interval: {}", interval),
            }),
        }
    }
}

fn parse_date(date_str: &str) -> Result<u64, BybitError> {
    let date = NaiveDate::parse_from_str(date_str, "%Y/%m/%d")
        .map_err(|e| BybitError::DateParseError(format!("Invalid date format '{}': {}", date_str, e)))?;
    
    let datetime = date.and_hms_opt(0, 0, 0)
        .ok_or_else(|| BybitError::DateParseError("Invalid time".to_string()))?;
    
    let utc_datetime = DateTime::<Utc>::from_naive_utc_and_offset(datetime, Utc);
    Ok(utc_datetime.timestamp_millis() as u64)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Only show info for table format
    if args.output_format != "barter" {
        println!("Fetching Bybit Kline Data");
        println!("Symbol: {}", args.symbol);
        println!("Interval: {} minutes", args.interval);
        println!("Category: {}", args.category);
        println!("Start Date: {}", args.start_date);
        println!("End Date: {}", args.end_date);
        println!("Max Records: {}", args.max_records);
        println!("Using: {}", if args.testnet { "Testnet" } else { "Mainnet" });
        println!();
        println!("Note: Program will automatically paginate to fetch all data within the date range.");
        println!();
    }

    let start_timestamp = parse_date(&args.start_date)?;
    let end_timestamp = parse_date(&args.end_date)?;

    if start_timestamp >= end_timestamp {
        return Err(BybitError::DateParseError(
            "Start date must be before end date".to_string(),
        ).into());
    }

    let client = BybitClient::new(args.testnet);
    
    if args.output_format != "barter" {
        println!("Fetching kline data...");
    }
    let klines = client
        .get_kline(
            &args.symbol,
            &args.interval,
            start_timestamp,
            end_timestamp,
            &args.category,
            args.max_records,
            &args.output_format,
        )
        .await?;

    match args.output_format.as_str() {
        "barter" => {
            // Parse interval to get minutes for close_time calculation
            let interval_minutes: u32 = args.interval.parse().unwrap_or(15);
            
            // Output in barter-compatible JSON format
            for kline in &klines {
                let barter_event = kline.to_barter_event(args.instrument_index, interval_minutes, &args.category);
                println!("{}", serde_json::to_string(&barter_event)?);
            }
        },
        "table" | _ => {
            // Default table format
            println!("\nReceived {} kline records:\n", klines.len());
            println!(
                "{:<20} {:<12} {:<12} {:<12} {:<12} {:<15} {:<15}",
                "Time", "Open", "High", "Low", "Close", "Volume", "Turnover"
            );
            println!("{}", "-".repeat(110));

            for kline in &klines {
                println!(
                    "{:<20} {:<12.4} {:<12.4} {:<12.4} {:<12.4} {:<15.4} {:<15.4}",
                    kline.format_time(),
                    kline.open_price,
                    kline.high_price,
                    kline.low_price,
                    kline.close_price,
                    kline.volume,
                    kline.turnover
                );
            }

            println!("\nTotal records: {}", klines.len());
        }
    }
    Ok(())
}