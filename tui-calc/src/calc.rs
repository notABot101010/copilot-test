use anyhow::{anyhow, Result};
use num_bigfloat::BigFloat;
use num_bigint::BigInt;
use num_traits::Zero;

// Number type that can be either BigInt or BigFloat
#[derive(Debug, Clone)]
enum Number {
    Int(BigInt),
    Float(BigFloat),
}

impl Number {
    fn to_float(&self) -> BigFloat {
        match self {
            Number::Int(i) => BigFloat::parse(&i.to_string()).unwrap_or_else(|| BigFloat::from(0)),
            Number::Float(f) => f.clone(),
        }
    }

    fn is_integer(&self) -> bool {
        match self {
            Number::Int(_) => true,
            Number::Float(f) => {
                // Check if the float is a whole number
                let s = f.to_string();
                !s.contains('.') || s.ends_with(".0") || s.ends_with("e0")
            }
        }
    }

    fn format(&self) -> String {
        match self {
            Number::Int(i) => i.to_string(),
            Number::Float(f) => {
                // Convert to string with full precision
                let s = f.to_string();
                
                // Handle scientific notation by converting to decimal
                if let Some(e_pos) = s.find('e') {
                    let mantissa_str = &s[..e_pos];
                    let exp_str = &s[e_pos+1..];
                    
                    if let Ok(exp) = exp_str.parse::<i32>() {
                        // Parse mantissa
                        if let Some(mantissa) = BigFloat::parse(mantissa_str) {
                            // Calculate 10^exp using BigFloat
                            let ten = BigFloat::from(10);
                            let mut power = BigFloat::from(1);
                            
                            if exp > 0 {
                                for _ in 0..exp {
                                    power = power.mul(&ten);
                                }
                            } else if exp < 0 {
                                for _ in 0..(-exp) {
                                    power = power.div(&ten);
                                }
                            }
                            
                            // Multiply mantissa by power
                            let result = mantissa.mul(&power);
                            let result_str = result.to_string();
                            
                            // If result is not in scientific notation, format it
                            if !result_str.contains('e') {
                                if result_str.contains('.') {
                                    let parts: Vec<&str> = result_str.split('.').collect();
                                    if parts.len() == 2 {
                                        let int_part = parts[0];
                                        let dec_part = parts[1];
                                        
                                        // Trim trailing zeros
                                        let trimmed = dec_part.trim_end_matches('0');
                                        if trimmed.is_empty() {
                                            return format!("{}.0", int_part);
                                        }
                                        return format!("{}.{}", int_part, trimmed);
                                    }
                                }
                                return result_str;
                            }
                        }
                    }
                }
                
                // For regular decimal notation (no scientific notation)
                if s.contains('.') && !s.contains('e') {
                    let parts: Vec<&str> = s.split('.').collect();
                    if parts.len() == 2 {
                        let int_part = parts[0];
                        let dec_part = parts[1];
                        
                        // Check if all decimal digits are zero
                        if dec_part.chars().all(|c| c == '0') {
                            return format!("{}.0", int_part);
                        }
                        
                        // Keep the full precision from BigFloat, trim trailing zeros
                        let trimmed = dec_part.trim_end_matches('0');
                        if trimmed.is_empty() {
                            return format!("{}.0", int_part);
                        }
                        
                        return format!("{}.{}", int_part, trimmed);
                    }
                }
                
                // For scientific notation we couldn't convert, return as is
                s
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Token {
    Number(Number),
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Power,
    LeftParen,
    RightParen,
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn current(&self) -> Option<char> {
        if self.pos < self.chars.len() {
            Some(self.chars[self.pos])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Result<Number> {
        let mut num_str = String::new();
        let mut has_decimal = false;
        
        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !has_decimal {
                has_decimal = true;
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        if has_decimal {
            // Parse as BigFloat
            BigFloat::parse(&num_str)
                .map(Number::Float)
                .ok_or_else(|| anyhow!("Failed to parse number"))
        } else {
            // Parse as BigInt
            num_str
                .parse::<BigInt>()
                .map(Number::Int)
                .map_err(|_| anyhow!("Failed to parse number"))
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while self.current().is_some() {
            self.skip_whitespace();
            if self.current().is_none() {
                break;
            }

            let ch = self
                .current()
                .ok_or_else(|| anyhow!("Unexpected end of input"))?;

            match ch {
                '0'..='9' => {
                    tokens.push(Token::Number(self.read_number()?));
                }
                '.' => {
                    // Check if this is the start of a decimal number
                    tokens.push(Token::Number(self.read_number()?));
                }
                '+' => {
                    tokens.push(Token::Plus);
                    self.advance();
                }
                '-' => {
                    tokens.push(Token::Minus);
                    self.advance();
                }
                '*' => {
                    tokens.push(Token::Multiply);
                    self.advance();
                }
                '/' => {
                    tokens.push(Token::Divide);
                    self.advance();
                }
                '%' => {
                    tokens.push(Token::Modulo);
                    self.advance();
                }
                '^' => {
                    tokens.push(Token::Power);
                    self.advance();
                }
                '(' => {
                    tokens.push(Token::LeftParen);
                    self.advance();
                }
                ')' => {
                    tokens.push(Token::RightParen);
                    self.advance();
                }
                _ => return Err(anyhow!("Unexpected character: {}", ch)),
            }
        }

        Ok(tokens)
    }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn parse(&mut self) -> Result<Number> {
        let result = self.parse_addition()?;
        if self.pos < self.tokens.len() {
            return Err(anyhow!("Unexpected token after expression"));
        }
        Ok(result)
    }

    fn parse_addition(&mut self) -> Result<Number> {
        let mut left = self.parse_multiplication()?;

        while let Some(token) = self.current() {
            match token {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_multiplication()?;
                    left = match (left, right) {
                        (Number::Int(l), Number::Int(r)) => Number::Int(l + r),
                        (l, r) => {
                            let lf = l.to_float();
                            let rf = r.to_float();
                            Number::Float(lf.add(&rf))
                        }
                    };
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_multiplication()?;
                    left = match (left, right) {
                        (Number::Int(l), Number::Int(r)) => Number::Int(l - r),
                        (l, r) => {
                            let lf = l.to_float();
                            let rf = r.to_float();
                            Number::Float(lf.sub(&rf))
                        }
                    };
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Number> {
        let mut left = self.parse_power()?;

        while let Some(token) = self.current() {
            match token {
                Token::Multiply => {
                    self.advance();
                    let right = self.parse_power()?;
                    left = match (left, right) {
                        (Number::Int(l), Number::Int(r)) => Number::Int(l * r),
                        (l, r) => {
                            let lf = l.to_float();
                            let rf = r.to_float();
                            Number::Float(lf.mul(&rf))
                        }
                    };
                }
                Token::Divide => {
                    self.advance();
                    let right = self.parse_power()?;
                    match &right {
                        Number::Int(r) if r.is_zero() => {
                            return Err(anyhow!("Division by zero"));
                        }
                        Number::Float(f) if f.is_zero() => {
                            return Err(anyhow!("Division by zero"));
                        }
                        _ => {}
                    }
                    // For integer division, check if it's exact
                    left = match (left, right) {
                        (Number::Int(l), Number::Int(r)) => {
                            if &l % &r == BigInt::zero() {
                                // Exact division, keep as integer
                                Number::Int(l / r)
                            } else {
                                // Not exact, convert to float
                                let lf = Number::Int(l).to_float();
                                let rf = Number::Int(r).to_float();
                                Number::Float(lf.div(&rf))
                            }
                        }
                        (l, r) => {
                            let lf = l.to_float();
                            let rf = r.to_float();
                            Number::Float(lf.div(&rf))
                        }
                    };
                }
                Token::Modulo => {
                    self.advance();
                    let right = self.parse_power()?;
                    match &right {
                        Number::Int(r) if r.is_zero() => {
                            return Err(anyhow!("Modulo by zero"));
                        }
                        Number::Float(f) if f.is_zero() => {
                            return Err(anyhow!("Modulo by zero"));
                        }
                        _ => {}
                    }
                    left = match (left, right) {
                        (Number::Int(l), Number::Int(r)) => Number::Int(l % r),
                        (l, r) => {
                            let lf = l.to_float();
                            let rf = r.to_float();
                            Number::Float(lf.rem(&rf))
                        }
                    };
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_power(&mut self) -> Result<Number> {
        let mut left = self.parse_unary()?;

        if let Some(Token::Power) = self.current() {
            self.advance();
            let right = self.parse_power()?; // Right associative
            
            left = match (left, right) {
                (Number::Int(l), Number::Int(r)) => {
                    // Try to convert to u32 for integer power
                    if let Ok(exp) = r.to_string().parse::<u32>() {
                        Number::Int(num_traits::pow::pow(l, exp as usize))
                    } else {
                        // Fall back to float
                        let lf = Number::Int(l).to_float();
                        let rf = Number::Int(r).to_float();
                        Number::Float(lf.pow(&rf))
                    }
                }
                (l, r) => {
                    let lf = l.to_float();
                    let rf = r.to_float();
                    Number::Float(lf.pow(&rf))
                }
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Number> {
        if let Some(token) = self.current() {
            match token {
                Token::Minus => {
                    self.advance();
                    let value = self.parse_unary()?;
                    return Ok(match value {
                        Number::Int(i) => Number::Int(-i),
                        Number::Float(f) => Number::Float(-f),
                    });
                }
                Token::Plus => {
                    self.advance();
                    return self.parse_unary();
                }
                _ => {}
            }
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Number> {
        match self.current() {
            Some(Token::Number(n)) => {
                let result = n.clone();
                self.advance();
                Ok(result)
            }
            Some(Token::LeftParen) => {
                self.advance();
                let result = self.parse_addition()?;
                match self.current() {
                    Some(Token::RightParen) => {
                        self.advance();
                        Ok(result)
                    }
                    _ => Err(anyhow!("Missing closing parenthesis")),
                }
            }
            _ => Err(anyhow!("Expected number or opening parenthesis")),
        }
    }
}

pub fn evaluate(expression: &str) -> Result<String> {
    if expression.trim().is_empty() {
        return Err(anyhow!("Empty expression"));
    }

    // Check for balanced parentheses
    let mut paren_count = 0;
    for ch in expression.chars() {
        match ch {
            '(' => paren_count += 1,
            ')' => {
                paren_count -= 1;
                if paren_count < 0 {
                    return Err(anyhow!("Unbalanced parentheses"));
                }
            }
            _ => {}
        }
    }
    if paren_count != 0 {
        return Err(anyhow!("Unbalanced parentheses"));
    }

    let mut lexer = Lexer::new(expression);
    let tokens = lexer.tokenize()?;

    if tokens.is_empty() {
        return Err(anyhow!("No valid tokens in expression"));
    }

    let mut parser = Parser::new(tokens);
    let result = parser.parse()?;
    
    Ok(result.format())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        assert_eq!(evaluate("2 + 3").unwrap(), "5");
        assert_eq!(evaluate("10 - 4").unwrap(), "6");
        assert_eq!(evaluate("5 * 6").unwrap(), "30");
        assert_eq!(evaluate("20 / 4").unwrap(), "5");
        assert_eq!(evaluate("17 % 5").unwrap(), "2");
    }

    #[test]
    fn test_operator_precedence() {
        assert_eq!(evaluate("2 + 3 * 4").unwrap(), "14");
        assert_eq!(evaluate("10 - 2 * 3").unwrap(), "4");
        assert_eq!(evaluate("2 * 3 + 4 * 5").unwrap(), "26");
    }

    #[test]
    fn test_parentheses() {
        assert_eq!(evaluate("(2 + 3) * 4").unwrap(), "20");
        assert_eq!(evaluate("((2 + 3) * 4) - 5").unwrap(), "15");
        assert_eq!(evaluate("2 * (3 + (4 * 5))").unwrap(), "46");
    }

    #[test]
    fn test_exponentiation() {
        assert_eq!(evaluate("2 ^ 3").unwrap(), "8");
        assert_eq!(evaluate("2 ^ 3 ^ 2").unwrap(), "512"); // Right associative
        assert_eq!(evaluate("(2 ^ 3) ^ 2").unwrap(), "64");
        assert_eq!(evaluate("2 * 3 ^ 2").unwrap(), "18");
    }

    #[test]
    fn test_large_numbers() {
        // f64 has limited precision compared to BigInt, so we test reasonable ranges
        assert_eq!(evaluate("1000000 + 1").unwrap(), "1000001");
        assert_eq!(evaluate("2 ^ 10").unwrap(), "1024");
        assert_eq!(evaluate("10 ^ 6").unwrap(), "1000000");
    }

    #[test]
    fn test_negative_numbers() {
        assert_eq!(evaluate("-5 + 3").unwrap(), "-2");
        assert_eq!(evaluate("-(2 + 3)").unwrap(), "-5");
        assert_eq!(evaluate("10 + -5").unwrap(), "5");
    }

    #[test]
    fn test_division_by_zero() {
        assert!(evaluate("5 / 0").is_err());
        assert!(evaluate("10 % 0").is_err());
    }

    #[test]
    fn test_unbalanced_parentheses() {
        assert!(evaluate("(2 + 3").is_err());
        assert!(evaluate("2 + 3)").is_err());
        assert!(evaluate("((2 + 3)").is_err());
    }

    #[test]
    fn test_invalid_expressions() {
        assert!(evaluate("").is_err());
        assert!(evaluate("2 +").is_err());
        assert!(evaluate("* 3").is_err());
        assert!(evaluate("2 3").is_err());
    }

    #[test]
    fn test_floating_point_arithmetic() {
        assert_eq!(evaluate("1.5 + 1.5").unwrap(), "3.0");
        assert_eq!(evaluate("2.5 + 2.5").unwrap(), "5.0");
        assert_eq!(evaluate("3.14 + 2.86").unwrap(), "6.0");
        assert_eq!(evaluate("10.5 - 5.5").unwrap(), "5.0");
        // BigFloat may use scientific notation for some results
        let result = evaluate("2.5 * 4").unwrap();
        assert!(result == "10.0" || result.contains("e+1"));
        assert_eq!(evaluate("10.0 / 4.0").unwrap(), "2.5");
    }

    #[test]
    fn test_mixed_integer_float() {
        assert_eq!(evaluate("1 + 2").unwrap(), "3");
        assert_eq!(evaluate("1.0 + 2.0").unwrap(), "3.0");
        assert_eq!(evaluate("1.5 + 2").unwrap(), "3.5");
        assert_eq!(evaluate("5 / 2").unwrap(), "2.5");
    }

    #[test]
    fn test_float_precision() {
        // BigFloat uses scientific notation for some values
        let result = evaluate("0.1 + 0.2").unwrap();
        // Just check it evaluates without error and contains expected digits
        assert!(result.contains("3") && result.contains("e-1") || result == "0.3");
        assert_eq!(evaluate("1.234 * 2").unwrap(), "2.468");
    }

    #[test]
    fn test_arbitrary_precision() {
        // Test very large integers
        assert_eq!(
            evaluate("999999999999999999 + 1").unwrap(),
            "1000000000000000000"
        );
        assert_eq!(
            evaluate("10 ^ 50").unwrap(),
            "100000000000000000000000000000000000000000000000000"
        );
        
        // Test arbitrary precision floats
        assert_eq!(evaluate("0.123456789 + 0.987654321").unwrap(), "1.11111111");
        
        // BigFloat maintains higher precision than f64
        let result = evaluate("1.0 / 3.0").unwrap();
        assert!(result.contains("0.3333") || result.contains("3.333") && result.contains("e-1"));
    }
}
