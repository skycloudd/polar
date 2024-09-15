use crate::span::Span;
use codespan_reporting::diagnostic::{LabelStyle, Severity};
use std::borrow::Cow;

pub mod error;
pub mod report;

pub trait Diag {
    fn message(&self) -> Cow<str>;
    fn spans(&self) -> Vec<ErrorSpan>;
    fn notes(&self) -> Vec<String>;
    fn kind(&self) -> Severity;
}

#[derive(Debug)]
pub struct ErrorSpan {
    message: Option<String>,
    span: Span,
    label_style: LabelStyle,
}

impl ErrorSpan {
    fn primary(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: Some(message.into()),
            span,
            label_style: LabelStyle::Primary,
        }
    }

    const fn primary_span(span: Span) -> Self {
        Self {
            message: None,
            span,
            label_style: LabelStyle::Primary,
        }
    }

    // fn secondary(message: impl Into<String>, span: Span) -> Self {
    //     Self {
    //         message: Some(message.into()),
    //         span,
    //         label_style: LabelStyle::Secondary,
    //     }
    // }

    // const fn secondary_span(span: Span) -> Self {
    //     Self {
    //         message: None,
    //         span,
    //         label_style: LabelStyle::Secondary,
    //     }
    // }
}
