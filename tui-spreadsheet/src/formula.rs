use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};

pub type CellData = String;
pub type CellMap = HashMap<String, CellData>;

/// Convert column letter to index (A=0, B=1, ..., Z=25, AA=26, etc.)
pub fn column_to_index(col: &str) -> usize {
    let mut result = 0;
    for ch in col.chars() {
        result = result * 26 + (ch as usize - 64);
    }
    result - 1
}

/// Convert index to column letter
pub fn index_to_column(mut index: usize) -> String {
    let mut result = String::new();
    index += 1;
    while index > 0 {
        let remainder = (index - 1) % 26;
        result.insert(0, (65 + remainder as u8) as char);
        index = (index - 1) / 26;
    }
    result
}

/// Parse cell reference like A1, B2, AA10
pub fn parse_cell_reference(reference: &str) -> Option<(usize, usize)> {
    let reference = reference.trim().to_uppercase();
    let mut col = String::new();
    let mut row = String::new();
    
    for ch in reference.chars() {
        if ch.is_ascii_alphabetic() {
            col.push(ch);
        } else if ch.is_ascii_digit() {
            row.push(ch);
        } else {
            return None;
        }
    }
    
    if col.is_empty() || row.is_empty() {
        return None;
    }
    
    let col_idx = column_to_index(&col);
    let row_idx = row.parse::<usize>().ok()?.checked_sub(1)?;
    
    Some((row_idx, col_idx))
}

/// Format cell reference from row/col to A1 notation
pub fn format_cell_reference(row: usize, col: usize) -> String {
    format!("{}{}", index_to_column(col), row + 1)
}

/// Get cell key from row/col
pub fn get_cell_key(row: usize, col: usize) -> String {
    format!("{}:{}", row, col)
}

/// Parse range like A1:B5
pub fn parse_range(range: &str) -> Option<((usize, usize), (usize, usize))> {
    let parts: Vec<&str> = range.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    
    let start = parse_cell_reference(parts[0])?;
    let end = parse_cell_reference(parts[1])?;
    
    Some((start, end))
}

/// Get all numeric values in a range
pub fn get_cells_in_range(range: &str, cells: &CellMap) -> Vec<f64> {
    let Some(((start_row, start_col), (end_row, end_col))) = parse_range(range) else {
        return Vec::new();
    };
    
    let min_row = start_row.min(end_row);
    let max_row = start_row.max(end_row);
    let min_col = start_col.min(end_col);
    let max_col = start_col.max(end_col);
    
    let mut values = Vec::new();
    
    for row in min_row..=max_row {
        for col in min_col..=max_col {
            let key = get_cell_key(row, col);
            if let Some(cell_value) = cells.get(&key) {
                if let Ok(num) = cell_value.parse::<f64>() {
                    values.push(num);
                }
            }
        }
    }
    
    values
}

/// Built-in spreadsheet functions
pub fn evaluate_function(name: &str, args: Vec<Vec<f64>>) -> Result<f64> {
    let flat: Vec<f64> = args.into_iter().flatten().collect();
    
    match name {
        "SUM" => Ok(flat.iter().sum()),
        "AVERAGE" => {
            if flat.is_empty() {
                Ok(0.0)
            } else {
                Ok(flat.iter().sum::<f64>() / flat.len() as f64)
            }
        }
        "MIN" => {
            flat.iter()
                .copied()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .ok_or_else(|| anyhow!("MIN requires at least one argument"))
        }
        "MAX" => {
            flat.iter()
                .copied()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .ok_or_else(|| anyhow!("MAX requires at least one argument"))
        }
        "COUNT" => Ok(flat.len() as f64),
        "ROUND" => {
            if flat.is_empty() {
                return Ok(0.0);
            }
            let value = flat[0];
            let decimals = if flat.len() > 1 { flat[1] as i32 } else { 0 };
            let factor = 10_f64.powi(decimals);
            Ok((value * factor).round() / factor)
        }
        "FLOOR" => {
            if flat.is_empty() {
                return Ok(0.0);
            }
            Ok(flat[0].floor())
        }
        "CEIL" => {
            if flat.is_empty() {
                return Ok(0.0);
            }
            Ok(flat[0].ceil())
        }
        "ABS" => {
            if flat.is_empty() {
                return Ok(0.0);
            }
            Ok(flat[0].abs())
        }
        "SQRT" => {
            if flat.is_empty() {
                return Ok(0.0);
            }
            Ok(flat[0].sqrt())
        }
        "POW" => {
            if flat.len() < 2 {
                return Ok(0.0);
            }
            Ok(flat[0].powf(flat[1]))
        }
        "MOD" => {
            if flat.len() < 2 {
                return Ok(0.0);
            }
            Ok(flat[0] % flat[1])
        }
        "PI" => Ok(std::f64::consts::PI),
        _ => Err(anyhow!("Unknown function: {}", name)),
    }
}

