use hemtt_tokens::{whitespace, Symbol, Token};
use peekmore::PeekMoreIterator;

use crate::{Array, Entry, Error, Number, Str};

use super::{Options, Parse};

impl Parse for Entry {
    fn parse(
        options: &Options,
        tokens: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        from: &Token,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let skipped = whitespace::skip_newline(tokens);
        let last = skipped.last().cloned();
        if let Some(token) = tokens.peek() {
            match token.symbol() {
                Symbol::LeftBrace => {
                    let array = Self::Array(Array::parse(
                        options,
                        tokens,
                        &last.unwrap_or_else(|| from.clone()),
                    )?);
                    return Ok(array);
                }
                Symbol::DoubleQuote => {
                    let string = Self::Str(Str::parse(
                        options,
                        tokens,
                        &last.unwrap_or_else(|| from.clone()),
                    )?);
                    return Ok(string);
                }
                Symbol::Digit(_) | Symbol::Dash => {
                    let number = Self::Number(Number::parse(
                        options,
                        tokens,
                        &last.unwrap_or_else(|| from.clone()),
                    )?);
                    return Ok(number);
                }
                Symbol::Newline => {
                    return Err(Error::UnexpectedToken {
                        token: Box::new(tokens.next().unwrap()),
                        expected: vec![Symbol::LeftBrace, Symbol::DoubleQuote, Symbol::Digit(0)],
                    });
                }
                Symbol::Whitespace(_) => {
                    tokens.next();
                    return Self::parse(options, tokens, &last.unwrap_or_else(|| from.clone()));
                }
                _ => {
                    return Err(Error::UnexpectedToken {
                        token: Box::new(token.clone()),
                        expected: vec![Symbol::LeftBrace, Symbol::DoubleQuote, Symbol::Digit(0)],
                    });
                }
            }
        }
        Err(Error::UnexpectedToken {
            token: Box::new(tokens.next().unwrap()),
            expected: vec![Symbol::LeftBrace, Symbol::DoubleQuote, Symbol::Digit(0)],
        })
    }
}

#[cfg(test)]
mod tests {
    use peekmore::PeekMore;

    use super::*;

    #[test]
    fn str() {
        let mut tokens = hemtt_preprocessor::preprocess_string(r#""test""#)
            .unwrap()
            .into_iter()
            .peekmore();
        let entry = Entry::parse(&Options::default(), &mut tokens, &Token::builtin(None)).unwrap();
        assert_eq!(entry, Entry::Str(Str("test".to_string())));
    }

    #[test]
    fn number() {
        for source in [-1, 0, 1, 23] {
            let mut tokens = hemtt_preprocessor::preprocess_string(&source.to_string())
                .unwrap()
                .into_iter()
                .peekmore();
            let number =
                super::Entry::parse(&Options::default(), &mut tokens, &Token::builtin(None))
                    .unwrap();
            assert_eq!(number, super::Entry::Number(Number::Int32(source)));
        }
    }

    #[test]
    fn empty_array() {
        for source in ["{}", "{   }"] {
            let mut tokens = hemtt_preprocessor::preprocess_string(source)
                .unwrap()
                .into_iter()
                .peekmore();
            let array =
                super::Entry::parse(&Options::default(), &mut tokens, &Token::builtin(None))
                    .unwrap();
            assert_eq!(
                array,
                super::Entry::Array(Array {
                    expand: false,
                    elements: vec![]
                })
            );
        }
    }

    #[test]
    fn array() {
        for source in ["{1,2,3}", "{1,   2,3        }", "{ 1, 2, 3 }"] {
            let mut tokens = hemtt_preprocessor::preprocess_string(source)
                .unwrap()
                .into_iter()
                .peekmore();
            let array =
                super::Entry::parse(&Options::default(), &mut tokens, &Token::builtin(None))
                    .unwrap();
            assert_eq!(
                array,
                super::Entry::Array(Array {
                    expand: false,
                    elements: vec![
                        super::Entry::Number(Number::Int32(1)),
                        super::Entry::Number(Number::Int32(2)),
                        super::Entry::Number(Number::Int32(3)),
                    ]
                })
            );
        }
    }

    #[test]
    fn array_trailing_comma() {
        for source in ["{1,2,3,}", "{1,   2,3    ,    }", "{ 1, 2, 3, }"] {
            let mut tokens = hemtt_preprocessor::preprocess_string(source)
                .unwrap()
                .into_iter()
                .peekmore();
            assert!(super::Entry::parse(
                &Options::default(),
                &mut tokens.clone(),
                &Token::builtin(None)
            )
            .is_err());
            assert_eq!(
                super::Entry::parse(
                    &Options::from_preset(crate::parse::Preset::Hemtt),
                    &mut tokens,
                    &Token::builtin(None)
                )
                .unwrap(),
                super::Entry::Array(Array {
                    expand: false,
                    elements: vec![
                        super::Entry::Number(Number::Int32(1)),
                        super::Entry::Number(Number::Int32(2)),
                        super::Entry::Number(Number::Int32(3)),
                    ]
                })
            );
        }
    }
}