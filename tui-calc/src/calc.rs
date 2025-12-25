use anyhow::{anyhow, Result};
use num_bigint::BigInt;
use num_traits::{Pow, Zero};

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(BigInt),
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

    fn read_number(&mut self) -> Result<BigInt> {
        let mut num_str = String::new();
        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        num_str
            .parse::<BigInt>()
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

    fn parse(&mut self) -> Result<BigInt> {
        let result = self.parse_addition()?;
        if self.pos < self.tokens.len() {
            return Err(anyhow!("Unexpected token after expression"));
        }
        Ok(result)
    }

    fn parse_addition(&mut self) -> Result<BigInt> {
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

    fn parse_multiplication(&mut self) -> Result<BigInt> {
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
                    if right.is_zero() {
                        return Err(anyhow!("Division by zero"));
                    }
                    left = left / right;
                }
                Token::Modulo => {
                    self.advance();
                    let right = self.parse_power()?;
                    if right.is_zero() {
                        return Err(anyhow!("Modulo by zero"));
                    }
                    left = left % right;
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_power(&mut self) -> Result<BigInt> {
        let mut left = self.parse_unary()?;

        if let Some(Token::Power) = self.current() {
            self.advance();
            let right = self.parse_power()?; // Right associative

            // Convert BigInt to u32 for exponentiation
            let exp_u32: u32 = right
                .try_into()
                .map_err(|_| anyhow!("Exponent must be a non-negative integer that fits in u32"))?;

            left = left.pow(exp_u32);
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<BigInt> {
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

    fn parse_primary(&mut self) -> Result<BigInt> {
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

pub fn evaluate(expression: &str) -> Result<BigInt> {
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
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        assert_eq!(evaluate("2 + 3").unwrap().to_string(), "5");
        assert_eq!(evaluate("10 - 4").unwrap().to_string(), "6");
        assert_eq!(evaluate("5 * 6").unwrap().to_string(), "30");
        assert_eq!(evaluate("20 / 4").unwrap().to_string(), "5");
        assert_eq!(evaluate("17 % 5").unwrap().to_string(), "2");
    }

    #[test]
    fn test_operator_precedence() {
        assert_eq!(evaluate("2 + 3 * 4").unwrap().to_string(), "14");
        assert_eq!(evaluate("10 - 2 * 3").unwrap().to_string(), "4");
        assert_eq!(evaluate("2 * 3 + 4 * 5").unwrap().to_string(), "26");
    }

    #[test]
    fn test_parentheses() {
        assert_eq!(evaluate("(2 + 3) * 4").unwrap().to_string(), "20");
        assert_eq!(evaluate("((2 + 3) * 4) - 5").unwrap().to_string(), "15");
        assert_eq!(evaluate("2 * (3 + (4 * 5))").unwrap().to_string(), "46");
    }

    #[test]
    fn test_exponentiation() {
        assert_eq!(evaluate("2 ^ 3").unwrap().to_string(), "8");
        assert_eq!(evaluate("2 ^ 3 ^ 2").unwrap().to_string(), "512"); // Right associative
        assert_eq!(evaluate("(2 ^ 3) ^ 2").unwrap().to_string(), "64");
        assert_eq!(evaluate("2 * 3 ^ 2").unwrap().to_string(), "18");
    }

    #[test]
    fn test_large_numbers() {
        assert_eq!(
            evaluate("999999999999999999 + 1").unwrap().to_string(),
            "1000000000000000000"
        );
        assert_eq!(
            evaluate("10 ^ 50").unwrap().to_string(),
            "100000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_negative_numbers() {
        assert_eq!(evaluate("-5 + 3").unwrap().to_string(), "-2");
        assert_eq!(evaluate("-(2 + 3)").unwrap().to_string(), "-5");
        assert_eq!(evaluate("10 + -5").unwrap().to_string(), "5");
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
}
