use anyhow::Result;
use tokio::fs;

/// Loads all symbols from the CSV file into memory.
/// Returns a vector of symbol strings (first column of the CSV).
pub async fn load_symbols(path: String) -> Result<Vec<String>> {
    let content = fs::read_to_string(path).await?;
    let mut symbols = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let line = line.trim();

        // Skip empty lines and header row
        if line.is_empty() || (i == 0 && line.to_ascii_lowercase().starts_with("symbol")) {
            continue;
        }

        // Extract first column (symbol) and push if not empty
        if let Some(symbol) = line
            .split(',')
            .next()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            symbols.push(symbol.to_string());
        }
    }

    Ok(symbols)
}
