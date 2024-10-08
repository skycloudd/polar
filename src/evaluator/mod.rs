use crate::{
    diagnostics::error::Error,
    parser::ast::{BinaryOp, Expression, Statement, UnaryOp},
};
use core::ops::ControlFlow;
use malachite::{
    num::{
        basic::traits::Zero,
        conversion::{string::options::ToSciOptions, traits::ToSci},
    },
    Rational,
};
use owo_colors::OwoColorize;
use rustc_hash::FxHashMap;

#[derive(Debug, Default)]
pub struct Evaluator {
    names: FxHashMap<&'static str, Value>,
    options: ToSciOptions,
}

impl Evaluator {
    pub fn evaluate_statement(
        &mut self,
        stmt: Statement,
    ) -> Result<ControlFlow<(), Option<Value>>, Error> {
        match stmt {
            Statement::Expression(expr) => self
                .evaluate_expression(expr.0)
                .map(Some)
                .map(ControlFlow::Continue),
            Statement::Assign { name, value } => {
                let value = self.evaluate_expression(value.0)?;
                self.names.insert(name.0.resolve(), value);

                Ok(ControlFlow::Continue(None))
            }
            Statement::SetPrecision(precision) => match self.evaluate_expression(precision.0)? {
                Value::Number(Rational::ZERO) => Err(Error::PrecisionZero(precision.1)),
                prec => {
                    let n = prec.display(self.options).parse().map_err(|err| {
                        Error::InvalidPrecision {
                            span: precision.1,
                            err,
                        }
                    })?;

                    self.options.set_precision(n);

                    eprintln!("Set precision to: {n}");

                    Ok(ControlFlow::Continue(None))
                }
            },
            Statement::FullPrecision => {
                self.options.set_size_complete();

                eprintln!("Using full precision");

                Ok(ControlFlow::Continue(None))
            }
            Statement::Help => {
                print_help();

                Ok(ControlFlow::Continue(None))
            }
            Statement::Exit => {
                eprintln!("Exiting...");

                Ok(ControlFlow::Break(()))
            }
            Statement::Vars => {
                let mut vars = self.names.iter().collect::<Vec<_>>();

                vars.sort_by_key(|(name, _)| *name);

                for (name, value) in vars {
                    println!(
                        "{} = {}{}",
                        name.blue(),
                        value.display(self.options),
                        match value {
                            Value::Number(num) => format!(" = ({num})"),
                        }
                        .black()
                    );
                }

                Ok(ControlFlow::Continue(None))
            }
        }
    }

    fn evaluate_expression(&self, expr: Expression) -> Result<Value, Error> {
        match expr {
            Expression::Number(number) => Ok(Value::Number(number)),
            Expression::Variable(name) => {
                let name = name.map(|name| name.resolve());

                self.names
                    .get(name.0)
                    .cloned()
                    .ok_or(Error::UndefinedVariable {
                        name: name.0,
                        span: name.1,
                    })
            }
            Expression::BinaryOp { op, lhs, rhs } => {
                let lhs = self.evaluate_expression(*lhs.0)?;
                let rhs = self.evaluate_expression(*rhs.0)?;

                {
                    use BinaryOp::{Add, Div, Mul, Sub};
                    use Value::Number;

                    match (op.0, (lhs, rhs)) {
                        (Add, (Number(lhs), Number(rhs))) => Ok(Number(lhs + rhs)),
                        (Sub, (Number(lhs), Number(rhs))) => Ok(Number(lhs - rhs)),
                        (Mul, (Number(lhs), Number(rhs))) => Ok(Number(lhs * rhs)),
                        (Div, (Number(lhs), Number(rhs))) => Ok(Number(lhs / rhs)),
                    }
                }
            }
            Expression::UnaryOp { op, expr } => {
                let expr = self.evaluate_expression(*expr.0)?;

                {
                    use UnaryOp::Neg;
                    use Value::Number;

                    match (op.0, expr) {
                        (Neg, Number(expr)) => Ok(Number(-expr)),
                    }
                }
            }
        }
    }

    pub fn insert(&mut self, name: &'static str, value: f64) {
        let value = Value::Number(Rational::try_from(value).unwrap());

        self.names.insert(name, value);
    }

    pub const fn options(&self) -> ToSciOptions {
        self.options
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Number(Rational),
}

impl Value {
    pub fn display(&self, options: ToSciOptions) -> String {
        match self {
            Self::Number(rational) => rational.to_sci_with_options(options).to_string(),
        }
    }
}

fn print_help() {
    println!("Syntax:");
    println!("  <expr> - Evaluate an expression and print the result");
    println!("  <var> = <expr> - Assign a value to a variable");
    println!();
    println!("Commands:");
    println!("  precision <p> - Set the precision of numbers to <p>");
    println!("  fullprecision - Use full precision for numbers");
    println!("  help - Print this help message");
    println!("  exit - Exit the program");
}
