use chumsky::{input::Input as _, Parser as _};
use codespan_reporting::{
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use core::ops::ControlFlow;
use diagnostics::{error::convert, report::report};
use rustyline::error::ReadlineError;
use span::{File, FileId};

pub mod diagnostics;
pub mod lexer;
pub mod span;

fn main() {
    let mut editor = rustyline::DefaultEditor::new().unwrap();

    let mut files = SimpleFiles::new();

    loop {
        let input = editor.readline(">> ");

        match input {
            Ok(input) => {
                let file_id = FileId::new(files.add("<stdin>", input.clone()));

                match handle_input(&files, &input, File::Repl(file_id)) {
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
    files: &SimpleFiles<&str, String>,
    input: &str,
    file_id: File,
) -> ControlFlow<(), ()> {
    println!("input: {input}");

    let mut errors = vec![];

    let (tokens, lexer_errors) = lexer::lexer()
        .parse(input.with_context(file_id))
        .into_output_errors();

    errors.extend(lexer_errors.iter().flat_map(|error| convert(error)));

    println!("tokens: {tokens:?}");

    let writer = StandardStream::stderr(ColorChoice::Auto);
    let term_config = term::Config::default();

    for error in &errors {
        let diagnostic = report(error);

        term::emit(&mut writer.lock(), &term_config, files, &diagnostic).unwrap();
    }

    ControlFlow::Continue(())
}
