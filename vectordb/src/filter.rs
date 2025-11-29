use serde::{Deserialize, Serialize};

use crate::error::{Result, VectorDbError};

/// Represents a filter expression for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterExpr {
    /// Comparison: ["field", "Op", value]
    Comparison(String, ComparisonOp, serde_json::Value),
    /// Logical And: ["And", [filters...]]
    And(Vec<FilterExpr>),
    /// Logical Or: ["Or", [filters...]]
    Or(Vec<FilterExpr>),
    /// Logical Not: ["Not", filter]
    Not(Box<FilterExpr>),
}

/// Comparison operators
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComparisonOp {
    Eq,  // Equal
    Ne,  // Not equal
    Lt,  // Less than
    Le,  // Less than or equal
    Gt,  // Greater than
    Ge,  // Greater than or equal
    In,  // In array
    Contains, // String contains
}

impl FilterExpr {
    /// Parse a filter expression from JSON value
    pub fn from_json(value: &serde_json::Value) -> Result<Self> {
        match value {
            serde_json::Value::Array(arr) if arr.len() >= 2 => {
                let first = arr[0].as_str().ok_or_else(|| {
                    VectorDbError::InvalidInput("Filter operator must be a string".to_string())
                })?;

                match first {
                    "And" => {
                        let filters = arr.get(1).ok_or_else(|| {
                            VectorDbError::InvalidInput("And requires filters array".to_string())
                        })?;
                        let filter_array = filters.as_array().ok_or_else(|| {
                            VectorDbError::InvalidInput("And filters must be an array".to_string())
                        })?;
                        let parsed: Result<Vec<_>> =
                            filter_array.iter().map(FilterExpr::from_json).collect();
                        Ok(FilterExpr::And(parsed?))
                    }
                    "Or" => {
                        let filters = arr.get(1).ok_or_else(|| {
                            VectorDbError::InvalidInput("Or requires filters array".to_string())
                        })?;
                        let filter_array = filters.as_array().ok_or_else(|| {
                            VectorDbError::InvalidInput("Or filters must be an array".to_string())
                        })?;
                        let parsed: Result<Vec<_>> =
                            filter_array.iter().map(FilterExpr::from_json).collect();
                        Ok(FilterExpr::Or(parsed?))
                    }
                    "Not" => {
                        let filter = arr.get(1).ok_or_else(|| {
                            VectorDbError::InvalidInput("Not requires a filter".to_string())
                        })?;
                        Ok(FilterExpr::Not(Box::new(FilterExpr::from_json(filter)?)))
                    }
                    field if arr.len() == 3 => {
                        // Field comparison: ["field", "Op", value]
                        let op_str = arr[1].as_str().ok_or_else(|| {
                            VectorDbError::InvalidInput("Comparison operator must be a string".to_string())
                        })?;
                        let op = match op_str {
                            "Eq" => ComparisonOp::Eq,
                            "Ne" => ComparisonOp::Ne,
                            "Lt" => ComparisonOp::Lt,
                            "Le" => ComparisonOp::Le,
                            "Gt" => ComparisonOp::Gt,
                            "Ge" => ComparisonOp::Ge,
                            "In" => ComparisonOp::In,
                            "Contains" => ComparisonOp::Contains,
                            _ => {
                                return Err(VectorDbError::InvalidInput(format!(
                                    "Unknown comparison operator: {}",
                                    op_str
                                )))
                            }
                        };
                        Ok(FilterExpr::Comparison(field.to_string(), op, arr[2].clone()))
                    }
                    _ => Err(VectorDbError::InvalidInput(format!(
                        "Invalid filter expression: {}",
                        value
                    ))),
                }
            }
            _ => Err(VectorDbError::InvalidInput(format!(
                "Filter must be an array: {}",
                value
            ))),
        }
    }

    /// Evaluate the filter against a document's attributes
    pub fn matches(&self, attributes: &serde_json::Map<String, serde_json::Value>) -> bool {
        match self {
            FilterExpr::Comparison(field, op, value) => {
                if let Some(attr_value) = attributes.get(field) {
                    compare_values(attr_value, op, value)
                } else {
                    false
                }
            }
            FilterExpr::And(filters) => filters.iter().all(|f| f.matches(attributes)),
            FilterExpr::Or(filters) => filters.iter().any(|f| f.matches(attributes)),
            FilterExpr::Not(filter) => !filter.matches(attributes),
        }
    }
}

