use crate::frontend::{Parser, Precedence};

pub(crate) struct ParseRule<'source> {
    pub(crate) prefix: Option<ParseFn<'source>>,
    pub(crate) infix: Option<ParseFn<'source>>,
    pub(crate) precedence: Precedence,
}

impl<'source> ParseRule<'source> {
    pub(crate) fn new(
        prefix: Option<ParseFn<'source>>,
        infix: Option<ParseFn<'source>>,
        precedence: Precedence,
    ) -> Self {
        Self {
            prefix,
            infix,
            precedence,
        }
    }
}

type ParseFn<'source> = fn(&mut Parser<'source>, bool);
