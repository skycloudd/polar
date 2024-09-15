use crate::span::Span;
use chumsky::{input::WithContext, prelude::*};
use malachite::{rational_sequences::RationalSequence, Natural, Rational};
use token::{Kw, Punc, Simple, Spanned, Token};

pub mod token;

type ParserInput<'src> = WithContext<Span, &'src str>;

type ParserExtra<'src> = extra::Err<Rich<'src, char, Span, &'src str>>;

#[must_use]
pub fn lexer<'src>(
) -> impl Parser<'src, ParserInput<'src>, Vec<Spanned<Token<'src>>>, ParserExtra<'src>> {
    recursive(|tokens| {
        let ident = text::ascii::ident().map(Simple::Identifier).boxed();

        let number_base = |base: u32| {
            text::int(base)
                .then(
                    just('.')
                        .ignore_then(text::digits(base).to_slice())
                        .or_not(),
                )
                .map(move |(before, after)| {
                    let str_to_digits = |s: &str| {
                        s.chars()
                            .map(|c| Natural::from(c.to_digit(base).unwrap()))
                            .collect()
                    };

                    Rational::from_digits(
                        &Natural::from(base),
                        str_to_digits(before),
                        RationalSequence::from_vec(after.map_or_else(Vec::new, str_to_digits)),
                    )
                })
                .map(Simple::Number)
                .boxed()
        };

        let decimal_number = number_base(10);

        let keyword = choice((text::keyword("to").to(Kw::To),))
            .map(Simple::Kw)
            .boxed();

        let punctuation = choice((
            just('+').to(Punc::Plus),
            just('-').to(Punc::Minus),
            just('*').to(Punc::Star),
            just('/').to(Punc::Slash),
        ))
        .map(Simple::Punc)
        .boxed();

        let simple = choice((keyword, ident, decimal_number, punctuation))
            .map(Token::Simple)
            .boxed();

        let parenthesised = tokens
            .clone()
            .delimited_by(just('('), just(')'))
            .map(Token::Parentheses)
            .boxed();

        let curly_braces = tokens
            .delimited_by(just('{'), just('}'))
            .map(Token::CurlyBraces)
            .boxed();

        let comment = just("//")
            .then(any().and_is(just('\n').not()).repeated())
            .padded()
            .boxed();

        let token = choice((simple, parenthesised, curly_braces))
            .map_with(|token, e| (token, e.span()))
            .padded_by(comment.repeated())
            .padded()
            .boxed();

        token.repeated().collect().padded().boxed()
    })
    .then_ignore(end())
    .boxed()
}

// trait SpannedExt<'src, O> {
//     fn with_span(self) -> impl Parser<'src, ParserInput<'src>, (O, Span), ParserExtra<'src>>;
// }

// impl<'src, P, O> SpannedExt<'src, O> for P
// where
//     P: Parser<'src, ParserInput<'src>, O, ParserExtra<'src>>,
// {
//     fn with_span(self) -> impl Parser<'src, ParserInput<'src>, (O, Span), ParserExtra<'src>> {
//         self.map_with(|t, e| (t, e.span()))
//     }
// }
