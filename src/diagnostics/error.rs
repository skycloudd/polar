use super::{Diag, ErrorSpan};
use crate::span::Span;
use chumsky::error::{Rich, RichReason};
use codespan_reporting::diagnostic::Severity;
use core::fmt::Display;
use core::num::ParseIntError;
use std::borrow::Cow;

#[derive(Clone, Debug)]
pub enum Error {
    ExpectedFound {
        expected: Vec<String>,
        found: Option<String>,
        span: Span,
    },
    Custom {
        message: String,
        span: Span,
    },
    UndefinedVariable {
        name: &'static str,
        span: Span,
    },
    PrecisionZero(Span),
    InvalidPrecision {
        span: crate::span::Span,
        err: ParseIntError,
    },
}

impl Diag for Error {
    #[allow(clippy::match_same_arms)]
    fn message(&self) -> Cow<str> {
        match self {
            Self::ExpectedFound {
                expected,
                found,
                span: _,
            } => format!(
                "Expected one of {}, but found {}",
                expected.join(", "),
                found.as_deref().unwrap_or("end of file")
            )
            .into(),
            Self::Custom { message, span: _ } => message.into(),
            Self::UndefinedVariable { name, span: _ } => {
                format!("Undefined variable `{name}`").into()
            }
            Self::PrecisionZero(_) => "Precision must be greater than zero".into(),
            Self::InvalidPrecision { span: _, err } => format!("Invalid precision: {err}").into(),
        }
    }

    #[allow(clippy::match_same_arms)]
    fn spans(&self) -> Vec<ErrorSpan> {
        match self {
            Self::ExpectedFound {
                expected: _,
                found,
                span,
            } => vec![ErrorSpan::primary(
                format!("Found {}", found.as_deref().unwrap_or("end of file")),
                *span,
            )],
            Self::Custom { message: _, span } => vec![ErrorSpan::primary_span(*span)],
            Self::UndefinedVariable { name: _, span } => {
                vec![ErrorSpan::primary("This variable is undefined", *span)]
            }
            Self::PrecisionZero(span) => vec![ErrorSpan::primary_span(*span)],
            Self::InvalidPrecision { span, err: _ } => vec![ErrorSpan::primary_span(*span)],
        }
    }

    fn notes(&self) -> Vec<String> {
        match self {
            Self::ExpectedFound { .. } | Self::Custom { .. } | Self::PrecisionZero(_) => vec![],
            Self::UndefinedVariable { name, span: _ } => {
                vec![
                    format!("Consider assigning a value to `{name}`:"),
                    format!("{name} = <value>"),
                ]
            }
            Self::InvalidPrecision { span: _, err: _ } => {
                vec!["The precision must be a natural number".into()]
            }
        }
    }

    fn kind(&self) -> Severity {
        Severity::Error
    }
}

pub fn convert(error: &Rich<impl Display, Span, &str>) -> Vec<Error> {
    fn convert_inner(reason: &RichReason<impl Display, &str>, span: Span) -> Vec<Error> {
        match reason {
            RichReason::ExpectedFound { expected, found } => vec![Error::ExpectedFound {
                expected: expected.iter().map(ToString::to_string).collect(),
                found: found.as_ref().map(|f| f.to_string()),
                span,
            }],
            RichReason::Custom(message) => vec![Error::Custom {
                message: message.to_owned(),
                span,
            }],
            RichReason::Many(reasons) => reasons
                .iter()
                .flat_map(|r| convert_inner(r, span))
                .collect(),
        }
    }

    convert_inner(error.reason(), *error.span())
}
