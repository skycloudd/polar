use crate::span::Span;
use chumsky::{input::WithContext, prelude::*};
use token::{Kw, Punc, Radix, Simple, Spanned, Token};

pub mod token;

type ParserInput<'src> = WithContext<Span, &'src str>;

type ParserExtra<'src> = extra::Err<Rich<'src, char, Span, &'src str>>;

pub fn lexer<'src>(
) -> impl Parser<'src, ParserInput<'src>, Vec<Spanned<Token<'src>>>, ParserExtra<'src>> {
    recursive(|tokens| {
        let ident = text::ascii::ident().map(Simple::Identifier).boxed();

        let number_base = |radix: Radix| {
            text::int(radix.to_u32())
                .then(
                    just('.')
                        .ignore_then(text::digits(radix.to_u32()).to_slice())
                        .or_not(),
                )
                .map(move |(before, after)| Simple::Number {
                    before,
                    after,
                    radix,
                })
                .boxed()
        };

        let binary_number = just("0b").ignore_then(number_base(Radix::Binary));
        let octal_number = just("0o").ignore_then(number_base(Radix::Octal));
        let hexadecimal_number = just("0x").ignore_then(number_base(Radix::Hexadecimal));

        let decimal_number = number_base(Radix::Decimal);

        let keyword = choice((
            text::keyword("to").to(Kw::To),
            text::keyword("precision").to(Kw::Precision),
            text::keyword("help").to(Kw::Help),
            text::keyword("exit").to(Kw::Exit),
        ))
        .map(Simple::Kw)
        .boxed();

        let punctuation = choice((
            just('+').to(Punc::Plus),
            just('-').to(Punc::Minus),
            just('*').to(Punc::Star),
            just('/').to(Punc::Slash),
            just('=').to(Punc::Equals),
        ))
        .map(Simple::Punc)
        .boxed();

        let simple = choice((
            keyword,
            ident,
            binary_number,
            octal_number,
            hexadecimal_number,
            decimal_number,
            punctuation,
        ))
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
