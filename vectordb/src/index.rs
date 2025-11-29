use std::collections::HashMap;

use rust_stemmers::{Algorithm, Stemmer};
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

/// A document stored in the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector: Option<Vec<f32>>,
    #[serde(flatten)]
    pub attributes: serde_json::Map<String, serde_json::Value>,
}

/// Search index for a namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceIndex {
    /// All documents in this namespace
    pub documents: HashMap<String, Document>,
    /// Inverted index for full-text search: term -> doc_ids with term frequency
    pub inverted_index: HashMap<String, HashMap<String, f32>>,
    /// Document term frequencies for BM25
    pub doc_term_counts: HashMap<String, usize>,
    /// Distance metric for vector search
    pub distance_metric: DistanceMetric,
    /// Vector dimensions (determined from first document)
    pub vector_dimensions: Option<usize>,
}

/// Distance metrics for vector search
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DistanceMetric {
    CosineDistance,
    EuclideanDistance,
    DotProduct,
}

impl Default for DistanceMetric {
    fn default() -> Self {
        DistanceMetric::CosineDistance
    }
}

impl DistanceMetric {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "euclidean_distance" | "euclidean" => DistanceMetric::EuclideanDistance,
            "dot_product" | "dot" => DistanceMetric::DotProduct,
            _ => DistanceMetric::CosineDistance,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            DistanceMetric::CosineDistance => "cosine_distance",
            DistanceMetric::EuclideanDistance => "euclidean_distance",
            DistanceMetric::DotProduct => "dot_product",
        }
    }
}

impl Default for NamespaceIndex {
    fn default() -> Self {
        Self::new(DistanceMetric::default())
    }
}

impl NamespaceIndex {
    pub fn new(distance_metric: DistanceMetric) -> Self {
        Self {
            documents: HashMap::new(),
            inverted_index: HashMap::new(),
            doc_term_counts: HashMap::new(),
            distance_metric,
            vector_dimensions: None,
        }
    }

    /// Add or update a document in the index
    pub fn upsert_document(&mut self, doc: Document) {
        let doc_id = doc.id.clone();

        // Remove old inverted index entries if document exists
        if self.documents.contains_key(&doc_id) {
            self.remove_from_inverted_index(&doc_id);
        }

        // Set vector dimensions from first document with a vector
        if self.vector_dimensions.is_none() {
            if let Some(ref v) = doc.vector {
                self.vector_dimensions = Some(v.len());
            }
        }

        // Index text fields for full-text search
        let terms = self.extract_terms(&doc);
        self.doc_term_counts.insert(doc_id.clone(), terms.len());
        
        for (term, tf) in terms {
            self.inverted_index
                .entry(term)
                .or_default()
                .insert(doc_id.clone(), tf);
        }

        self.documents.insert(doc_id, doc);
    }

    /// Remove a document from the index
    pub fn remove_document(&mut self, doc_id: &str) -> Option<Document> {
        self.remove_from_inverted_index(doc_id);
        self.doc_term_counts.remove(doc_id);
        self.documents.remove(doc_id)
    }

    /// Get a document by ID
    pub fn get_document(&self, doc_id: &str) -> Option<&Document> {
        self.documents.get(doc_id)
    }

    /// Get all document IDs
    pub fn document_ids(&self) -> Vec<String> {
        self.documents.keys().cloned().collect()
    }

    /// Get document count
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    fn remove_from_inverted_index(&mut self, doc_id: &str) {
        let mut empty_terms = Vec::new();
        
        for (term, doc_tfs) in &mut self.inverted_index {
            doc_tfs.remove(doc_id);
            if doc_tfs.is_empty() {
                empty_terms.push(term.clone());
            }
        }
        
        for term in empty_terms {
            self.inverted_index.remove(&term);
        }
    }

    fn extract_terms(&self, doc: &Document) -> HashMap<String, f32> {
        let mut terms = HashMap::new();
        let stemmer = Stemmer::create(Algorithm::English);

        // Extract text from all string attributes
        for (_, value) in &doc.attributes {
            if let Some(text) = value.as_str() {
                self.tokenize_and_stem(text, &stemmer, &mut terms);
            }
        }

        // Convert to TF (term frequency)
        let total_terms = terms.values().sum::<f32>().max(1.0);
        for tf in terms.values_mut() {
            *tf /= total_terms;
        }

        terms
    }

    fn tokenize_and_stem(
        &self,
        text: &str,
        stemmer: &Stemmer,
        terms: &mut HashMap<String, f32>,
    ) {
        for word in text.unicode_words() {
            let lower = word.to_lowercase();
            if lower.len() >= 2 && !is_stop_word(&lower) {
                let stemmed = stemmer.stem(&lower).to_string();
                *terms.entry(stemmed).or_insert(0.0) += 1.0;
            }
        }
    }
}

/// Check if a word is a stop word
fn is_stop_word(word: &str) -> bool {
    matches!(
        word,
        "a" | "an" | "and" | "are" | "as" | "at" | "be" | "by" | "for" | "from" | "has" | "he"
            | "in" | "is" | "it" | "its" | "of" | "on" | "or" | "that" | "the" | "to" | "was"
            | "were" | "will" | "with"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_document_serialization() {
        let mut attrs = serde_json::Map::new();
        attrs.insert("text".to_string(), json!("Hello world"));
        attrs.insert("category".to_string(), json!("greeting"));

        let doc = Document {
            id: "doc1".to_string(),
            vector: Some(vec![0.1, 0.2, 0.3]),
            attributes: attrs,
        };

        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("doc1"));
        assert!(json.contains("Hello world"));

        let parsed: Document = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "doc1");
        assert!(parsed.vector.is_some());
    }

    #[test]
    fn test_namespace_index() {
        let mut index = NamespaceIndex::new(DistanceMetric::CosineDistance);

        let mut attrs = serde_json::Map::new();
        attrs.insert("text".to_string(), json!("A cat sleeping on a windowsill"));

        let doc = Document {
            id: "1".to_string(),
            vector: Some(vec![0.1, 0.2]),
            attributes: attrs,
        };

        index.upsert_document(doc);

        assert_eq!(index.document_count(), 1);
        assert!(index.get_document("1").is_some());

        // Check inverted index was built
        assert!(index.inverted_index.contains_key("cat"));
        assert!(index.inverted_index.contains_key("sleep"));
        assert!(index.inverted_index.contains_key("windowsil"));

        // Remove document
        let removed = index.remove_document("1");
        assert!(removed.is_some());
        assert_eq!(index.document_count(), 0);
        assert!(index.inverted_index.is_empty());
    }

    #[test]
    fn test_distance_metric() {
        assert_eq!(
            DistanceMetric::from_str("cosine_distance"),
            DistanceMetric::CosineDistance
        );
        assert_eq!(
            DistanceMetric::from_str("euclidean_distance"),
            DistanceMetric::EuclideanDistance
        );
        assert_eq!(
            DistanceMetric::from_str("dot_product"),
            DistanceMetric::DotProduct
        );
        assert_eq!(
            DistanceMetric::from_str("unknown"),
            DistanceMetric::CosineDistance
        );
    }
}