/// Tokenize and evaluate a mathematical expression
fn evaluate_expression(expr: &str) -> Result<f64> {
    let tokens = tokenize(expr)?;
    parse_expression(&tokens, &mut 0)
}

#[derive(Debug, Clone)]
enum Token {
    Number(f64),
    Operator(char),
    LParen,
    RParen,
}

fn tokenize(expr: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let ch = chars[i];
        
        if ch.is_whitespace() {
            i += 1;
            continue;
        }
        
        if ch.is_ascii_digit() || ch == '.' {
            let mut num_str = String::new();
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                num_str.push(chars[i]);
                i += 1;
            }
            tokens.push(Token::Number(num_str.parse()?));
            continue;
        }
        
        if ch == '-' && (tokens.is_empty() || matches!(tokens.last(), Some(Token::Operator(_)) | Some(Token::LParen))) {
            // Unary minus
            let mut num_str = String::from("-");
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                num_str.push(chars[i]);
                i += 1;
            }
            if num_str.len() > 1 {
                tokens.push(Token::Number(num_str.parse()?));
            } else {
                tokens.push(Token::Operator('-'));
            }
            continue;
        }
        
        if "+-*/%".contains(ch) {
            tokens.push(Token::Operator(ch));
            i += 1;
            continue;
        }
        
        if ch == '(' {
            tokens.push(Token::LParen);
            i += 1;
            continue;
        }
        
        if ch == ')' {
            tokens.push(Token::RParen);
            i += 1;
            continue;
        }
        
        i += 1;
    }
    
    Ok(tokens)
}

fn parse_expression(tokens: &[Token], pos: &mut usize) -> Result<f64> {
    parse_addition(tokens, pos)
}

fn parse_addition(tokens: &[Token], pos: &mut usize) -> Result<f64> {
    let mut left = parse_multiplication(tokens, pos)?;
    
    while *pos < tokens.len() {
        match &tokens[*pos] {
            Token::Operator('+') => {
                *pos += 1;
                let right = parse_multiplication(tokens, pos)?;
                left += right;
            }
            Token::Operator('-') => {
                *pos += 1;
                let right = parse_multiplication(tokens, pos)?;
                left -= right;
            }
            _ => break,
        }
    }
    
    Ok(left)
}

fn parse_multiplication(tokens: &[Token], pos: &mut usize) -> Result<f64> {
    let mut left = parse_primary(tokens, pos)?;
    
    while *pos < tokens.len() {
        match &tokens[*pos] {
            Token::Operator('*') => {
                *pos += 1;
                let right = parse_primary(tokens, pos)?;
                left *= right;
            }
            Token::Operator('/') => {
                *pos += 1;
                let right = parse_primary(tokens, pos)?;
                if right == 0.0 {
                    return Err(anyhow!("Division by zero"));
                }
                left /= right;
            }
            Token::Operator('%') => {
                *pos += 1;
                let right = parse_primary(tokens, pos)?;
                if right == 0.0 {
                    return Err(anyhow!("Modulo by zero"));
                }
                left %= right;
            }
            _ => break,
        }
    }
    
    Ok(left)
}

fn parse_primary(tokens: &[Token], pos: &mut usize) -> Result<f64> {
    if *pos >= tokens.len() {
        return Err(anyhow!("Unexpected end of expression"));
    }
    
    match &tokens[*pos] {
        Token::Number(n) => {
            *pos += 1;
            Ok(*n)
        }
        Token::LParen => {
            *pos += 1;
            let result = parse_expression(tokens, pos)?;
            if *pos >= tokens.len() || !matches!(tokens[*pos], Token::RParen) {
                return Err(anyhow!("Missing closing parenthesis"));
            }
            *pos += 1;
            Ok(result)
        }
        Token::Operator('-') => {
            *pos += 1;
            Ok(-parse_primary(tokens, pos)?)
        }
        _ => Err(anyhow!("Unexpected token")),
    }
}

