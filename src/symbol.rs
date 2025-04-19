use std::cmp::PartialEq;
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufReader, Read};
use std::str::FromStr;
use anyhow::{anyhow, bail};

pub(crate) struct KicadSymbolLib {
    version: u64,
    generator: String,
    symbols: Vec<KiCadSymbol>
}

struct KiCadSymbol(String);

#[derive(Debug, PartialEq, Clone)]
enum Token {
    OpenParen,
    CloseParen,
    Word(String)
}

impl KicadSymbolLib {
    pub(crate) fn from_file(file: File) -> Result<Self, anyhow::Error> {
        let mut content = String::new();
        let mut reader = BufReader::new(file);
        reader.read_to_string(&mut content)?;

        println!("content: {content}");
        let tokens = tokenise(&content)?;
        let mut tokens_deq = VecDeque::from(tokens);

        if tokens_deq[0] != Token::OpenParen {
            bail!("Not a valid KiCad symbol")
        }
        tokens_deq.pop_front();
        if tokens_deq[0] != Token::Word("kicad_symbol_lib".to_string()) {
            bail!("Not a valid KiCad symbol: no lib designation")
        }
        tokens_deq.pop_front();
        let Some(last_el) = tokens_deq.iter().last() else { bail!("Not a valid KiCad symbol: no last element") };
        if *last_el != Token::CloseParen {
            bail!("Not a valid KiCad symbol: no closing parenthesis")
        }
        tokens_deq.pop_back();
        
        let expressions = parse_token_vector(tokens_deq);
        let kicad_symbol_lib = lib_from_expression_vec(expressions)?;

        Ok(Self {
            version: 1,
            generator: "dummy".to_string(),
            symbols: Vec::<KiCadSymbol>::new()
        })
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

fn parse_token_vector(tokens: VecDeque<Token>) -> Vec<Vec<Token>> {
    let mut tokens_peekable = tokens.iter().peekable();
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
    
    println!("symbols: {:?}", symbols_vec.len());
    
    symbols_vec
    
}

fn lib_from_expression_vec(expressions: Vec<Vec<Token>>) -> Result<KicadSymbolLib, anyhow::Error> {
    let version = parse_parameter_from_expression(&expressions[0], "version".to_string())?;
    let generator = parse_parameter_from_expression(&expressions[1], "generator".to_string())?;

    let mut symbols = Vec::<KiCadSymbol>::new();

    for index in 2..expressions.len() {
        let kicad_symbol = KiCadSymbol::try_from(expressions[index].clone())?;
        println!("kicad symbol: {}", kicad_symbol.0);
        symbols.push(kicad_symbol);
    }

    Ok(
        KicadSymbolLib {
            version,
            generator,
            symbols
        }
    )

}

impl TryFrom<Vec<Token>> for KiCadSymbol {
    type Error = anyhow::Error;

    fn try_from(expression: Vec<Token>) -> Result<Self, Self::Error> {
        if expression[0] != Token::OpenParen {
            bail!("Symbol expression does not start with opening parentheses");
        }
        if expression[1] != Token::Word("symbol".to_string()) {
            bail!("Expression is not a symbol");
        }
        let mut symbol_str = String::new();
        let mut indentation = 0;

        symbol_str.push('(');
        symbol_str += "symbol";
        let mut prev_token = &expression[1];

        for index in 2..expression.len() {
            let token = &expression[index];
            match token {
                Token::OpenParen => {
                    indentation += 1;
                    if prev_token.clone() != Token::CloseParen {
                        symbol_str.push('\n');
                    }
                    for i in 0..indentation {
                        symbol_str.push(' ')
                    }
                    symbol_str.push('(');
                }
                Token::CloseParen => {
                    indentation -= 1;
                    symbol_str.push(')');
                    symbol_str.push('\n');
                    for i in 0..indentation {
                        symbol_str.push(' ')
                    }

                }
                Token::Word(word) => {
                    if prev_token.clone() != Token::OpenParen {
                        symbol_str.push(' ');
                    }
                    symbol_str += word;
                }
            }
            prev_token = token;
        }

        Ok(KiCadSymbol(symbol_str))
    }
}

fn parse_parameter_from_expression<T>(expression: &Vec<Token>, parameter: String) -> Result<T, anyhow::Error>
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

