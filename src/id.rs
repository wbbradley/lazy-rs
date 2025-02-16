use crate::{location::Location, token::Token, value::Value};

pub const KEYWORDS: &[&str] = &[
    "<-", "->", ":", ";", "else", "if", "let", "match", "do", "then",
];

pub fn parse_id<E: IdErrorTrait>(token: Token) -> Result<Id, crate::error::PitaError> {
    let name = &token.text;
    if !E::is_valid(name) {
        Err(E::new_error(token).into())
    } else {
        Ok(Id { token })
    }
}

pub fn value_from_id<C: IdErrorTrait>(id: &Id) -> Value {
    Value::Id(id.clone())
}

pub fn gensym(location: Location) -> Id {
    static mut COUNTER: u64 = 0;
    Id {
        token: Token {
            text: format!("__{}", unsafe {
                let c = COUNTER;
                COUNTER += 1;
                c
            }),
            location,
        },
    }
}

pub fn internal_id(name: &str) -> Id {
    internal_id_impl::<IdImpl>(name)
}

pub fn internal_ctor_id(name: &str) -> Id {
    internal_id_impl::<CtorIdImpl>(name)
}

pub fn internal_id_impl<E: IdErrorTrait>(name: &str) -> Id {
    debug_assert!(E::is_valid(name));
    parse_id::<E>(Token {
        text: name.to_string(),
        location: Location::unknown(),
    })
    .unwrap()
}

pub trait IdErrorTrait {
    fn is_valid(text: &str) -> bool;
    fn error_text() -> &'static str;
    fn new_error(token: Token) -> IdError<Self>
    where
        Self: Sized,
    {
        IdError::<Self>::new(token)
    }
}

#[derive(Debug, Clone)]
pub struct Id {
    token: Token,
}

#[derive(Debug, Clone)]
pub struct IdImpl;

impl IdErrorTrait for IdImpl {
    fn error_text() -> &'static str {
        "id must start with an alphabetic letter or valid punctuation"
    }
    fn is_valid(text: &str) -> bool {
        text.chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c.is_ascii_punctuation())
            && !KEYWORDS.iter().any(|&kwd| kwd == text)
    }
}

#[derive(Debug, Clone)]
pub struct CtorIdImpl;

impl IdErrorTrait for CtorIdImpl {
    fn error_text() -> &'static str {
        "constructor id must start with an uppercase alphabetic letter"
    }
    fn is_valid(text: &str) -> bool {
        text.chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c.is_uppercase())
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token.text)
    }
}

#[derive(Debug, Clone)]
pub struct IdError<E: IdErrorTrait> {
    pub token: Token,
    _phantom: std::marker::PhantomData<E>,
}

impl<C: IdErrorTrait> IdError<C> {
    fn new(token: Token) -> Self {
        Self {
            token,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<C: IdErrorTrait> std::fmt::Display for IdError<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: error: {}: '{}'",
            self.token.location,
            C::error_text(),
            self.token.text
        )
    }
}
impl<C: IdErrorTrait + std::fmt::Debug> std::error::Error for IdError<C> {}

impl Id {
    pub fn name(&self) -> &str {
        &self.token.text
    }
    pub fn location(&self) -> Location {
        self.token.location
    }
}
