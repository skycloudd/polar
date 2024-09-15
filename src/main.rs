use chumsky::{input::Input as _, span::Span as _, Parser as _};
use codespan_reporting::{
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use core::ops::ControlFlow;
use diagnostics::{error::convert, report::report};
use evaluator::Evaluator;
use lasso::ThreadedRodeo;
use owo_colors::{AnsiColors, OwoColorize as _};
use rustyline::error::ReadlineError;
use span::{File, FileId, Span};
use std::sync::LazyLock;

mod diagnostics;
mod evaluator;
mod lexer;
mod parser;
mod span;

static RODEO: LazyLock<ThreadedRodeo> = LazyLock::new(ThreadedRodeo::new);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let mut editor = rustyline::DefaultEditor::new()?;

    let mut files = SimpleFiles::new();

    let mut evaluator = Evaluator::default();

    eprintln!("Welcome to Polar v{}!", env!("CARGO_PKG_VERSION"));
    eprintln!("Type `help` for help.");
    eprintln!("Type `exit` to exit the REPL.");

    let mut previous_success = true;

    loop {
        let prompt = if previous_success {
            ">> ".color(AnsiColors::Green)
        } else {
            ">> ".color(AnsiColors::Red)
        };

        let input = editor.readline(&prompt.to_string());

        match input {
            Ok(input) => {
                let file_id = FileId::new(files.add("<stdin>", input.clone()));

                match handle_input(&mut evaluator, &files, &input, File::Repl(file_id))? {
                    ControlFlow::Continue(success) => previous_success = success,
                    ControlFlow::Break(()) => break,
                }
            }
            Err(err) => {
                eprintln!("{err}");

                match err {
                    ReadlineError::Eof | ReadlineError::Interrupted => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn handle_input(
    evaluator: &mut Evaluator,
    files: &SimpleFiles<&str, String>,
    input: &str,
    file_id: File,
) -> Result<ControlFlow<(), bool>, Box<dyn core::error::Error>> {
    let mut errors = vec![];

    let (tokens, lexer_errors) = lexer::lexer()
        .parse(input.with_context(file_id))
        .into_output_errors();

    errors.extend(lexer_errors.iter().flat_map(|error| convert(error)));

    let (statement, parser_errors) = tokens.as_ref().map_or_else(
        || (None, vec![]),
        |tokens| {
            let eoi = tokens
                .last()
                .map_or_else(|| Span::zero(file_id), |(_, span)| span.to_end());

            parser::repl()
                .parse(tokens.spanned(eoi))
                .into_output_errors()
        },
    );

    errors.extend(parser_errors.iter().flat_map(|error| convert(error)));

    if let Some(statement) = statement {
        match evaluator.evaluate_statement(statement) {
            Ok(ControlFlow::Continue(Some(value))) => {
                println!("{}", value.display(evaluator.options()));
            }
            Ok(ControlFlow::Continue(None)) => {}
            Ok(ControlFlow::Break(())) => return Ok(ControlFlow::Break(())),
            Err(err) => errors.push(err),
        }
    }

    let writer = StandardStream::stderr(ColorChoice::Auto);
    let term_config = term::Config::default();

    for error in &errors {
        let diagnostic = report(error);

        term::emit(&mut writer.lock(), &term_config, files, &diagnostic)?;
    }

    Ok(ControlFlow::Continue(errors.is_empty()))
}