/// Evaluate a formula
pub fn evaluate_formula(
    formula: &str,
    cells: &CellMap,
    visited_cells: &mut HashSet<String>,
) -> String {
    if !formula.starts_with('=') {
        return formula.to_string();
    }
    
    let expression = &formula[1..].trim();
    
    match evaluate_formula_internal(expression, cells, visited_cells) {
        Ok(result) => {
            if result.is_nan() || result.is_infinite() {
                "#ERROR".to_string()
            } else if result.fract() == 0.0 && result.abs() < 1e10 {
                format!("{}", result as i64)
            } else {
                // Format with reasonable precision
                format!("{:.6}", result).trim_end_matches('0').trim_end_matches('.').to_string()
            }
        }
        Err(_) => "#ERROR".to_string(),
    }
}

fn evaluate_formula_internal(
    expression: &str,
    cells: &CellMap,
    visited_cells: &mut HashSet<String>,
) -> Result<f64> {
    let mut processed = expression.to_string();
    
    // Replace range references (e.g., A1:B5) with arrays
    let range_regex = regex::Regex::new(r"([A-Z]+\d+):([A-Z]+\d+)").unwrap();
    while let Some(captures) = range_regex.captures(&processed.clone()) {
        let full_match = captures.get(0).unwrap().as_str();
        let values = get_cells_in_range(full_match, cells);
        let values_str = format!("[{}]", values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
        processed = processed.replace(full_match, &values_str);
    }
    
    // Replace function calls
    let func_regex = regex::Regex::new(r"([A-Z]+)\s*\(([^)]*)\)").unwrap();
    while let Some(captures) = func_regex.captures(&processed.clone()) {
        let func_name = captures.get(1).unwrap().as_str();
        let args_str = captures.get(2).unwrap().as_str();
        let full_match = captures.get(0).unwrap().as_str();
        
        let args = parse_function_args(args_str)?;
        let result = evaluate_function(func_name, args)?;
        processed = processed.replace(full_match, &result.to_string());
    }
    
    // Replace single cell references (e.g., A1, B2)
    let cell_regex = regex::Regex::new(r"\b([A-Z]+)(\d+)\b").unwrap();
    while let Some(captures) = cell_regex.captures(&processed.clone()) {
        let full_match = captures.get(0).unwrap().as_str();
        
        if let Some((row, col)) = parse_cell_reference(full_match) {
            let key = get_cell_key(row, col);
            
            // Check for circular reference
            if visited_cells.contains(&key) {
                return Err(anyhow!("Circular reference detected"));
            }
            
            let cell_value = cells.get(&key).map(|s| s.as_str()).unwrap_or("0");
            
            // If referenced cell is a formula, evaluate it
            let value = if cell_value.starts_with('=') {
                visited_cells.insert(key.clone());
                let result = evaluate_formula(cell_value, cells, visited_cells);
                visited_cells.remove(&key);
                result.parse::<f64>().unwrap_or(0.0)
            } else {
                cell_value.parse::<f64>().unwrap_or(0.0)
            };
            
            processed = processed.replace(full_match, &value.to_string());
        }
    }
    
    // Evaluate the final mathematical expression
    evaluate_expression(&processed)
}

fn parse_function_args(args_str: &str) -> Result<Vec<Vec<f64>>> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    
    for ch in args_str.chars() {
        if ch == ',' && depth == 0 {
            if !current.is_empty() {
                args.push(parse_arg_value(&current.trim())?);
                current.clear();
            }
        } else {
            if ch == '[' {
                depth += 1;
            } else if ch == ']' {
                depth -= 1;
            }
            current.push(ch);
        }
    }
    
    if !current.trim().is_empty() {
        args.push(parse_arg_value(&current.trim())?);
    }
    
    Ok(args)
}

fn parse_arg_value(value: &str) -> Result<Vec<f64>> {
    if value.starts_with('[') && value.ends_with(']') {
        let inner = &value[1..value.len() - 1];
        if inner.is_empty() {
            return Ok(Vec::new());
        }
        inner
            .split(',')
            .map(|s| s.trim().parse::<f64>().map_err(|_| anyhow!("Invalid number")))
            .collect()
    } else {
        Ok(vec![value.parse::<f64>().unwrap_or(0.0)])
    }
}

/// Get computed value (handles formulas)
pub fn get_computed_value(row: usize, col: usize, cells: &CellMap) -> String {
    let key = get_cell_key(row, col);
    let cell_value = cells.get(&key).map(|s| s.as_str()).unwrap_or("");
    
    if cell_value.is_empty() {
        return String::new();
    }
    
    if !cell_value.starts_with('=') {
        return cell_value.to_string();
    }
    
    let mut visited_cells = HashSet::new();
    visited_cells.insert(key);
    evaluate_formula(cell_value, cells, &mut visited_cells)
}