/// Compare two JSON values using the given operator
fn compare_values(
    left: &serde_json::Value,
    op: &ComparisonOp,
    right: &serde_json::Value,
) -> bool {
    match op {
        ComparisonOp::Eq => left == right,
        ComparisonOp::Ne => left != right,
        ComparisonOp::Lt => compare_ordering(left, right, std::cmp::Ordering::Less),
        ComparisonOp::Le => {
            compare_ordering(left, right, std::cmp::Ordering::Less)
                || compare_ordering(left, right, std::cmp::Ordering::Equal)
        }
        ComparisonOp::Gt => compare_ordering(left, right, std::cmp::Ordering::Greater),
        ComparisonOp::Ge => {
            compare_ordering(left, right, std::cmp::Ordering::Greater)
                || compare_ordering(left, right, std::cmp::Ordering::Equal)
        }
        ComparisonOp::In => {
            if let serde_json::Value::Array(arr) = right {
                arr.contains(left)
            } else {
                false
            }
        }
        ComparisonOp::Contains => {
            if let (serde_json::Value::String(s), serde_json::Value::String(pattern)) = (left, right)
            {
                s.to_lowercase().contains(&pattern.to_lowercase())
            } else {
                false
            }
        }
    }
}

/// Compare values for ordering
fn compare_ordering(
    left: &serde_json::Value,
    right: &serde_json::Value,
    expected: std::cmp::Ordering,
) -> bool {
    match (left, right) {
        (serde_json::Value::Number(a), serde_json::Value::Number(b)) => {
            if let (Some(af), Some(bf)) = (a.as_f64(), b.as_f64()) {
                af.partial_cmp(&bf) == Some(expected)
            } else if let (Some(ai), Some(bi)) = (a.as_i64(), b.as_i64()) {
                ai.cmp(&bi) == expected
            } else {
                false
            }
        }
        (serde_json::Value::String(a), serde_json::Value::String(b)) => a.cmp(b) == expected,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_eq_filter() {
        let filter = FilterExpr::from_json(&json!(["color", "Eq", "blue"])).unwrap();
        let mut attrs = serde_json::Map::new();
        attrs.insert("color".to_string(), json!("blue"));
        assert!(filter.matches(&attrs));

        attrs.insert("color".to_string(), json!("red"));
        assert!(!filter.matches(&attrs));
    }

    #[test]
    fn test_lt_filter() {
        let filter = FilterExpr::from_json(&json!(["price", "Lt", 50000])).unwrap();
        let mut attrs = serde_json::Map::new();
        attrs.insert("price".to_string(), json!(35000));
        assert!(filter.matches(&attrs));

        attrs.insert("price".to_string(), json!(60000));
        assert!(!filter.matches(&attrs));
    }

    #[test]
    fn test_and_filter() {
        let filter = FilterExpr::from_json(&json!([
            "And",
            [
                ["price", "Lt", 60000],
                ["color", "Eq", "blue"]
            ]
        ]))
        .unwrap();

        let mut attrs = serde_json::Map::new();
        attrs.insert("price".to_string(), json!(35000));
        attrs.insert("color".to_string(), json!("blue"));
        assert!(filter.matches(&attrs));

        attrs.insert("color".to_string(), json!("red"));
        assert!(!filter.matches(&attrs));
    }

    #[test]
    fn test_or_filter() {
        let filter = FilterExpr::from_json(&json!([
            "Or",
            [
                ["color", "Eq", "blue"],
                ["color", "Eq", "red"]
            ]
        ]))
        .unwrap();

        let mut attrs = serde_json::Map::new();
        attrs.insert("color".to_string(), json!("blue"));
        assert!(filter.matches(&attrs));

        attrs.insert("color".to_string(), json!("green"));
        assert!(!filter.matches(&attrs));
    }

    #[test]
    fn test_contains_filter() {
        let filter = FilterExpr::from_json(&json!(["description", "Contains", "car"])).unwrap();
        let mut attrs = serde_json::Map::new();
        attrs.insert("description".to_string(), json!("A shiny red sports car"));
        assert!(filter.matches(&attrs));

        attrs.insert("description".to_string(), json!("A blue truck"));
        assert!(!filter.matches(&attrs));
    }

    #[test]
    fn test_in_filter() {
        let filter = FilterExpr::from_json(&json!(["type", "In", ["car", "truck"]])).unwrap();
        let mut attrs = serde_json::Map::new();
        attrs.insert("type".to_string(), json!("car"));
        assert!(filter.matches(&attrs));

        attrs.insert("type".to_string(), json!("bike"));
        assert!(!filter.matches(&attrs));
    }
}
