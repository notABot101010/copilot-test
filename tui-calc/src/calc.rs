use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
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

    fn read_number(&mut self) -> Result<f64> {
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
        
        num_str
            .parse::<f64>()
            .map_err(|_| anyhow!("Failed to parse number"))
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

    fn parse(&mut self) -> Result<f64> {
        let result = self.parse_addition()?;
        if self.pos < self.tokens.len() {
            return Err(anyhow!("Unexpected token after expression"));
        }
        Ok(result)
    }

    fn parse_addition(&mut self) -> Result<f64> {
        let mut left = self.parse_multiplication()?;

        while let Some(token) = self.current() {
            match token {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_multiplication()?;
                    left = left + right;
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_multiplication()?;
                    left = left - right;
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<f64> {
        let mut left = self.parse_power()?;

        while let Some(token) = self.current() {
            match token {
                Token::Multiply => {
                    self.advance();
                    let right = self.parse_power()?;
                    left = left * right;
                }
                Token::Divide => {
                    self.advance();
                    let right = self.parse_power()?;
                    if right == 0.0 {
                        return Err(anyhow!("Division by zero"));
                    }
                    left = left / right;
                }
                Token::Modulo => {
                    self.advance();
                    let right = self.parse_power()?;
                    if right == 0.0 {
                        return Err(anyhow!("Modulo by zero"));
                    }
                    left = left % right;
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_power(&mut self) -> Result<f64> {
        let mut left = self.parse_unary()?;

        if let Some(Token::Power) = self.current() {
            self.advance();
            let right = self.parse_power()?; // Right associative
            left = left.powf(right);
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<f64> {
        if let Some(token) = self.current() {
            match token {
                Token::Minus => {
                    self.advance();
                    let value = self.parse_unary()?;
                    return Ok(-value);
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

    fn parse_primary(&mut self) -> Result<f64> {
        match self.current() {
            Some(Token::Number(n)) => {
                let result = *n;
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

    // Check if expression contains any decimal points
    let has_decimal = expression.contains('.');

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
    
    // Format the result:
    // - If expression contains decimal points or result is fractional, show as float
    // - If expression has no decimals and result is whole, show as integer
    if has_decimal || result.fract() != 0.0 {
        if result.fract() == 0.0 && has_decimal {
            // Result is whole but expression had decimals, show with .0
            Ok(format!("{:.1}", result))
        } else {
            // Result has fractional part
            Ok(format!("{}", result))
        }
    } else if result.is_finite() {
        // Integer result from integer inputs
        Ok(format!("{}", result as i64))
    } else {
        Ok(format!("{}", result))
    }
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
        assert_eq!(evaluate("2.5 * 4").unwrap(), "10.0");
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
        assert_eq!(evaluate("0.1 + 0.2").unwrap(), "0.30000000000000004");
        assert_eq!(evaluate("1.234 * 2").unwrap(), "2.468");
    }
}
