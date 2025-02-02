#![allow(dead_code, unused_imports)]

extern crate alloc;
use alloc::rc::Rc;
use std::str::Chars;

use rc_slice2::RcSlice;
use RcSlice as Rcs;

#[derive(Clone, Debug)]
pub struct ParseState {
    text: RcSlice<[char]>,
    pos: usize,
    line: usize,
    col: usize,
}

#[derive(Debug)]
pub struct Parsing<T> {
    value: T,
    state: ParseState,
}

type Parser<T> = Box<dyn Fn(&ParseState) -> Option<Parsing<T>>>;

impl ParseState {
    pub fn new(text: &str) -> Self {
        let content: Vec<char> = text.chars().collect();
        let rc_chars: Rc<[char]> = content.into();
        Self {
            text: Rcs::new(&rc_chars, 0..),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn advance(&mut self) -> Option<char> {
        self.text
            .get(self.pos)
            .inspect(|&c| {
                self.pos += 1;
                match c {
                    '\n' => {
                        self.line += 1;
                        self.col = 1;
                    }
                    _ => {
                        self.col += 1;
                    }
                }
            })
            .copied()
    }

    pub fn peek(&self) -> Option<char> {
        self.text.get(self.pos).copied()
    }
}

pub fn parse_string(state: &ParseState, s: &str) -> Option<Parsing<String>> {
    let mut current = state.clone();
    let start_pos = current.pos;

    for expected in s.chars() {
        match current.advance() {
            Some(c) if c == expected => continue,
            _ => return None,
        }
    }

    Some(Parsing {
        value: state.text[start_pos..current.pos].to_string(),
        state: current,
    })
}

pub fn whitespace(state: &ParseState) -> Option<Parsing<()>> {
    let mut current = state.clone();
    let mut found = false;

    while let Some(c) = current.peek() {
        if !c.is_whitespace() {
            break;
        }
        current.advance();
        found = true;
    }

    if found {
        Some(Parsing {
            value: (),
            state: current,
        })
    } else {
        None
    }
}

pub fn digits(state: &ParseState) -> Option<Parsing<String>> {
    let mut current = state.clone();
    let start_pos = current.pos;
    let mut found = false;

    while let Some(c) = current.peek() {
        if !c.is_ascii_digit() {
            break;
        }
        current.advance();
        found = true;
    }

    if found {
        Some(Parsing {
            value: state.text[start_pos..current.pos].to_string(),
            state: current,
        })
    } else {
        None
    }
}

pub fn many<T: 'static>(parser: Parser<T>) -> Parser<Vec<T>> {
    Box::new(move |state| {
        let mut current = state.clone();
        let mut results = Vec::new();

        while let Some(parsing) = parser(&current) {
            results.push(parsing.value);
            current = parsing.state;
        }

        Some(Parsing {
            value: results,
            state: current,
        })
    })
}

pub fn many1<T: 'static>(parser: Parser<T>) -> Parser<Vec<T>> {
    Box::new(move |state| {
        let mut current = state.clone();
        let mut results = Vec::new();

        if let Some(parsing) = parser(&current) {
            results.push(parsing.value);
            current = parsing.state;

            while let Some(parsing) = parser(&current) {
                results.push(parsing.value);
                current = parsing.state;
            }

            Some(Parsing {
                value: results,
                state: current,
            })
        } else {
            None
        }
    })
}

pub fn sequence<T: 'static>(parsers: Vec<Parser<T>>) -> Parser<Vec<T>> {
    Box::new(move |state| {
        let mut current = state.clone();
        let mut results = Vec::new();

        for parser in &parsers {
            match parser(&current) {
                Some(parsing) => {
                    results.push(parsing.value);
                    current = parsing.state;
                }
                None => return None,
            }
        }

        Some(Parsing {
            value: results,
            state: current,
        })
    })
}

pub fn map<T: 'static, U, F>(parser: Parser<T>, f: F) -> Parser<U>
where
    F: Fn(T) -> U + 'static,
{
    Box::new(move |state| {
        parser(state).map(|parsing| Parsing {
            value: f(parsing.value),
            state: parsing.state,
        })
    })
}

pub fn any_of<T: 'static>(parsers: Vec<Parser<T>>) -> Parser<T> {
    Box::new(move |state| {
        for parser in &parsers {
            if let Some(result) = parser(state) {
                return Some(result);
            }
        }
        None
    })
}
