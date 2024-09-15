use crate::span::Span;

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
    Number {
        before: &'src str,
        after: Option<&'src str>,
        radix: Radix,
    },
    Kw(Kw),
    Punc(Punc),
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Radix {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}

impl Radix {
    pub const fn to_u32(self) -> u32 {
        match self {
            Self::Binary => 2,
            Self::Octal => 8,
            Self::Decimal => 10,
            Self::Hexadecimal => 16,
        }
    }
}

impl core::fmt::Display for Radix {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Binary => "0b",
                Self::Octal => "0o",
                Self::Decimal => "",
                Self::Hexadecimal => "0x",
            }
        )
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kw {
    To,
    Precision,
    FullPrecision,
    Help,
    Exit,
    Vars,
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
            Self::Number {
                before,
                after,
                radix,
            } => write!(f, "{}", number_to_string(before, after.as_deref(), *radix)),
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
                Self::FullPrecision => "fullprecision",
                Self::Help => "help",
                Self::Exit => "exit",
                Self::Vars => "vars",
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

fn number_to_string(before: &str, after: Option<&str>, radix: Radix) -> String {
    format!(
        "{}{}{}",
        radix,
        before,
        after.map_or_else(String::new, |after| format!(".{after}"))
    )
}
