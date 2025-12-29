mod http_client;
mod models;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use http_client::HttpClient;
use models::{HistoryEntry, ImageInfo, Link, NavigationHistory, Tab};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use std::io;
use ui::{ContentArea, HelpDialog, StatusBar, TabBar, UrlBar};

const SCROLL_STEP: usize = 1;
const PAGE_SCROLL_STEP: usize = 10;
const DEFAULT_VIEWPORT_HEIGHT: usize = 10;
const BORDER_HEIGHT: u16 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusPanel {
    TabBar,
    UrlBar,
    Content,
}

struct App {
    tabs: Vec<Tab>,
    current_tab_index: usize,
    focused_panel: FocusPanel,
    url_input: String,
    url_cursor_position: usize,
    should_quit: bool,
    http_client: HttpClient,
    history: NavigationHistory,
    show_help: bool,
    help_scroll_offset: usize,
    status_message: String,
    current_links: Vec<Link>,
    current_images: Vec<ImageInfo>,
    link_number_input: String,
    content_viewport_height: usize,
    content_width_percent: f32,
    search_mode: bool,
    search_query: String,
    search_results: Vec<usize>,
    current_search_result: usize,
    image_picker: Option<Picker>,
}

impl App {
    fn new() -> Result<Self> {
        let mut tabs = Vec::new();
        tabs.push(Tab::new());
        
        Ok(Self {
            tabs,
            current_tab_index: 0,
            focused_panel: FocusPanel::UrlBar,
            url_input: String::new(),
            url_cursor_position: 0,
            should_quit: false,
            http_client: HttpClient::new()?,
            history: NavigationHistory::new(),
            show_help: false,
            help_scroll_offset: 0,
            status_message: "Welcome to TUI Browser! Press Ctrl+H for help.".to_string(),
            current_links: Vec::new(),
            current_images: Vec::new(),
            link_number_input: String::new(),
            content_viewport_height: DEFAULT_VIEWPORT_HEIGHT,
            content_width_percent: 0.6,
            search_mode: false,
            search_query: String::new(),
            search_results: Vec::new(),
            current_search_result: 0,
            image_picker: None,
        })
    }

    fn current_tab(&self) -> &Tab {
        &self.tabs[self.current_tab_index]
    }

