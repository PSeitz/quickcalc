use core::iter::Peekable;


#[cfg(test)]
mod tests {
    use crate::parse;

    #[test]
    fn test_calulate() {
        let res = parse("1 + 1").unwrap().calculate();
        dbg!(res);
        let res = parse("1+1").unwrap().calculate();
        dbg!(res);
        let res = parse("1 + 1 * 3").unwrap().calculate(); // 4
        dbg!(res);
        let _res = parse("(1 + 1) * 3").unwrap().calculate(); // 6

        assert_eq!(parse("10/5").unwrap().calculate(), 2.0);
        assert_eq!(parse("1 + 1 * 3").unwrap().calculate(), 4.0);
        assert_eq!(parse("(1 + 1) * 3").unwrap().calculate(), 6.0);
        assert_eq!(parse("2^10").unwrap().calculate(), 1024.0);
        assert_eq!(parse("10*1.5").unwrap().calculate(), 15.0);

        let conversion = parse("100*JPY").unwrap().calculate();
        assert_eq!(conversion > 101.0, true);
    }
}


#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use serde::{Deserialize};

lazy_static! {
    static ref RATES: Rates = {

        use ureq::{Agent};
        use std::time::Duration;

        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();
        let body: String = agent.get("https://api.exchangeratesapi.io/latest?base=EUR")
            .call().unwrap()
            .into_string().unwrap();

        serde_json::from_str(&body).unwrap()
        
    };
}

#[derive(Debug, Deserialize)]
struct Rates {
    rates: HashMap<String, f64>
}


#[derive(Debug, Clone)]
pub enum GrammarItem {
    Division,
    Power,
    Product,
    Sum,
    Number(f64),
    Paren
}

#[derive(Debug, Clone)]
pub struct ParseNode {
    pub children: Vec<ParseNode>,
    pub entry: GrammarItem,
}

impl ParseNode {
    pub fn new() -> ParseNode {
        ParseNode {
            children: Vec::new(),
            entry: GrammarItem::Paren,
        }
    }

    pub fn calculate(&mut self) -> f64 {
        match self.entry {
            GrammarItem::Product => {
                let mut first = self.children.pop().expect("expect children in product node").calculate();
                while let Some(mut next) = self.children.pop() {
                    first *= next.calculate();
                }
                first
            },
            GrammarItem::Division => {
                let mut first = self.children.pop().expect("expect children in division node").calculate();
                while let Some(mut next) = self.children.pop() {
                    first = next.calculate() / first;
                }
                first
            },
            GrammarItem::Power => {
                let mut first = self.children.pop().expect("expect children in power node").calculate();
                while let Some(mut next) = self.children.pop() {
                    first = next.calculate().powf(first);
                }
                first
            },
            GrammarItem::Sum=> {
                let mut first = self.children.pop().expect("expect children in sum node").calculate();
                while let Some(mut next) = self.children.pop() {
                    first += next.calculate();
                }
                first
            },
            GrammarItem::Number(val) => {
                val
            },
            GrammarItem::Paren => {
                let mut first = self.children.pop().expect("expect children in parenthesis node");
                first.calculate()
            }
        }


    }

}

#[derive(Debug, Clone)]
pub enum LexItem {
    Paren(char),
    Op(char),
    Num(f64),
}

fn lex(input: &str) -> Result<Vec<LexItem>, String> {
    let mut result = Vec::new();

    let mut it = input.chars().peekable();
    while let Some(&c) = it.peek() {
        match c {
            '0'..='9' | '.' => {
                it.next();
                let n = peek_number(c, &mut it);
                result.push(LexItem::Num(n));
            }
            '+' | '*'| '/'| '^' => {
                result.push(LexItem::Op(c));
                it.next();
            }
            '(' | ')' | '[' | ']' | '{' | '}' => {
                result.push(LexItem::Paren(c));
                it.next();
            }
            ' ' => {
                it.next();
            }
            'A'..='Z' => {
                it.next();
                let n = peek_currency(c, &mut it)?;
                result.push(LexItem::Num(n));
            }
            _ => {
                return Err(format!("unexpected character {}", c));
            }
        }
    }
    Ok(result)
}

