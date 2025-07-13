#!/bin/bash

# Bybit Kline Data Fetcher - Usage Examples
# Build the project first: cargo build --release

echo "=== Bybit Kline Data Fetcher Examples ==="
echo "Note: Make sure to build the project first with 'cargo build --release'"
echo "⚠️  When copying commands below, copy only the command part (after 'Command: '), not the quotes!"
echo

# Example 1: Basic usage with 15-minute intervals (default)
echo "Example 1: Fetching 15-minute BTCUSDT klines for one day"
echo "Command: ./target/release/bybit-kline --testnet --symbol BTCUSDT --interval 15 --category linear --start-date 2024/01/01 --end-date 2024/01/02 --max-records 100"
echo

# Example 2: Custom interval (60 minutes)
echo "Example 2: Fetching 60-minute BTCUSDT klines"
echo "Command: ./target/release/bybit-kline --testnet --symbol BTCUSDT --interval 60 --category linear --start-date 2024/01/01 --end-date 2024/01/02 --max-records 50"
echo

# Example 3: Different symbol (ETHUSDT)
echo "Example 3: Fetching ETHUSDT klines"
echo "Command: ./target/release/bybit-kline --testnet --symbol ETHUSDT --interval 15 --category linear --start-date 2024/01/01 --end-date 2024/01/02 --max-records 100"
echo

# Example 4: Spot category
echo "Example 4: Fetching spot market data"
echo "Command: ./target/release/bybit-kline --testnet --symbol BTCUSDT --interval 15 --category spot --start-date 2024/01/01 --end-date 2024/01/02 --max-records 100"
echo

# Example 5: Large date range with automatic pagination
echo "Example 5: Fetching data for a week with automatic pagination"
echo "Command: ./target/release/bybit-kline --testnet --symbol BTCUSDT --interval 60 --category linear --start-date 2024/01/01 --end-date 2024/01/08 --max-records 2000"
echo

# Example 6: Barter-compatible JSON output for backtesting
echo "Example 6: Generating barter-compatible JSON output"
echo "Command: ./target/release/bybit-kline --testnet --symbol BTCUSDT --interval 15 --category spot --start-date 2024/01/01 --end-date 2024/01/02 --max-records 100 --output-format barter --instrument-index 1"
echo

# Example 7: Save barter output to file for backtesting
echo "Example 7: Saving barter output to file"
echo "Command: ./target/release/bybit-kline --testnet --symbol ETHUSDT --interval 60 --category spot --start-date 2024/01/01 --end-date 2024/01/03 --max-records 500 --output-format barter --instrument-index 2 > eth_market_data.json"
echo

# Example 8: Daily intervals for a month
echo "Example 8: Fetching daily klines for a month"
echo "Command: ./target/release/bybit-kline --testnet --symbol BTCUSDT --interval D --category linear --start-date 2024/01/01 --end-date 2024/02/01 --max-records 50"
echo

# Example 9: Production mainnet usage
echo "Example 9: Using mainnet (remove --testnet flag)"
echo "Command: ./target/release/bybit-kline --symbol BTCUSDT --interval 15 --category linear --start-date 2024/01/01 --end-date 2024/01/02 --max-records 100"
echo

echo "=== Executable Examples ==="
echo "Uncomment and run any of the following commands:"
echo

# Uncomment to run examples:
# echo "Running Example 1: Basic BTCUSDT fetch..."
# ./target/release/bybit-kline --testnet --symbol BTCUSDT --interval 15 --category linear --start-date 2024/01/01 --end-date 2024/01/02 --max-records 10

# echo "Running Example 4: Spot market data..."
# ./target/release/bybit-kline --testnet --symbol BTCUSDT --interval 15 --category spot --start-date 2024/01/01 --end-date 2024/01/02 --max-records 5

# echo "Running Example 6: Barter JSON output..."
# ./target/release/bybit-kline --testnet --symbol BTCUSDT --interval 15 --category spot --start-date 2024/01/01 --end-date 2024/01/02 --max-records 3 --output-format barter --instrument-index 1

echo "=== Documentation ==="
echo "📋 Required Parameters:"
echo "  --symbol: Trading pair (e.g., BTCUSDT, ETHUSDT)"
echo "  --interval: Time interval (1, 3, 5, 15, 30, 60, 120, 240, 360, 720, D, W, M)"
echo "  --category: Market type (spot, linear, inverse)"
echo "  --start-date: Start date (YYYY/MM/DD format)"
echo "  --end-date: End date (YYYY/MM/DD format)"
echo
echo "⚙️  Optional Parameters:"
echo "  --max-records: Maximum records to fetch (default: 1000)"
echo "  --output-format: Output format (table, barter)"
echo "  --instrument-index: Required for barter format"
echo "  --testnet: Use testnet (recommended for testing)"
echo
echo "🚀 Features:"
echo "  • Automatic Pagination: Handles large date ranges automatically"
echo "  • Rate Limiting: Built-in delays to prevent API violations"
echo "  • Progress Tracking: Real-time updates during data fetching"
echo "  • Duplicate Prevention: Removes overlapping records"
echo "  • Barter Integration: JSON output for backtesting framework"
echo
echo "💡 Tips:"
echo "  • Always test with --testnet first"
echo "  • Use smaller --max-records for initial testing"
echo "  • Save barter output to files: command > output.json"
echo "  • Check Bybit API limits and your account permissions"