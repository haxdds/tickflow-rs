use tickflow::messages::{AlpacaMessage, Bar};

// Part 1: Parsing different message types

#[test]
fn test_parse_success_message() {
    let json = r#"{"T":"success","msg":"authenticated"}"#;
    let msg = serde_json::from_str::<AlpacaMessage>(json).unwrap();
    
    match msg {
        AlpacaMessage::Success { msg } => {
            assert_eq!(msg, "authenticated");
        }
        _ => panic!("Expected Success message"),
    }
}

#[test]
fn test_parse_error_message() {
    let json = r#"{"T":"error","code":401,"msg":"invalid credentials"}"#;
    let msg = serde_json::from_str::<AlpacaMessage>(json).unwrap();
    
    match msg {
        AlpacaMessage::Error { code, msg } => {
            assert_eq!(code, 401);
            assert_eq!(msg, "invalid credentials");
        }
        _ => panic!("Expected Error message"),
    }
}

#[test]
fn test_parse_subscription_message() {
    let json = r#"{"T":"subscription","trades":["AAPL","TSLA"],"quotes":[],"bars":["AAPL"],"updated_bars":[],"daily_bars":[],"statuses":[],"lulds":[],"corrections":[],"cancel_errors":[]}"#;
    let msg = serde_json::from_str::<AlpacaMessage>(json).unwrap();
    
    match msg {
        AlpacaMessage::Subscription { trades, quotes, bars, .. } => {
            assert_eq!(trades, vec!["AAPL", "TSLA"]);
            assert_eq!(quotes, Vec::<String>::new());
            assert_eq!(bars, vec!["AAPL"]);
        }
        _ => panic!("Expected Subscription message"),
    }
}

#[test]
fn test_parse_bar_message() {
    let json = r#"{"T":"b","S":"AAPL","o":150.0,"h":152.5,"l":149.5,"c":151.0,"v":1000000,"t":"2024-01-01T10:00:00Z","n":1500,"vw":150.75}"#;
    let msg = serde_json::from_str::<AlpacaMessage>(json).unwrap();
    
    match msg {
        AlpacaMessage::Bar(bar) => {
            assert_eq!(bar.symbol, "AAPL");
            assert_eq!(bar.open, 150.0);
            assert_eq!(bar.high, 152.5);
            assert_eq!(bar.low, 149.5);
            assert_eq!(bar.close, 151.0);
            assert_eq!(bar.volume, 1000000.0);
            assert_eq!(bar.timestamp, "2024-01-01T10:00:00Z");
            assert_eq!(bar.trade_count, 1500);
            assert_eq!(bar.vwap, 150.75);
            
            // Test Bar methods
            assert_eq!(bar.price_change(), 1.0);
            assert!((bar.price_change_percent() - 0.6666666666666666).abs() < 0.0001);
        }
        _ => panic!("Expected Bar message"),
    }
}

#[test]
fn test_parse_quote_message() {
    let json = r#"{"T":"q","S":"TSLA","bx":"NASDAQ","bp":250.10,"bs":100,"ax":"NASDAQ","ap":250.15,"as":200,"c":[],"z":"C","t":"2024-01-01T10:00:01Z"}"#;
    let msg = serde_json::from_str::<AlpacaMessage>(json).unwrap();
    
    match msg {
        AlpacaMessage::Quote(quote) => {
            assert_eq!(quote.symbol, "TSLA");
            assert_eq!(quote.bid_exchange, "NASDAQ");
            assert_eq!(quote.bid_price, 250.10);
            assert_eq!(quote.bid_size, 100.0);
            assert_eq!(quote.ask_exchange, "NASDAQ");
            assert_eq!(quote.ask_price, 250.15);
            assert_eq!(quote.ask_size, 200.0);
            assert_eq!(quote.conditions, Vec::<String>::new());
            assert_eq!(quote.tape, "C");
            assert_eq!(quote.timestamp, "2024-01-01T10:00:01Z");
            
            // Test Quote methods
            // 0.01_f64 just means the floating-point literal 0.01 as f64 ("type suffix").
            // You could use 0.01 without the _f64 because the types match in this context.
            assert!((quote.spread() - 0.05).abs() < 0.00001);
            // assert!((quote.spread_bps() - 1.9976).abs() < 0.0001);
        }
        _ => panic!("Expected Quote message"),
    }
}

#[test]
fn test_parse_trade_message() {
    let json = r#"{"T":"t","S":"MSFT","i":12345,"x":"NASDAQ","p":380.50,"s":50,"c":[],"z":"C","t":"2024-01-01T10:00:02Z"}"#;
    let msg = serde_json::from_str::<AlpacaMessage>(json).unwrap();
    
    match msg {
        AlpacaMessage::Trade(trade) => {
            assert_eq!(trade.symbol, "MSFT");
            assert_eq!(trade.id, 12345);
            assert_eq!(trade.exchange, "NASDAQ");
            assert_eq!(trade.price, 380.50);
            assert_eq!(trade.size, 50.0);
            assert_eq!(trade.conditions, Vec::<String>::new());
            assert_eq!(trade.tape, "C");
            assert_eq!(trade.timestamp, "2024-01-01T10:00:02Z");
        }
        _ => panic!("Expected Trade message"),
    }
}

// Part 2: Debug trait demonstration

#[test]
fn test_bar_debug_trait() {
    let json = r#"{"T":"b","S":"AAPL","o":150.0,"h":152.5,"l":149.5,"c":151.0,"v":1000000,"t":"2024-01-01T10:00:00Z","n":1500,"vw":150.75}"#;
    let bar: Bar = serde_json::from_str(json).unwrap();
    
    // Debug trait should work (compiles and runs without panicking)
    let debug_output = format!("{:?}", bar);
    assert!(debug_output.contains("AAPL"));
    assert!(debug_output.contains("150.0"));
    
    // Pretty debug should also work
    let pretty_debug = format!("{:#?}", bar);
    assert!(pretty_debug.contains("AAPL"));
}

// Part 3: Error handling

#[test]
fn test_parse_invalid_message_type() {
    let invalid_json = r#"{"T":"invalid","unknown":"field"}"#;
    
    let result = serde_json::from_str::<AlpacaMessage>(invalid_json);
    assert!(result.is_err());
}

#[test]
fn test_parse_malformed_json() {
    let malformed_json = r#"{"T":"b","S":"AAPL" MISSING BRACE"#;
    
    let result = serde_json::from_str::<AlpacaMessage>(malformed_json);
    assert!(result.is_err());
}
