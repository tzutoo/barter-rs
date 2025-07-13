# Bybit Kline Data Fetcher

A Rust command-line tool to fetch historical kline (candlestick) data from Bybit's API.

## Features

- **Automatic Pagination**: Handles large date ranges by automatically chunking requests to stay within Bybit's 1000-record API limit
- **Smart Rate Limiting**: Built-in delays to prevent API rate limit violations
- **Progress Tracking**: Real-time progress updates during data fetching
- **Duplicate Prevention**: Automatically removes duplicate records that may occur at chunk boundaries
- **Flexible Intervals**: Supports all Bybit kline intervals from 1 minute to 1 month
- **Multi-Category Support**: Works with spot, linear, and inverse perpetual markets
- **Testnet Support**: Option to use Bybit's testnet for testing purposes
- **Barter Integration**: Output data in JSON format compatible with the barter backtesting system
- **Dual Output Formats**: Choose between human-readable table format or machine-readable JSON

## Usage

### Basic Usage (Table Format)

```bash
cargo run -- --start-date 2024/01/01 --end-date 2024/01/02
```

### Barter-Compatible JSON Output

```bash
./bybit-kline --symbol BTCUSDT --interval 15 --category spot --start-date 2024/01/01 --end-date 2024/01/02 --output-format barter --instrument-index 1
```

The barter output format produces JSON lines compatible with the [barter-rs](https://github.com/barter-rs/barter-rs) backtesting framework. Each line contains a `MarketStreamEvent` with candle data:

```json
{"Item":{"Ok":{"time_exchange":"2024-01-01T00:15:00Z","time_received":"2025-07-12T14:23:47.949648597Z","exchange":"bybit_spot","instrument":1,"kind":{"Candle":{"close_time":"2024-01-01T00:30:00Z","open":42486.39,"high":42552.0,"low":42413.81,"close":42421.0,"volume":49.749055,"trade_count":0}}}}}
```

**Exchange Mapping:**
- `spot` category → `bybit_spot`
- `linear` category → `bybit_perpetuals_usd`
- `inverse` category → `bybit_perpetuals_usd`

### Custom Interval (60 minutes)

```bash
cargo run -- --interval 60 --start-date 2024/01/01 --end-date 2024/01/02
```

### Different Symbol and Category

```bash
cargo run -- --symbol ETHUSDT --category spot --start-date 2024/01/01 --end-date 2024/01/02
```

### Using Testnet

```bash
cargo run -- --testnet --start-date 2024/01/01 --end-date 2024/01/02
```

### Large Date Range with Pagination

```bash
cargo run -- --testnet --start-date 2024/01/01 --end-date 2024/01/10 --max-records 5000
```

This will automatically paginate through multiple API calls to fetch all data within the 10-day range.

## Command Line Options

- `--symbol, -s`: Symbol to fetch (default: BTCUSDT)
- `--interval, -i`: Kline interval in minutes (default: 15)
- `--start-date`: Start date in YYYY/MM/DD format (required)
- `--end-date`: End date in YYYY/MM/DD format (required)
- `--category, -c`: Product category - spot, linear, inverse (default: linear)
- `--max-records, -m`: Maximum number of records to fetch (default: 1000)
- `--output-format`: Output format - "table" (default) or "barter" for JSON compatible with barter backtesting system
- `--instrument-index`: Instrument index for barter format (required when using barter output)
- `--testnet`: Use testnet instead of mainnet

**Note**: The program automatically handles pagination when the date range requires more than 1000 records per API call. It will make multiple requests as needed to fetch all data within the specified date range, up to the `max-records` limit.

## Supported Intervals

According to Bybit API documentation, supported intervals are:
- `1`, `3`, `5`, `15`, `30`, `60`, `120`, `240`, `360`, `720` (minutes)
- `D` (daily), `W` (weekly), `M` (monthly)

## Example Output

```
Fetching Bybit Kline Data
Symbol: BTCUSDT
Interval: 15 minutes
Category: linear
Start Date: 2024/01/01
End Date: 2024/01/02
Limit: 200
Using: Mainnet

Fetching kline data...

Received 96 kline records:

Time                 Open         High         Low          Close        Volume          Turnover       
--------------------------------------------------------------------------------------------------------------
2024-01-01 00:00:00  42250.0000   42280.0000   42200.0000   42260.0000   125.4500        5304250.0000   
2024-01-01 00:15:00  42260.0000   42300.0000   42240.0000   42285.0000   98.7500         4175625.0000   
...

Total records: 96
```

## Error Handling

The program handles various error cases:
- Invalid date formats
- Network connectivity issues
- API errors from Bybit
- Invalid response data

## Dependencies

- `tokio`: Async runtime
- `reqwest`: HTTP client
- `serde`: JSON serialization/deserialization
- `chrono`: Date/time handling
- `clap`: Command-line argument parsing
- `thiserror`: Error handling