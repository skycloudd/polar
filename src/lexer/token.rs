use crate::span::Span;
use malachite::Rational;

pub type Spanned<T> = (T, Span);

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Token<'src> {
    Simple(Simple<'src>),
    Parentheses(Vec<Spanned<Token<'src>>>),
    CurlyBraces(Vec<Spanned<Token<'src>>>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Simple<'src> {
    Identifier(&'src str),
    Number(Rational),
    Kw(Kw),
    Punc(Punc),
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kw {
    To,
    Precision,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Punc {
    Plus,
    Minus,
    Star,
    Slash,
    Equals,
}

impl core::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Simple(simple) => write!(f, "{simple}"),
            Self::Parentheses(_tokens) => write!(f, "(...)"),
            Self::CurlyBraces(_tokens) => write!(f, "{{...}}"),
        }
    }
}

impl core::fmt::Display for Simple<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Identifier(ident) => write!(f, "{ident}"),
            Self::Number(num) => write!(f, "{num}"),
            Self::Kw(kw) => write!(f, "{kw}"),
            Self::Punc(punc) => write!(f, "{punc}"),
        }
    }
}

impl core::fmt::Display for Kw {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::To => "to",
                Self::Precision => "precision",
            }
        )
    }
}

impl core::fmt::Display for Punc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Plus => "+",
                Self::Minus => "-",
                Self::Star => "*",
                Self::Slash => "/",
                Self::Equals => "=",
            }
        )
    }
}
