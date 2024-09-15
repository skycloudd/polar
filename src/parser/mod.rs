use crate::{
    lexer::token::{Kw, Punc, Simple, Token},
    span::{Span, Spanned},
    RODEO,
};
use ast::{BinaryOp, Expression, Identifier, Statement, UnaryOp};
use chumsky::{extra, input::SpannedInput, prelude::*};

pub mod ast;

type ParserInput<'src, 'tok> = SpannedInput<Token<'src>, Span, &'tok [(Token<'src>, Span)]>;

type ParserExtra<'src, 'tok> = extra::Err<Rich<'tok, Token<'src>, Span, &'src str>>;

pub fn repl<'src: 'tok, 'tok>(
) -> impl Parser<'tok, ParserInput<'src, 'tok>, Statement, ParserExtra<'src, 'tok>> {
    statement()
}

fn statement<'src: 'tok, 'tok>(
) -> impl Parser<'tok, ParserInput<'src, 'tok>, Statement, ParserExtra<'src, 'tok>> {
    let expr = expression().with_span().map(Statement::Expression).boxed();

    let assign = ident()
        .with_span()
        .then_ignore(just(Token::Simple(Simple::Punc(Punc::Equals))))
        .then(expression().with_span())
        .map(|(name, value)| Statement::Assign { name, value })
        .boxed();

    let set_precision = just(Token::Simple(Simple::Kw(Kw::Precision)))
        .ignore_then(expression().with_span())
        .map(Statement::SetPrecision)
        .boxed();

    let help = just(Token::Simple(Simple::Kw(Kw::Help)))
        .ignored()
        .map(|()| Statement::Help)
        .boxed();

    let exit = just(Token::Simple(Simple::Kw(Kw::Exit)))
        .ignored()
        .map(|()| Statement::Exit)
        .boxed();

    choice((assign, expr, set_precision, help, exit)).boxed()
}

fn expression<'src: 'tok, 'tok>(
) -> impl Parser<'tok, ParserInput<'src, 'tok>, Expression, ParserExtra<'src, 'tok>> {
    macro_rules! unary_op {
        ($base:expr, $(($punc:expr => $to:expr)),*) => {{
            let ops = choice((
                $(
                    just(Token::Simple(Simple::Punc($punc))).to($to),
                )*
            ))
            .with_span()
            .boxed();

            ops
                .repeated()
                .foldr($base.with_span(), |op, expr| {
                    let span = op.1.union(expr.1);

                    Spanned::new(
                        Expression::UnaryOp {
                            op,
                            expr: expr.boxed(),
                        },
                        span
                    )
                })
                .map(|expr| expr.0)
                .boxed()
        }};
    }

    macro_rules! binary_op {
        ($base:expr, $(($punc:expr => $to:expr)),*) => {{
            let ops = choice((
                $(
                    just(Token::Simple(Simple::Punc($punc))).to($to),
                )*
            ))
            .with_span()
            .boxed();

            $base
                .clone()
                .with_span()
                .foldl(ops.then($base.with_span()).repeated(), |lhs, (op, rhs)| {
                    let span = lhs.1.union(rhs.1);

                    Spanned::new(
                        Expression::BinaryOp {
                            op,
                            lhs: lhs.boxed(),
                            rhs: rhs.boxed(),
                        },
                        span
                    )
                })
                .map(|expr| expr.0)
                .boxed()
        }};
    }

    recursive(|expression| {
        let number = select! {
            Token::Simple(Simple::Number(num)) => num,
        }
        .map(Expression::Number);

        let variable = ident().with_span().map(Expression::Variable).boxed();

        let parenthesized = expression
            .clone()
            .with_span()
            .parenthesized()
            .map(|expr| expr.0)
            .boxed();

        let atom = choice((parenthesized, number, variable)).boxed();

        let unary = unary_op!(atom, (Punc::Minus => UnaryOp::Neg)).boxed();

        let factor =
            binary_op!(unary, (Punc::Star => BinaryOp::Mul), (Punc::Slash => BinaryOp::Div))
                .boxed();

        binary_op!(factor, (Punc::Plus => BinaryOp::Add), (Punc::Minus => BinaryOp::Sub)).boxed()
    })
    .boxed()
}

fn ident<'src: 'tok, 'tok>(
) -> impl Parser<'tok, ParserInput<'src, 'tok>, Identifier, ParserExtra<'src, 'tok>> {
    select! {
        Token::Simple(Simple::Identifier(ident)) = e => Identifier::new(Spanned::new(RODEO.get_or_intern(ident), e.span())),
    }
    .boxed()
}

trait SpannedExt<'src: 'tok, 'tok, O> {
    fn with_span(
        self,
    ) -> impl Parser<'tok, ParserInput<'src, 'tok>, Spanned<O>, ParserExtra<'src, 'tok>>;

    fn parenthesized(
        self,
    ) -> impl Parser<'tok, ParserInput<'src, 'tok>, O, ParserExtra<'src, 'tok>>;
}

impl<'src: 'tok, 'tok, P, O> SpannedExt<'src, 'tok, O> for P
where
    P: Parser<'tok, ParserInput<'src, 'tok>, O, ParserExtra<'src, 'tok>>,
{
    fn with_span(
        self,
    ) -> impl Parser<'tok, ParserInput<'src, 'tok>, Spanned<O>, ParserExtra<'src, 'tok>> {
        self.map_with(|t, e| Spanned::new(t, e.span()))
    }

    fn parenthesized(
        self,
    ) -> impl Parser<'tok, ParserInput<'src, 'tok>, O, ParserExtra<'src, 'tok>> {
        self.nested_in(select_ref! {
            Token::Parentheses(tokens) = e => tokens.as_slice().spanned(e.span()),
        })
    }
}
