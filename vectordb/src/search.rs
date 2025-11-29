use std::collections::HashMap;

use rust_stemmers::{Algorithm, Stemmer};
use unicode_segmentation::UnicodeSegmentation;

use crate::filter::FilterExpr;
use crate::index::{DistanceMetric, Document, NamespaceIndex};

/// Search result with score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub document: Document,
    pub score: f32,
}

/// Rank by options
#[derive(Debug, Clone)]
pub enum RankBy {
    /// Vector search: ["vector", "ANN", [query_vector]]
    Vector(Vec<f32>),
    /// Full-text search: ["text", "BM25", "query string"]
    Text(String),
    /// Hybrid search combining vector and text
    Hybrid { vector: Vec<f32>, text: String, alpha: f32 },
}

impl RankBy {
    /// Parse rank_by from JSON value
    pub fn from_json(value: &serde_json::Value) -> Result<Self, String> {
        match value {
            serde_json::Value::Array(arr) if arr.len() >= 3 => {
                let rank_type = arr[0].as_str().ok_or("First element must be a string")?;
                let method = arr[1].as_str().ok_or("Second element must be a string")?;

                match (rank_type, method) {
                    ("vector", "ANN") => {
                        let vector = arr[2]
                            .as_array()
                            .ok_or("Third element must be a vector array")?
                            .iter()
                            .filter_map(|v| v.as_f64().map(|f| f as f32))
                            .collect();
                        Ok(RankBy::Vector(vector))
                    }
                    ("text", "BM25") => {
                        let query = arr[2]
                            .as_str()
                            .ok_or("Third element must be a query string")?
                            .to_string();
                        Ok(RankBy::Text(query))
                    }
                    ("hybrid", _) => {
                        // ["hybrid", "ANN+BM25", [vector], "text query", alpha]
                        if arr.len() < 5 {
                            return Err("Hybrid search requires vector, text, and alpha".to_string());
                        }
                        let vector = arr[2]
                            .as_array()
                            .ok_or("Third element must be a vector array")?
                            .iter()
                            .filter_map(|v| v.as_f64().map(|f| f as f32))
                            .collect();
                        let text = arr[3]
                            .as_str()
                            .ok_or("Fourth element must be a text query")?
                            .to_string();
                        let alpha = arr[4]
                            .as_f64()
                            .ok_or("Fifth element must be alpha weight")?
                            as f32;
                        Ok(RankBy::Hybrid { vector, text, alpha })
                    }
                    _ => Err(format!("Unknown rank_by type: {} {}", rank_type, method)),
                }
            }
            _ => Err("rank_by must be an array with at least 3 elements".to_string()),
        }
    }
}

/// Search engine for namespace indexes
pub struct SearchEngine;

impl SearchEngine {
    /// Perform a search on a namespace index
    pub fn search(
        index: &NamespaceIndex,
        rank_by: &RankBy,
        filter: Option<&FilterExpr>,
        top_k: usize,
    ) -> Vec<SearchResult> {
        // First, get candidate documents (optionally filtered)
        let candidates: Vec<&Document> = if let Some(filter) = filter {
            index
                .documents
                .values()
                .filter(|doc| filter.matches(&doc.attributes))
                .collect()
        } else {
            index.documents.values().collect()
        };

        // Score each candidate
        let mut results: Vec<SearchResult> = match rank_by {
            RankBy::Vector(query_vector) => {
                candidates
                    .into_iter()
                    .filter_map(|doc| {
                        doc.vector.as_ref().map(|v| {
                            let score = Self::vector_similarity(v, query_vector, index.distance_metric);
                            SearchResult {
                                document: doc.clone(),
                                score,
                            }
                        })
                    })
                    .collect()
            }
            RankBy::Text(query) => {
                Self::bm25_search(index, query, &candidates)
            }
            RankBy::Hybrid { vector, text, alpha } => {
                // Get vector scores
                let vector_scores: HashMap<String, f32> = candidates
                    .iter()
                    .filter_map(|doc| {
                        doc.vector.as_ref().map(|v| {
                            (
                                doc.id.clone(),
                                Self::vector_similarity(v, vector, index.distance_metric),
                            )
                        })
                    })
                    .collect();

                // Get text scores
                let text_results = Self::bm25_search(index, text, &candidates);
                let text_scores: HashMap<String, f32> = text_results
                    .into_iter()
                    .map(|r| (r.document.id.clone(), r.score))
                    .collect();

                // Normalize and combine scores
                let max_vector = vector_scores.values().copied().fold(0.0f32, f32::max).max(1.0);
                let max_text = text_scores.values().copied().fold(0.0f32, f32::max).max(1.0);

                candidates
                    .into_iter()
                    .map(|doc| {
                        let v_score = vector_scores.get(&doc.id).copied().unwrap_or(0.0) / max_vector;
                        let t_score = text_scores.get(&doc.id).copied().unwrap_or(0.0) / max_text;
                        let combined = alpha * v_score + (1.0 - alpha) * t_score;
                        SearchResult {
                            document: doc.clone(),
                            score: combined,
                        }
                    })
                    .collect()
            }
        };

        // Sort by score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Return top_k results
        results.truncate(top_k);
        results
    }

