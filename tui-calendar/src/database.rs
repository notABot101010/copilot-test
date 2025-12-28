use anyhow::Result;
use chrono::{NaiveDate, NaiveTime};
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;

use crate::CalendarEvent;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        
        // Create directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let conn = Connection::open(&db_path)?;
        
        let mut db = Database { conn };
        db.init_schema()?;
        
        Ok(db)
    }
    
    fn get_db_path() -> Result<PathBuf> {
        // Try HOME first (Unix), then USERPROFILE (Windows)
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| anyhow::anyhow!("Cannot determine home directory. Please set HOME or USERPROFILE environment variable."))?;
        
        let mut path = PathBuf::from(home);
        path.push(".tuicalendar");
        path.push("tuicalendar.db");
        
        Ok(path)
    }
    
    fn init_schema(&mut self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                start_date TEXT NOT NULL,
                end_date TEXT,
                start_time TEXT,
                end_time TEXT,
                category TEXT
            )",
            [],
        )?;
        
        Ok(())
    }
    
    pub fn save_event(&self, event: &CalendarEvent) -> Result<()> {
        let start_date_str = event.start_date.format("%Y-%m-%d").to_string();
        let end_date_str = event.end_date.map(|d| d.format("%Y-%m-%d").to_string());
        let start_time_str = event.start_time.map(|t| t.format("%H:%M").to_string());
        let end_time_str = event.end_time.map(|t| t.format("%H:%M").to_string());
        
        self.conn.execute(
            "INSERT OR REPLACE INTO events (id, title, description, start_date, end_date, start_time, end_time, category)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                event.id,
                &event.title,
                &event.description,
                start_date_str,
                end_date_str,
                start_time_str,
                end_time_str,
                &event.category,
            ],
        )?;
        
        Ok(())
    }
    
    pub fn load_events(&self) -> Result<Vec<CalendarEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, start_date, end_date, start_time, end_time, category
             FROM events
             ORDER BY start_date, start_time"
        )?;
        
        let events = stmt.query_map([], |row| {
            let id: usize = row.get(0)?;
            let title: String = row.get(1)?;
            let description: String = row.get(2)?;
            let start_date_str: String = row.get(3)?;
            let end_date_str: Option<String> = row.get(4)?;
            let start_time_str: Option<String> = row.get(5)?;
            let end_time_str: Option<String> = row.get(6)?;
            let category: Option<String> = row.get(7)?;
            
            let start_date = NaiveDate::parse_from_str(&start_date_str, "%Y-%m-%d")
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    3, rusqlite::types::Type::Text, Box::new(e)
                ))?;
            
            let end_date = end_date_str.and_then(|s| {
                NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()
            });
            
            let start_time = start_time_str.and_then(|s| {
                NaiveTime::parse_from_str(&s, "%H:%M").ok()
            });
            
            let end_time = end_time_str.and_then(|s| {
                NaiveTime::parse_from_str(&s, "%H:%M").ok()
            });
            
            Ok(CalendarEvent {
                id,
                title,
                description,
                start_date,
                end_date,
                start_time,
                end_time,
                category,
            })
        })?;
        
        let mut result = Vec::new();
        for event in events {
            result.push(event?);
        }
        
        Ok(result)
    }
    
    pub fn delete_event(&self, event_id: usize) -> Result<()> {
        self.conn.execute(
            "DELETE FROM events WHERE id = ?1",
            params![event_id],
        )?;
        
        Ok(())
    }
    
    pub fn get_max_event_id(&self) -> Result<usize> {
        let max_id: Option<usize> = self.conn.query_row(
            "SELECT MAX(id) FROM events",
            [],
            |row| row.get(0)
        ).ok().flatten();
        
        Ok(max_id.unwrap_or(0))
    }
}
