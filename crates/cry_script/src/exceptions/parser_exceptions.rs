use std::rc::Rc;

use crate::{lexer::token::TokenType, FileData, Position};

use super::{PositionException, EXCEPTION};

pub struct UnexpectedToken;

impl UnexpectedToken {
    pub(crate) fn call(
        (start, end, file_data): (&Position, &Position, &Rc<FileData>),
        expected: &str,
        found: &str,
    ) -> ! {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "unexpected character",
            format!("expected » {} « found » {} «", expected, found).as_str(),
            &EXCEPTION,
        )
        .run()
    }
}

pub struct UnterminatedString;

impl UnterminatedString {
    pub(crate) fn call(start: &Position, end: &Position, file_data: &Rc<FileData>) -> ! {
        PositionException::call(
            start,
            end,
            end,
            file_data,
            "unterminated string",
            "consider adding a quotation mark",
            &EXCEPTION,
        )
        .run()
    }
}

pub struct UnterminatedParenthesis;

impl UnterminatedParenthesis {
    pub(crate) fn call(start: &Position, end: &Position, file_data: &Rc<FileData>) -> ! {
        PositionException::call(
            start,
            end,
            end,
            file_data,
            "unterminated parenthesis",
            "consider adding a matching parenthesis",
            &EXCEPTION,
        )
        .run()
    }
}

pub struct NotATypeHint;

impl NotATypeHint {
    pub(crate) fn call(
        start: &Position,
        end: &Position,
        file_data: &Rc<FileData>,
        current: &TokenType,
    ) -> ! {
        PositionException::call(
            start,
            end,
            end,
            file_data,
            "not type hint",
            format!("token \"{}\" is not a valid type hint", current).as_str(),
            &EXCEPTION,
        )
        .run()
    }
}

pub struct InvalidInstructionInClass;

impl InvalidInstructionInClass {
    pub(crate) fn call(start: &Position, end: &Position, file_data: &Rc<FileData>) -> ! {
        PositionException::call(
            start,
            end,
            end,
            file_data,
            "invalid instruction in class declaration",
            "class declarations don't accept this instruction, consider putting it inside a function",
            &EXCEPTION,
        )
        .run()
    }
}
