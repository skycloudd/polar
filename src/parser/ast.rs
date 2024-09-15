use crate::{span::Spanned, RODEO};
use lasso::Spur;
use malachite::Rational;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Statement {
    Expression(Spanned<Expression>),
    Assign {
        name: Spanned<Identifier>,
        value: Spanned<Expression>,
    },
    SetPrecision(Spanned<Expression>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expression {
    Number(Rational),
    Variable(Spanned<Identifier>),
    BinaryOp {
        op: Spanned<BinaryOp>,
        lhs: Spanned<Box<Expression>>,
        rhs: Spanned<Box<Expression>>,
    },
    UnaryOp {
        op: Spanned<UnaryOp>,
        expr: Spanned<Box<Expression>>,
    },
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl core::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryOp {
    Neg,
}

impl core::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Neg => write!(f, "-"),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identifier(Spanned<Spur>);

impl Identifier {
    pub const fn new(ident: Spanned<Spur>) -> Self {
        Self(ident)
    }
}

impl Identifier {
    pub fn resolve(&self) -> &'static str {
        RODEO.resolve(&self.0 .0)
    }
}