fn peek_number<T: Iterator<Item = char>>(c: char, iter: &mut Peekable<T>) -> f64 {
    let mut number = c.to_string();
    while let Some(Some(digit)) = iter.peek().map(|c| {
        match c {
            '0'..='9' | '.' => Some(c.to_string()),
            _ => None,
        }
    }) {
        number+=&digit;
        iter.next();
    }
    number.parse::<f64>().unwrap()
}
fn peek_currency<T: Iterator<Item = char>>(c: char, iter: &mut Peekable<T>) -> Result<f64, String> {
    let mut currency = c.to_string();
    while let Some(Some(next_char)) = iter.peek().map(|c| {
        match c {
            'A'..='Z' => Some(c.to_string()),
            _ => None,
        }
    }) {
        currency+=&next_char;
        iter.next();
    }

    RATES.rates.get(&currency).cloned().ok_or_else(|| "Could not convert to currency".to_string())

}

fn parse_expr(tokens: &Vec<LexItem>, pos: usize) -> Result<(ParseNode, usize), String> {
    let (node_summand, next_pos) = parse_summand(tokens, pos)?;
    let c = tokens.get(next_pos);
    match c {
        Some(&LexItem::Op('+')) => {
            // recurse on the expr
            let mut sum = ParseNode::new();
            sum.entry = GrammarItem::Sum;
            sum.children.push(node_summand);
            let (rhs, i) = parse_expr(tokens, next_pos + 1)?;
            sum.children.push(rhs);
            Ok((sum, i))
        }
        _ => {
            // we have just the summand production, nothing more.
            Ok((node_summand, next_pos))
        }
    }
}

fn parse_summand(tokens: &Vec<LexItem>, pos: usize) -> Result<(ParseNode, usize), String> {
    let (node_term, next_pos) = parse_term(tokens, pos)?;
    let c = tokens.get(next_pos);
    match c {
        Some(&LexItem::Op(c @ '*')) | Some(&LexItem::Op(c @ '/')) | Some(&LexItem::Op(c @ '^'))  => {
            // recurse on the summand
            let mut product = ParseNode::new();
            product.entry = if c == '*' {
                GrammarItem::Product
            }else if c == '^' {
                GrammarItem::Power
            }else{
                GrammarItem::Division
            };
            product.children.push(node_term);
            let (rhs, i) = parse_summand(tokens, next_pos + 1)?;
            product.children.push(rhs);
            Ok((product, i))
        }
        _ => {
            // we have just the term production, nothing more.
            Ok((node_term, next_pos))
        }
    }
}

fn parse_term(tokens: &Vec<LexItem>, pos: usize) -> Result<(ParseNode, usize), String> {
    let c: &LexItem = tokens.get(pos)
        .ok_or(String::from("Unexpected end of input, expected paren or number"))?;
    match c {
        &LexItem::Num(n) => {
            let mut node = ParseNode::new();
            node.entry = GrammarItem::Number(n);
            Ok((node, pos + 1))
        }
        &LexItem::Paren(c) => {
            match c {
                '(' | '[' | '{' => {
                    parse_expr(tokens, pos + 1).and_then(|(node, next_pos)| {
                        if let Some(&LexItem::Paren(c2)) = tokens.get(next_pos) {
                            if c2 == matching(c) {
                                // okay!
                                let mut paren = ParseNode::new();
                                paren.children.push(node);
                                Ok((paren, next_pos + 1))
                            } else {
                                Err(format!("Expected {} but found {} at {}",
                                            matching(c),
                                            c2,
                                            next_pos))
                            }
                        } else {
                            Err(format!("Expected closing paren at {} but found {:?}",
                                        next_pos,
                                        tokens.get(next_pos)))
                        }
                    })
                }
                _ => Err(format!("Expected paren at {} but found {:?}", pos, c)),
            }
        }
        _ => {
            Err(format!("Unexpected token {:?}, expected paren or number", {
                c
            }))
        }
    }
}

fn matching(c: char) -> char {
    match c {
        ')' => '(',
        ']' => '[',
        '}' => '{',
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => panic!("should have been a parenthesis!"),
    }
}

pub fn parse(input: &str) -> Result<ParseNode, String> {
    let tokens = lex(input)?;
    parse_expr(&tokens, 0).and_then(|(n, i)| if i == tokens.len() {
        Ok(n)
    } else {
        Err(format!("Expected end of input, found {:?} at {}", tokens[i], i))
    })
}

