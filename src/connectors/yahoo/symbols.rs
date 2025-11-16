use tokio::fs;
use anyhow::Result;

const SYMBOLS_CSV_PATH: &str = "/Users/rahul/Development/tickflow-rs/symbols.csv";

/// Loads all symbols from the CSV file into memory.
/// Returns a vector of symbol strings (first column of the CSV).
pub async fn load_symbols() -> Result<Vec<String>> {
    let content = fs::read_to_string(SYMBOLS_CSV_PATH).await?;
    let mut symbols = Vec::new();
    
    for (i, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        // Skip header row
        if i == 0 && line.to_ascii_lowercase().starts_with("symbol") {
            continue;
        }
        
        // Extract first column (symbol)
        if let Some(symbol) = line.split(',').next() {
            let symbol = symbol.trim();
            if !symbol.is_empty() {
                symbols.push(symbol.to_string());
            }
        }
    }
    
    Ok(symbols)
}