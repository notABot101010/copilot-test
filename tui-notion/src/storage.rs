use crate::document::Document;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

pub struct Storage {
    data_dir: PathBuf,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let data_dir = Self::get_data_dir()?;
        fs::create_dir_all(&data_dir)?;
        Ok(Self { data_dir })
    }

    fn get_data_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        
        let data_dir = PathBuf::from(home).join(".tui-notion");
        Ok(data_dir)
    }

    fn get_document_path(&self, doc_id: Uuid) -> PathBuf {
        self.data_dir.join(format!("{}.json", doc_id))
    }

    pub fn save_document(&self, document: &Document) -> Result<()> {
        let path = self.get_document_path(document.id);
        let json = serde_json::to_string_pretty(document)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_document(&self, doc_id: Uuid) -> Result<Document> {
        let path = self.get_document_path(doc_id);
        let json = fs::read_to_string(path)?;
        let document = serde_json::from_str(&json)?;
        Ok(document)
    }

    pub fn delete_document(&self, doc_id: Uuid) -> Result<()> {
        let path = self.get_document_path(doc_id);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn load_all_documents(&self) -> Result<Vec<Document>> {
        let mut documents = Vec::new();
        
        if !self.data_dir.exists() {
            return Ok(documents);
        }

        for entry in fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(json) = fs::read_to_string(&path) {
                    if let Ok(document) = serde_json::from_str::<Document>(&json) {
                        documents.push(document);
                    }
                }
            }
        }
        
        Ok(documents)
    }
}
