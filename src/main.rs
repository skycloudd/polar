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
use rustyline::error::ReadlineError;
use span::{File, FileId, Span};
use std::sync::LazyLock;

mod diagnostics;
mod evaluator;
mod lexer;
mod parser;
mod span;

static RODEO: LazyLock<ThreadedRodeo> = LazyLock::new(ThreadedRodeo::new);

fn main() {
    let mut editor = rustyline::DefaultEditor::new().unwrap();

    let mut files = SimpleFiles::new();

    let mut evaluator = Evaluator::default();

    loop {
        let input = editor.readline(">> ");

        match input {
            Ok(input) => {
                let file_id = FileId::new(files.add("<stdin>", input.clone()));

                match handle_input(&mut evaluator, &files, &input, File::Repl(file_id)) {
                    ControlFlow::Continue(()) => {}
                    ControlFlow::Break(()) => break,
                }
            }
            Err(err) => {
                println!("{err}");

                match err {
                    ReadlineError::Eof | ReadlineError::Interrupted => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

fn handle_input(
    evaluator: &mut Evaluator,
    files: &SimpleFiles<&str, String>,
    input: &str,
    file_id: File,
) -> ControlFlow<(), ()> {
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
        match evaluator.evaluate(statement) {
            Ok(Some(value)) => println!("{}", value.display(evaluator.options())),
            Ok(None) => {}
            Err(err) => errors.push(err),
        }
    }

    let writer = StandardStream::stderr(ColorChoice::Auto);
    let term_config = term::Config::default();

    for error in &errors {
        let diagnostic = report(error);

        term::emit(&mut writer.lock(), &term_config, files, &diagnostic).unwrap();
    }

    ControlFlow::Continue(())
}