    /// Calculate vector similarity based on distance metric
    fn vector_similarity(a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        match metric {
            DistanceMetric::CosineDistance => {
                let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
                let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
                let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm_a == 0.0 || norm_b == 0.0 {
                    0.0
                } else {
                    dot / (norm_a * norm_b)
                }
            }
            DistanceMetric::EuclideanDistance => {
                let dist: f32 = a
                    .iter()
                    .zip(b.iter())
                    .map(|(x, y)| (x - y) * (x - y))
                    .sum::<f32>()
                    .sqrt();
                // Convert distance to similarity (inverse)
                1.0 / (1.0 + dist)
            }
            DistanceMetric::DotProduct => {
                a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
            }
        }
    }

    /// BM25 full-text search
    fn bm25_search(
        index: &NamespaceIndex,
        query: &str,
        candidates: &[&Document],
    ) -> Vec<SearchResult> {
        let stemmer = Stemmer::create(Algorithm::English);
        let query_terms: Vec<String> = query
            .unicode_words()
            .map(|w| stemmer.stem(&w.to_lowercase()).to_string())
            .collect();

        if query_terms.is_empty() {
            return Vec::new();
        }

        // BM25 parameters
        let k1 = 1.2;
        let b = 0.75;
        let n = index.documents.len() as f32;
        let avg_dl = if n > 0.0 {
            index.doc_term_counts.values().sum::<usize>() as f32 / n
        } else {
            1.0
        };

        let candidate_ids: std::collections::HashSet<_> =
            candidates.iter().map(|d| &d.id).collect();

        candidates
            .iter()
            .map(|doc| {
                let dl = *index.doc_term_counts.get(&doc.id).unwrap_or(&1) as f32;
                let mut score = 0.0;

                for term in &query_terms {
                    if let Some(posting) = index.inverted_index.get(term) {
                        // Document frequency (only count candidates)
                        let df = posting
                            .keys()
                            .filter(|id| candidate_ids.contains(id))
                            .count() as f32;

                        // Term frequency in this document
                        if let Some(&tf) = posting.get(&doc.id) {
                            // IDF component
                            let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();

                            // TF component with length normalization
                            let tf_component =
                                (tf * (k1 + 1.0)) / (tf + k1 * (1.0 - b + b * dl / avg_dl));

                            score += idf * tf_component;
                        }
                    }
                }

                SearchResult {
                    document: (*doc).clone(),
                    score,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_index() -> NamespaceIndex {
        let mut index = NamespaceIndex::new(DistanceMetric::CosineDistance);

        // Add some test documents
        let docs = vec![
            ("1", vec![0.1, 0.2], "A cat sleeping on a windowsill", "animal"),
            ("2", vec![0.15, 0.25], "A playful kitten chasing a toy", "animal"),
            ("3", vec![0.8, 0.9], "An airplane flying through clouds", "vehicle"),
        ];

        for (id, vector, text, category) in docs {
            let mut attrs = serde_json::Map::new();
            attrs.insert("text".to_string(), json!(text));
            attrs.insert("category".to_string(), json!(category));

            index.upsert_document(Document {
                id: id.to_string(),
                vector: Some(vector),
                attributes: attrs,
            });
        }

        index
    }

    #[test]
    fn test_vector_search() {
        let index = create_test_index();
        let rank_by = RankBy::Vector(vec![0.12, 0.22]);

        let results = SearchEngine::search(&index, &rank_by, None, 2);

        assert_eq!(results.len(), 2);
        // Cat and kitten should be most similar to [0.12, 0.22]
        assert!(results[0].document.id == "1" || results[0].document.id == "2");
    }

    #[test]
    fn test_text_search() {
        let index = create_test_index();
        let rank_by = RankBy::Text("cat sleeping".to_string());

        let results = SearchEngine::search(&index, &rank_by, None, 2);

        assert!(!results.is_empty());
        assert_eq!(results[0].document.id, "1"); // Cat document should be first
    }

    #[test]
    fn test_filtered_search() {
        let index = create_test_index();
        let rank_by = RankBy::Vector(vec![0.1, 0.2]);
        let filter = FilterExpr::from_json(&json!(["category", "Eq", "animal"])).unwrap();

        let results = SearchEngine::search(&index, &rank_by, Some(&filter), 10);

        assert_eq!(results.len(), 2); // Only animal documents
        for result in &results {
            assert_eq!(
                result.document.attributes.get("category"),
                Some(&json!("animal"))
            );
        }
    }

    #[test]
    fn test_hybrid_search() {
        let index = create_test_index();
        let rank_by = RankBy::Hybrid {
            vector: vec![0.12, 0.22],
            text: "cat".to_string(),
            alpha: 0.5,
        };

        let results = SearchEngine::search(&index, &rank_by, None, 2);

        assert!(!results.is_empty());
        // Cat document should rank high due to both vector and text match
        assert_eq!(results[0].document.id, "1");
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0, 0.0];
        let sim = SearchEngine::vector_similarity(&a, &b, DistanceMetric::CosineDistance);
        assert!((sim - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0];
        let sim = SearchEngine::vector_similarity(&a, &c, DistanceMetric::CosineDistance);
        assert!(sim.abs() < 0.001);
    }

    #[test]
    fn test_rank_by_parsing() {
        let json = json!(["vector", "ANN", [0.1, 0.2, 0.3]]);
        let rank_by = RankBy::from_json(&json).unwrap();
        matches!(rank_by, RankBy::Vector(_));

        let json = json!(["text", "BM25", "hello world"]);
        let rank_by = RankBy::from_json(&json).unwrap();
        matches!(rank_by, RankBy::Text(_));

        let json = json!(["hybrid", "ANN+BM25", [0.1, 0.2], "hello", 0.7]);
        let rank_by = RankBy::from_json(&json).unwrap();
        matches!(rank_by, RankBy::Hybrid { .. });
    }
}
