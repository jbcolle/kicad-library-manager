use std::cmp::PartialEq;
use std::fs::File;
use std::io::{BufReader, Read};
use std::str::FromStr;
use anyhow::{anyhow, bail};
use crate::symbols::property::{check_expression_validity, KiCadSymbol};

mod property;
mod pin;

pub trait TryFromExpression<T> {
    fn try_from_expression(expression: Expression) -> Result<T, anyhow::Error>;
}

pub(crate) struct KicadSymbolLib {
    version: Option<u64>,
    generator: Option<String>,
    generator_version: Option<f32>,
    pub symbols: Vec<KiCadSymbol>,
}

type Expression = Vec<Token>;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Token {
    OpenParen,
    CloseParen,
    Word(String)
}

impl KicadSymbolLib {
    pub(crate) fn from_file(file: File) -> Result<Self, anyhow::Error> {
        let mut content = String::new();
        let mut reader = BufReader::new(file);
        reader.read_to_string(&mut content)?;

        // println!("content: {content}");
        let expression = tokenise(&content)?;

        check_expression_validity(&expression, "kicad_symbol_lib".to_string())?;
        
        let subexpressions = subdivide_expression(expression[2..expression.len()].to_owned());

        let mut generator = None;
        let mut generator_version = None;
        let mut version = None;
        let mut symbols = Vec::<KiCadSymbol>::new();

        for expression in subexpressions {
            if let Some(Token::Word(property)) = expression.get(1) {
                match property.as_str(){
                    "version" => {
                        version = Some(parse_parameter_from_expression::<u64>(&expression, "version".to_string())?);
                    }
                    "generator" => {
                        generator = Some(parse_parameter_from_expression::<String>(&expression, "generator".to_string())?);
                    }
                    "generator_version" => {
                        generator_version = Some(parse_parameter_from_expression::<f32>(&expression, "generator_version".to_string())?);
                    }
                    "symbol" => {
                        let kicad_symbol = KiCadSymbol::try_from_expression(expression.clone())?;
                        symbols.push(kicad_symbol);
                    }
                    _ => {
                        bail!("Not a valid KiCad symbol library property: {property}");
                    }
                }
            }
        }

        Ok(
            KicadSymbolLib {
                version,
                generator,
                generator_version,
                symbols
            }
        )
    }
}

fn tokenise(input: &str) -> Result<Vec<Token>, anyhow::Error> {
    let mut tokens = Vec::<Token>::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '(' => {
                tokens.push(Token::OpenParen);
                chars.next();
            },
            ')' => {
                tokens.push(Token::CloseParen);
                chars.next();
            },
            ' ' | '\t' | '\n' | '\r' => { chars.next(); },
            '"' => {
                chars.next();
                let mut word = String::new();

                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '"' {
                        break
                    }
                    word.push(c);
                }
                tokens.push(Token::Word(word));
            },
            _ => {
                let mut word = String::new();

                // Read until whitespace or special character
                while let Some(&c) = chars.peek() {
                    if c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '(' || c == ')' {
                        break;
                    }
                    word.push(c);
                    chars.next();
                }

                tokens.push(Token::Word(word));
            }
        }
    }

    Ok(tokens)
}

pub(crate) fn subdivide_expression(expression: Expression) -> Vec<Expression> {
    let mut tokens_peekable = expression.iter().peekable();
    let mut symbols_vec = Vec::<Vec<Token>>::new();
    let mut current_symbol = Vec::<Token>::new();
    let mut open_count = 0;
    
    while let Some(token) = tokens_peekable.peek() {
        let token_clone = token.clone();
        match token {
            Token::OpenParen => {
                current_symbol.push(token_clone.clone());
                open_count += 1;
                tokens_peekable.next();
            }
            Token::CloseParen => {
                current_symbol.push(token_clone.clone());
                if open_count == 1 {
                    symbols_vec.push(current_symbol.clone());
                    current_symbol.clear();
                }
                open_count -= 1;
                tokens_peekable.next();
            }
            Token::Word(_) => {
                current_symbol.push(token_clone.clone());
                tokens_peekable.next();
            }
        }
    }
    
    symbols_vec
    
}

fn parse_parameter_from_expression<T>(expression: &[Token], parameter: String) -> Result<T, anyhow::Error>
where
    T: FromStr, <T as std::str::FromStr>::Err: std::fmt::Display
{
    if expression.len() < 4 {
        bail!("Version expression does not contain four entries");
    }
    if expression[0] != Token::OpenParen {
        bail!("Version expression does not start with opening parentheses");
    }
    if expression[1] != Token::Word(parameter.clone()) {
        bail!("Expression does not contain '{}'", parameter);
    }
    match &expression[2] {
        Token::OpenParen => bail!("No version found"),
        Token::CloseParen => bail!("No version found"),
        Token::Word(value) => value.parse::<T>().map_err(|err| anyhow!("Could not parse value: {err}"))
    }
}

fn check_token_vec_healthy(tokens: Vec<Token>) -> bool {
    tokens.iter().filter(|token| **token == Token::OpenParen).count() == tokens.iter().filter(|token| **token == Token::CloseParen).count()
}

