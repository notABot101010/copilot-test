// Integration tests for tui-browser

#[test]
fn test_tab_creation() {
    // Test Tab creation
    let tab = tui_browser::Tab::new();
    assert_eq!(tab.title, "New Tab");
    assert!(tab.url.is_empty());
    assert!(!tab.loading);
}

#[test]
fn test_bookmark_creation() {
    // Test Bookmark creation
    let bookmark = tui_browser::Bookmark::new(
        "Test Site".to_string(),
        "https://example.com".to_string(),
    );
    assert_eq!(bookmark.title, "Test Site");
    assert_eq!(bookmark.url, "https://example.com");
}

#[test]
fn test_history_entry_creation() {
    // Test HistoryEntry creation
    let entry = tui_browser::HistoryEntry::new(
        "https://example.com".to_string(),
        "Example".to_string(),
    );
    assert_eq!(entry.url, "https://example.com");
    assert_eq!(entry.title, "Example");
}

#[test]
fn test_navigation_history() {
    // Test NavigationHistory
    let mut history = tui_browser::NavigationHistory::new();
    
    let entry1 = tui_browser::HistoryEntry::new(
        "https://example.com".to_string(),
        "Example".to_string(),
    );
    history.add_entry(entry1);
    
    let entry2 = tui_browser::HistoryEntry::new(
        "https://rust-lang.org".to_string(),
        "Rust".to_string(),
    );
    history.add_entry(entry2);
    
    // Test back navigation
    let back = history.go_back();
    assert!(back.is_some());
    assert_eq!(back.unwrap().url, "https://example.com");
    
    // Test forward navigation
    let forward = history.go_forward();
    assert!(forward.is_some());
    assert_eq!(forward.unwrap().url, "https://rust-lang.org");
}

#[test]
fn test_http_client_creation() {
    let client = tui_browser::HttpClient::new();
    assert!(client.is_ok());
}

#[test]
fn test_html_to_text_rendering() {
    let client = tui_browser::HttpClient::new().unwrap();
    
    // Test HTML to text rendering
    let html = "<html><head><title>Test</title></head><body><h1>Hello</h1><p>World</p></body></html>";
    let text = client.render_html_to_text(html);
    assert!(text.contains("Hello"));
    assert!(text.contains("World"));
}

#[test]
fn test_link_creation() {
    // Test Link creation
    let link = tui_browser::Link {
        text: "Example Link".to_string(),
        url: "https://example.com".to_string(),
        line_index: 5,
    };
    assert_eq!(link.text, "Example Link");
    assert_eq!(link.url, "https://example.com");
    assert_eq!(link.line_index, 5);
}