    fn current_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.current_tab_index]
    }

    fn open_new_tab(&mut self) {
        self.tabs.push(Tab::new());
        self.current_tab_index = self.tabs.len() - 1;
        self.url_input.clear();
        self.url_cursor_position = 0;
        self.focused_panel = FocusPanel::UrlBar;
        self.status_message = format!("Opened new tab ({})", self.tabs.len());
    }

    fn close_current_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.current_tab_index);
            if self.current_tab_index >= self.tabs.len() {
                self.current_tab_index = self.tabs.len() - 1;
            }
            self.status_message = format!("Closed tab. {} tab(s) remaining", self.tabs.len());
        } else {
            self.status_message = "Cannot close the last tab".to_string();
        }
    }

    fn next_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.current_tab_index = (self.current_tab_index + 1) % self.tabs.len();
            self.update_url_bar_from_current_tab();
        }
    }

    fn previous_tab(&mut self) {
        if self.tabs.len() > 1 {
            if self.current_tab_index == 0 {
                self.current_tab_index = self.tabs.len() - 1;
            } else {
                self.current_tab_index -= 1;
            }
            self.update_url_bar_from_current_tab();
        }
    }

    fn update_url_bar_from_current_tab(&mut self) {
        self.url_input = self.current_tab().url.clone();
        self.url_cursor_position = self.url_input.len();
    }

    fn cycle_focus(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusPanel::TabBar => FocusPanel::UrlBar,
            FocusPanel::UrlBar => FocusPanel::Content,
            FocusPanel::Content => FocusPanel::TabBar,
        };
    }

    fn navigate_to_url(&mut self) {
        if self.url_input.trim().is_empty() {
            self.status_message = "Please enter a URL".to_string();
            return;
        }

        let url = self.url_input.trim().to_string();
        
        // Add http:// if no protocol specified
        let url = if !url.starts_with("http://") && !url.starts_with("https://") {
            format!("https://{}", url)
        } else {
            url
        };

        self.status_message = format!("Loading {}...", url);
        
        let tab = self.current_tab_mut();
        tab.url = url.clone();
        tab.loading = true;
        tab.title = "Loading...".to_string();

        // Fetch the page
        match self.http_client.fetch_page(&url) {
            Ok(html) => {
                let text_content = self.http_client.render_html_to_text(&html);
                
                // Extract title from HTML (simple approach)
                let title = Self::extract_title(&html).unwrap_or_else(|| url.clone());
                
                // Extract links from HTML
                let links = Self::extract_links(&html, &text_content);
                
                // Extract images from HTML
                let mut images = Self::extract_images(&html, &url);
                
                // Download and decode images (limit to first 5 for performance)
                let max_images = 5;
                let total_images = images.len();
                for (idx, image_info) in images.iter_mut().take(max_images).enumerate() {
                    if let Ok(img_data) = self.http_client.fetch_image(&image_info.url) {
                        image_info.data = Some(img_data);
                        self.status_message = format!("Loading images... ({}/{})", idx + 1, total_images.min(max_images));
                    }
                }
                
                // Insert image placeholders in text
                let text_with_images = Self::insert_image_placeholders(&images, &text_content);
                
                let tab = self.current_tab_mut();
                tab.content = text_with_images;
                tab.loading = false;
                tab.scroll_offset = 0;
                tab.title = title.clone();
                
                // Update current links and images
                self.current_links = links;
                self.current_images = images;
                
                // Add to history
                let entry = HistoryEntry::new(url.clone(), title.clone());
                self.history.add_entry(entry);
                
                self.status_message = format!("Loaded: {} ({} images)", title, self.current_images.len());
            }
            Err(err) => {
                let tab = self.current_tab_mut();
                tab.loading = false;
                tab.content = format!("Error loading page:\n\n{}", err);
                tab.title = "Error".to_string();
                self.current_links.clear();
                self.current_images.clear();
                self.status_message = format!("Error: {}", err);
            }
        }
    }

    fn extract_title(html: &str) -> Option<String> {
        // Simple title extraction
        let lower = html.to_lowercase();
        if let Some(start) = lower.find("<title>") {
            if let Some(end) = lower[start..].find("</title>") {
                let title_start = start + 7;
                let title_end = start + end;
                return Some(html[title_start..title_end].trim().to_string());
            }
        }
        None
    }

    fn extract_images(html: &str, base_url: &str) -> Vec<ImageInfo> {
        let lower = html.to_lowercase();
        let mut images = Vec::new();
        
        // Find all <img> tags
        let mut pos = 0;
        while let Some(start_pos) = lower[pos..].find("<img") {
            let abs_start = pos + start_pos;
            if let Some(end) = lower[abs_start..].find('>') {
                let abs_end = abs_start + end;
                let img_tag = &html[abs_start..abs_end + 1];
                
                // Extract src attribute
                if let Some(src) = Self::extract_src_from_img(img_tag) {
                    // Convert relative URLs to absolute
                    let absolute_url = if src.starts_with("http://") || src.starts_with("https://") {
                        src
                    } else if src.starts_with("//") {
                        format!("https:{}", src)
                    } else if src.starts_with('/') {
                        // Absolute path
                        if let Ok(parsed) = url::Url::parse(base_url) {
                            if let Some(host) = parsed.host_str() {
                                let scheme = parsed.scheme();
                                format!("{}://{}{}", scheme, host, src)
                            } else {
                                src
                            }
                        } else {
                            src
                        }
                    } else {
                        // Relative path
                        if let Ok(parsed) = url::Url::parse(base_url) {
                            if let Ok(joined) = parsed.join(&src) {
                                joined.to_string()
                            } else {
                                src
                            }
                        } else {
                            src
                        }
                    };
                    
                    // Extract alt text if available
                    let alt = Self::extract_alt_from_img(img_tag).unwrap_or_else(|| "Image".to_string());
                    images.push(ImageInfo::new(absolute_url, alt, 0));
                }
                
                pos = abs_end + 1;
            } else {
                break;
            }
        }
        
        images
    }

    fn insert_image_placeholders(images: &[ImageInfo], text_content: &str) -> String {
        // If no images found, return original text
        if images.is_empty() {
            return text_content.to_string();
        }
        
        // Insert image placeholders at the beginning of the content
        let mut result = String::new();
        result.push_str("═══ Images on this Page ═══\n");
        for (idx, image) in images.iter().enumerate() {
            let status = if image.data.is_some() { "✓" } else { "✗" };
            result.push_str(&format!("{} [IMG {}] {}\n", status, idx + 1, image.alt));
            
            // Add placeholder lines for the image to be rendered
            if image.data.is_some() {
                result.push_str(&format!("[IMAGE_PLACEHOLDER_{}]\n", idx + 1));
            }
        }
        result.push_str("═══════════════════════════\n\n");
        result.push_str(text_content);
        
        result
    }

    fn extract_src_from_img(img_tag: &str) -> Option<String> {
        let src_pattern_double = "src=\"";
        let src_pattern_single = "src='";
        
        let (src_start, quote_char) = if let Some(pos) = img_tag.to_lowercase().find(src_pattern_double) {
            (Some(pos + src_pattern_double.len()), '"')
        } else if let Some(pos) = img_tag.to_lowercase().find(src_pattern_single) {
            (Some(pos + src_pattern_single.len()), '\'')
        } else {
            return None;
        };
        
        if let Some(src_value_start) = src_start {
            if let Some(src_end) = img_tag[src_value_start..].find(quote_char) {
                return Some(img_tag[src_value_start..src_value_start + src_end].to_string());
            }
        }
        None
    }

    fn extract_alt_from_img(img_tag: &str) -> Option<String> {
        let alt_pattern_double = "alt=\"";
        let alt_pattern_single = "alt='";
        
        let (alt_start, quote_char) = if let Some(pos) = img_tag.to_lowercase().find(alt_pattern_double) {
            (Some(pos + alt_pattern_double.len()), '"')
        } else if let Some(pos) = img_tag.to_lowercase().find(alt_pattern_single) {
            (Some(pos + alt_pattern_single.len()), '\'')
        } else {
            return None;
        };
        
        if let Some(alt_value_start) = alt_start {
            if let Some(alt_end) = img_tag[alt_value_start..].find(quote_char) {
                return Some(img_tag[alt_value_start..alt_value_start + alt_end].to_string());
            }
        }
        None
    }

    fn extract_href_from_link(link_html: &str) -> Option<String> {
        let href_pattern_double = "href=\"";
        let href_pattern_single = "href='";
        
        let (href_start, quote_char) = if let Some(pos) = link_html.to_lowercase().find(href_pattern_double) {
            (Some(pos + href_pattern_double.len()), '"')
        } else if let Some(pos) = link_html.to_lowercase().find(href_pattern_single) {
            (Some(pos + href_pattern_single.len()), '\'')
        } else {
            (None, '"')
        };
        
        if let Some(href_value_start) = href_start {
            if let Some(href_end) = link_html[href_value_start..].find(quote_char) {
                return Some(link_html[href_value_start..href_value_start + href_end].to_string());
            }
        }
        None
    }

    fn extract_text_from_link(link_html: &str) -> Option<String> {
        if let Some(text_start) = link_html.find('>') {
            let text_end = link_html.len() - 4; // remove </a>
            let raw_text = &link_html[text_start + 1..text_end];
            // Simple HTML tag stripping
            let mut text = String::new();
            let mut in_tag = false;
            for ch in raw_text.chars() {
                if ch == '<' {
                    in_tag = true;
                } else if ch == '>' {
                    in_tag = false;
                } else if !in_tag {
                    text.push(ch);
                }
            }
            let text = text.trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }
        None
    }

    fn find_line_index_for_text(text: &str, content_lines: &[&str]) -> usize {
        let text_words: Vec<String> = text.split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();
        
        let mut line_index = 0;
        let mut best_match = 0;
        
        for (idx, line) in content_lines.iter().enumerate() {
            // Strategy 1: Exact match
            if line.contains(text) {
                return idx;
            }
            
            // Strategy 2: Word-based matching with pre-computed lowercase words
            let line_lower = line.to_lowercase();
            let matches = text_words.iter()
                .filter(|word| line_lower.contains(word.as_str()))
                .count();
            
            if matches > best_match {
                best_match = matches;
                line_index = idx;
            }
        }
        
        line_index
    }

    fn extract_links(html: &str, content: &str) -> Vec<Link> {
        let mut links = Vec::new();
        let lower = html.to_lowercase();
        let content_lines: Vec<&str> = content.lines().collect();
        
        let mut pos = 0;
        while let Some(start_pos) = lower[pos..].find("<a") {
            let abs_start = pos + start_pos;
            // Check if it's actually an anchor tag (followed by space, >, newline, or tab)
            let next_char_pos = abs_start + 2;
            if next_char_pos < html.len() {
                let html_bytes = html.as_bytes();
                if next_char_pos < html_bytes.len() {
                    let next_byte = html_bytes[next_char_pos];
                    if !matches!(next_byte, b' ' | b'>' | b'\n' | b'\t') {
                        pos = abs_start + 2;
                        continue;
                    }
                }
            }
            
            if let Some(end) = lower[abs_start..].find("</a>") {
                let abs_end = abs_start + end;
                let link_html = &html[abs_start..abs_end + 4];
                
                if let Some(url) = Self::extract_href_from_link(link_html) {
                    if let Some(text) = Self::extract_text_from_link(link_html) {
                        let line_index = Self::find_line_index_for_text(&text, &content_lines);
                        
                        if !url.is_empty() {
                            links.push(Link {
                                text,
                                url,
                                line_index,
                            });
                        }
                    }
                }
                
                pos = abs_end + 4;
            } else {
                break;
            }
        }
        
        links
    }

    fn open_link_by_number(&mut self, link_number: usize, open_in_new_tab: bool) {
        if link_number == 0 || link_number > self.current_links.len() {
            self.status_message = format!("Invalid link number. Please enter 1-{}", self.current_links.len());
            return;
        }

        let link_index = link_number - 1;
        if let Some(link) = self.current_links.get(link_index) {
            let mut url = link.url.clone();
            
            // Handle relative URLs
            let current_url = &self.current_tab().url;
            if url.starts_with('/') {
                // Absolute path - need to construct full URL
                if let Ok(parsed) = url::Url::parse(current_url) {
                    if let Some(host) = parsed.host_str() {
                        let scheme = parsed.scheme();
                        url = format!("{}://{}{}", scheme, host, url);
                    }
                }
            } else if !url.starts_with("http://") && !url.starts_with("https://") {
                // Relative path
                if let Ok(parsed) = url::Url::parse(current_url) {
                    if let Ok(joined) = parsed.join(&url) {
                        url = joined.to_string();
                    }
                }
            }
            
            if open_in_new_tab {
                self.open_new_tab();
            }
            
            self.url_input = url;
            self.navigate_to_url();
        }
    }

    fn go_back(&mut self) {
        if !self.history.can_go_back() {
            self.status_message = "No previous page in history".to_string();
            return;
        }
        
        if let Some(entry) = self.history.go_back() {
            let url = entry.url.clone();
            let title = entry.title.clone();
            self.url_input = url;
            self.navigate_to_url();
            self.status_message = format!("Back: {}", title);
        }
    }

    fn go_forward(&mut self) {
        if !self.history.can_go_forward() {
            self.status_message = "No next page in history".to_string();
            return;
        }
        
        if let Some(entry) = self.history.go_forward() {
            let url = entry.url.clone();
            let title = entry.title.clone();
            self.url_input = url;
            self.navigate_to_url();
            self.status_message = format!("Forward: {}", title);
        }
    }

    fn scroll_content_up(&mut self) {
        let tab = self.current_tab_mut();
        tab.scroll_offset = tab.scroll_offset.saturating_sub(SCROLL_STEP);
    }

    fn scroll_content_down(&mut self) {
        let tab = self.current_tab_mut();
        tab.scroll_offset = tab.scroll_offset.saturating_add(SCROLL_STEP);
    }

    fn scroll_content_page_up(&mut self) {
        let tab = self.current_tab_mut();
        tab.scroll_offset = tab.scroll_offset.saturating_sub(PAGE_SCROLL_STEP);
    }

    fn scroll_content_page_down(&mut self) {
        let tab = self.current_tab_mut();
        tab.scroll_offset = tab.scroll_offset.saturating_add(PAGE_SCROLL_STEP);
    }

    fn zoom_in(&mut self) {
        self.content_width_percent = (self.content_width_percent + 0.1).min(1.0);
        self.status_message = format!("Zoom: {}%", (self.content_width_percent * 100.0) as u32);
    }

    fn zoom_out(&mut self) {
        self.content_width_percent = (self.content_width_percent - 0.1).max(0.3);
        self.status_message = format!("Zoom: {}%", (self.content_width_percent * 100.0) as u32);
    }

    fn refresh_page(&mut self) {
        if self.current_tab().url.is_empty() {
            self.status_message = "No page to refresh".to_string();
            return;
        }
        self.status_message = "Refreshing page...".to_string();
        self.navigate_to_url();
    }

    fn start_search(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.search_results.clear();
        self.current_search_result = 0;
        self.status_message = "Search: (type to search, Enter to find next, Esc to cancel)".to_string();
    }

    fn search_in_content(&mut self) {
        if self.search_query.is_empty() {
            self.search_results.clear();
            return;
        }

        let content = self.current_tab().content.clone();
        let query_lower = self.search_query.to_lowercase();
        self.search_results.clear();

        for (line_idx, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&query_lower) {
                self.search_results.push(line_idx);
            }
        }

        if !self.search_results.is_empty() {
            self.current_search_result = 0;
            let line_idx = self.search_results[0];
            self.current_tab_mut().scroll_offset = line_idx;
            self.status_message = format!(
                "Found {} result(s) for '{}' - Match 1/{}",
                self.search_results.len(),
                self.search_query,
                self.search_results.len()
            );
        } else {
            self.status_message = format!("No results found for '{}'", self.search_query);
        }
    }

    fn next_search_result(&mut self) {
        if self.search_results.is_empty() {
            self.status_message = "No search results".to_string();
            return;
        }

        self.current_search_result = (self.current_search_result + 1) % self.search_results.len();
        let line_idx = self.search_results[self.current_search_result];
        self.current_tab_mut().scroll_offset = line_idx;
        self.status_message = format!(
            "Match {}/{} for '{}'",
            self.current_search_result + 1,
            self.search_results.len(),
            self.search_query
        );
    }

    fn previous_search_result(&mut self) {
        if self.search_results.is_empty() {
            self.status_message = "No search results".to_string();
            return;
        }

        if self.current_search_result == 0 {
            self.current_search_result = self.search_results.len() - 1;
        } else {
            self.current_search_result -= 1;
        }
        let line_idx = self.search_results[self.current_search_result];
        self.current_tab_mut().scroll_offset = line_idx;
        self.status_message = format!(
            "Match {}/{} for '{}'",
            self.current_search_result + 1,
            self.search_results.len(),
            self.search_query
        );
    }

    fn handle_key_event(&mut self, key: event::KeyEvent) {
        // Search mode handling
        if self.search_mode {
            match key.code {
                KeyCode::Esc => {
                    self.search_mode = false;
                    self.search_query.clear();
                    self.search_results.clear();
                    self.status_message = "Search cancelled".to_string();
                }
                KeyCode::Enter => {
                    if !self.search_results.is_empty() {
                        self.next_search_result();
                    } else {
                        self.search_in_content();
                    }
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.search_in_content();
                }
                KeyCode::Backspace => {
                    if !self.search_query.is_empty() {
                        self.search_query.pop();
                        self.search_in_content();
                    } else {
                        self.search_mode = false;
                        self.status_message = "Search cancelled".to_string();
                    }
                }
                KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.previous_search_result();
                }
                KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.next_search_result();
                }
                _ => {}
            }
            return;
        }

        // Help dialog handling
        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.show_help = false;
                    self.help_scroll_offset = 0;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.help_scroll_offset = self.help_scroll_offset.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.help_scroll_offset = self.help_scroll_offset.saturating_add(1);
                }
                KeyCode::PageUp => {
                    self.help_scroll_offset = self.help_scroll_offset.saturating_sub(PAGE_SCROLL_STEP);
                }
                KeyCode::PageDown => {
                    self.help_scroll_offset = self.help_scroll_offset.saturating_add(PAGE_SCROLL_STEP);
                }
                _ => {}
            }
            return;
        }

        // Global shortcuts
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('q') => {
                    self.should_quit = true;
                    return;
                }
                KeyCode::Char('t') => {
                    self.open_new_tab();
                    return;
                }
                KeyCode::Char('w') => {
                    self.close_current_tab();
                    return;
                }
                KeyCode::Char('l') => {
                    self.focused_panel = FocusPanel::UrlBar;
                    self.update_url_bar_from_current_tab();
                    return;
                }
                KeyCode::Char('h') => {
                    self.show_help = !self.show_help;
                    return;
                }
                KeyCode::Left => {
                    self.go_back();
                    return;
                }
                KeyCode::Right => {
                    self.go_forward();
                    return;
                }
                KeyCode::Char('r') => {
                    self.refresh_page();
                    return;
                }
                KeyCode::Char('s') => {
                    self.start_search();
                    return;
                }
                _ => {}
            }
        }

        // Panel-specific handling
        match self.focused_panel {
            FocusPanel::TabBar => {
                match key.code {
                    KeyCode::Left => self.previous_tab(),
                    KeyCode::Right => self.next_tab(),
                    KeyCode::Tab => self.cycle_focus(),
                    KeyCode::Char('q') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.should_quit = true;
                    }
                    _ => {}
                }
            }
            FocusPanel::UrlBar => {
                match key.code {
                    KeyCode::Enter => {
                        self.navigate_to_url();
                        self.focused_panel = FocusPanel::Content;
                    }
                    KeyCode::Char(c) => {
                        self.url_input.insert(self.url_cursor_position, c);
                        self.url_cursor_position += 1;
                    }
                    KeyCode::Backspace => {
                        if self.url_cursor_position > 0 {
                            self.url_cursor_position -= 1;
                            self.url_input.remove(self.url_cursor_position);
                        }
                    }
                    KeyCode::Delete => {
                        if self.url_cursor_position < self.url_input.len() {
                            self.url_input.remove(self.url_cursor_position);
                        }
                    }
                    KeyCode::Left => {
                        self.url_cursor_position = self.url_cursor_position.saturating_sub(1);
                    }
                    KeyCode::Right => {
                        self.url_cursor_position = (self.url_cursor_position + 1).min(self.url_input.len());
                    }
                    KeyCode::Home => {
                        self.url_cursor_position = 0;
                    }
                    KeyCode::End => {
                        self.url_cursor_position = self.url_input.len();
                    }
                    KeyCode::Tab => self.cycle_focus(),
                    KeyCode::Esc => {
                        self.focused_panel = FocusPanel::Content;
                    }
                    _ => {}
                }
            }
            FocusPanel::Content => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => self.scroll_content_up(),
                    KeyCode::Down | KeyCode::Char('j') => self.scroll_content_down(),
                    KeyCode::PageUp => self.scroll_content_page_up(),
                    KeyCode::PageDown => self.scroll_content_page_down(),
                    KeyCode::Char('+') | KeyCode::Char('=') => self.zoom_in(),
                    KeyCode::Char('-') | KeyCode::Char('_') => self.zoom_out(),
                    KeyCode::Char('n') => {
                        if !self.search_results.is_empty() {
                            self.next_search_result();
                        }
                    }
                    KeyCode::Char('N') => {
                        if !self.search_results.is_empty() {
                            self.previous_search_result();
                        }
                    }
                    KeyCode::Enter => {
                        // Navigate to link by number
                        if !self.link_number_input.is_empty() {
                            if let Ok(link_num) = self.link_number_input.parse::<usize>() {
                                let open_in_new_tab = key.modifiers.contains(KeyModifiers::CONTROL);
                                self.open_link_by_number(link_num, open_in_new_tab);
                            } else {
                                self.status_message = "Invalid link number".to_string();
                            }
                            self.link_number_input.clear();
                        } else if !self.current_links.is_empty() {
                            self.status_message = format!("Type a link number (1-{}) and press Enter to navigate", self.current_links.len());
                        } else {
                            self.status_message = "No links found on this page".to_string();
                        }
                    }
                    KeyCode::Backspace => {
                        if !self.link_number_input.is_empty() {
                            self.link_number_input.pop();
                            self.status_message = if self.link_number_input.is_empty() {
                                "Link number cleared".to_string()
                            } else {
                                format!("Link number: {}", self.link_number_input)
                            };
                        } else {
                            self.go_back();
                        }
                    }
                    KeyCode::Tab => self.cycle_focus(),
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        self.link_number_input.push(c);
                        self.status_message = format!("Link number: {} (press Enter to navigate, Ctrl+Enter for new tab)", self.link_number_input);
                    }
                    KeyCode::Char('q') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.should_quit = true;
                    }
                    KeyCode::Esc => {
                        if !self.link_number_input.is_empty() {
                            self.link_number_input.clear();
                            self.status_message = "Link number cleared".to_string();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Tab bar
                    Constraint::Length(3), // URL bar
                    Constraint::Min(5),    // Content area
                    Constraint::Length(3), // Status bar
                ])
                .split(f.area());

            // Update content viewport height
            // Subtract border height (top and bottom borders) from total area
            app.content_viewport_height = chunks[2].height.saturating_sub(BORDER_HEIGHT) as usize;

            // Render tab bar
            TabBar::render(
                chunks[0],
                f.buffer_mut(),
                &app.tabs,
                app.current_tab_index,
                app.focused_panel == FocusPanel::TabBar,
            );

            // Render URL bar
            UrlBar::render(
                chunks[1],
                f.buffer_mut(),
                &app.url_input,
                app.url_cursor_position,
                app.focused_panel == FocusPanel::UrlBar,
            );

            // Render content area
            let tab = app.current_tab();
            ContentArea::render(
                chunks[2],
                f.buffer_mut(),
                &tab.content,
                tab.scroll_offset,
                app.focused_panel == FocusPanel::Content,
                tab.loading,
                &app.current_links,
                app.content_width_percent,
            );

            // Render status bar
            let help_text = "Ctrl+H: Help | Ctrl+Q: Quit";
            StatusBar::render(
                chunks[3],
                f.buffer_mut(),
                &app.status_message,
                help_text,
            );

            // Render help dialog if shown
            if app.show_help {
                HelpDialog::render(f.area(), f.buffer_mut(), app.help_scroll_offset);
            }
        })?;

        if app.should_quit {
            return Ok(());
        }

        if let Event::Key(key) = event::read()? {
            app.handle_key_event(key);
        }
    }
}

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new()?;
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}
