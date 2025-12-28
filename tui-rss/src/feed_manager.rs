use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    pub url: String,
    pub title: String,
    pub description: String,
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: String,
    pub feed_url: String,
    pub title: String,
    pub description: String,
    pub content: String,
    pub link: String,
    pub published: Option<DateTime<Utc>>,
    pub read: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Storage {
    feeds: Vec<Feed>,
    articles: Vec<Article>,
    read_articles: HashMap<String, bool>,
}

pub struct FeedManager {
    storage: Storage,
    storage_path: PathBuf,
}

impl FeedManager {
    pub fn load() -> Result<Self> {
        let storage_path = Self::get_storage_path()?;
        
        let storage = if storage_path.exists() {
            let content = fs::read_to_string(&storage_path)
                .context("Failed to read storage file")?;
            serde_json::from_str(&content)
                .context("Failed to parse storage file")?
        } else {
            Storage {
                feeds: Vec::new(),
                articles: Vec::new(),
                read_articles: HashMap::new(),
            }
        };

        Ok(Self {
            storage,
            storage_path,
        })
    }

    fn get_storage_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Could not determine home directory")?;
        
        let config_dir = PathBuf::from(home).join(".config").join("tui-rss");
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;
        
        Ok(config_dir.join("feeds.json"))
    }

    fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.storage)
            .context("Failed to serialize storage")?;
        fs::write(&self.storage_path, content)
            .context("Failed to write storage file")?;
        Ok(())
    }

    pub fn add_feed(&mut self, url: String) -> Result<()> {
        // Check if feed already exists
        if self.storage.feeds.iter().any(|f| f.url == url) {
            return Err(anyhow::anyhow!("Feed already exists"));
        }

        // Fetch and parse feed
        let feed_data = Self::fetch_feed(&url)?;
        
        let feed = Feed {
            url: url.clone(),
            title: feed_data.title.unwrap_or_else(|| "Untitled Feed".to_string()),
            description: feed_data.description.unwrap_or_default(),
            last_updated: Some(Utc::now()),
        };

        self.storage.feeds.push(feed);

        // Parse and store articles
        for entry in feed_data.entries {
            let article = Self::parse_article(&url, entry);
            if !self.storage.articles.iter().any(|a| a.id == article.id) {
                self.storage.articles.push(article);
            }
        }

        self.save()?;
        Ok(())
    }

    pub fn delete_feed(&mut self, url: &str) -> Result<()> {
        self.storage.feeds.retain(|f| f.url != url);
        self.storage.articles.retain(|a| a.feed_url != url);
        self.save()?;
        Ok(())
    }

    pub fn refresh_feed(&mut self, url: &str) -> Result<usize> {
        let feed_data = Self::fetch_feed(url)?;
        
        // Update feed info
        if let Some(feed) = self.storage.feeds.iter_mut().find(|f| f.url == url) {
            feed.title = feed_data.title.unwrap_or_else(|| feed.title.clone());
            feed.description = feed_data.description.unwrap_or_else(|| feed.description.clone());
            feed.last_updated = Some(Utc::now());
        }

        let mut new_count = 0;
        for entry in feed_data.entries {
            let article = Self::parse_article(url, entry);
            if !self.storage.articles.iter().any(|a| a.id == article.id) {
                self.storage.articles.push(article);
                new_count += 1;
            }
        }

        self.save()?;
        Ok(new_count)
    }

    pub fn refresh_all_feeds(&mut self) -> Result<usize> {
        let mut total_new = 0;
        let feed_urls: Vec<String> = self.storage.feeds.iter().map(|f| f.url.clone()).collect();
        
        for url in feed_urls {
            match self.refresh_feed(&url) {
                Ok(count) => total_new += count,
                Err(_) => continue, // Skip failed feeds
            }
        }
        
        Ok(total_new)
    }

    fn fetch_feed(url: &str) -> Result<ParsedFeed> {
        let response = reqwest::blocking::get(url)
            .context("Failed to fetch feed")?;
        let content = response.text()
            .context("Failed to read feed content")?;
        
        let feed = feed_rs::parser::parse(content.as_bytes())
            .context("Failed to parse feed")?;

        let title = feed.title.map(|t| t.content);
        let description = feed.description.map(|d| d.content);

        let entries: Vec<ParsedEntry> = feed.entries.into_iter().map(|entry| {
            let title = entry.title.map(|t| t.content).unwrap_or_default();
            let summary = entry.summary.map(|s| s.content).unwrap_or_default();
            let content = entry.content
                .and_then(|c| c.body)
                .unwrap_or_else(|| summary.clone());
            
            let link = entry.links.first()
                .map(|l| l.href.clone())
                .unwrap_or_default();

            let published = entry.published.or(entry.updated);

            ParsedEntry {
                id: entry.id,
                title,
                summary,
                content,
                link,
                published,
            }
        }).collect();

        Ok(ParsedFeed {
            title,
            description,
            entries,
        })
    }

    fn parse_article(feed_url: &str, entry: ParsedEntry) -> Article {
        // Convert HTML content to plain text
        let content_text = html2text::from_read(entry.content.as_bytes(), 80);
        let description_text = html2text::from_read(entry.summary.as_bytes(), 120);

        Article {
            id: format!("{}:{}", feed_url, entry.id),
            feed_url: feed_url.to_string(),
            title: entry.title,
            description: description_text,
            content: content_text,
            link: entry.link,
            published: entry.published,
            read: false,
        }
    }

    pub fn get_all_articles(&self) -> Vec<Article> {
        let mut articles: Vec<Article> = self.storage.articles.clone();
        
        // Apply read status
        for article in &mut articles {
            article.read = self.storage.read_articles.get(&article.id).copied().unwrap_or(false);
        }

        // Sort by published date (most recent first)
        articles.sort_by(|a, b| {
            match (b.published, a.published) {
                (Some(b_pub), Some(a_pub)) => b_pub.cmp(&a_pub),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        articles
    }

    pub fn get_articles_for_feed(&self, feed_url: &str) -> Vec<Article> {
        let mut articles: Vec<Article> = self.storage.articles
            .iter()
            .filter(|a| a.feed_url == feed_url)
            .cloned()
            .collect();
        
        // Apply read status
        for article in &mut articles {
            article.read = self.storage.read_articles.get(&article.id).copied().unwrap_or(false);
        }

        // Sort by published date (most recent first)
        articles.sort_by(|a, b| {
            match (b.published, a.published) {
                (Some(b_pub), Some(a_pub)) => b_pub.cmp(&a_pub),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        articles
    }

    fn save_with_warning(&mut self) {
        if let Err(err) = self.save() {
            eprintln!("Warning: Failed to save: {}", err);
        }
    }

    pub fn mark_as_read(&mut self, article_id: &str) {
        self.storage.read_articles.insert(article_id.to_string(), true);
        self.save_with_warning();
    }

    pub fn mark_as_unread(&mut self, article_id: &str) {
        self.storage.read_articles.insert(article_id.to_string(), false);
        self.save_with_warning();
    }

    pub fn get_feeds(&self) -> Vec<Feed> {
        self.storage.feeds.clone()
    }
}

struct ParsedFeed {
    title: Option<String>,
    description: Option<String>,
    entries: Vec<ParsedEntry>,
}

struct ParsedEntry {
    id: String,
    title: String,
    summary: String,
    content: String,
    link: String,
    published: Option<DateTime<Utc>>,
}
