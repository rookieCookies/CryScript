use std::rc::Rc;

use crate::{FileData, Position};

use super::{PositionException, EXCEPTION};

pub struct UnterminatedIndentation;

impl UnterminatedIndentation {
    pub(crate) fn call(start: &Position, end: &Position, file_data: &Rc<FileData>) -> ! {
        PositionException::call(
            start,
            end,
            end,
            file_data,
            "unterminated indentation",
            "consider adding a curly bracket",
            &EXCEPTION,
        )
        .run()
    }
}

pub struct UnmatchedDedentToken;

impl UnmatchedDedentToken {
    pub(crate) fn call(start: &Position, end: &Position, file_data: &Rc<FileData>) -> ! {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "unmatched dedent token",
            "this curly bracket has no matching bracket",
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

pub struct InvalidAmountOfDots;

impl InvalidAmountOfDots {
    pub(crate) fn call(start: &Position, end: &Position, file_data: &Rc<FileData>) -> ! {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "too many dots",
            "expected either 0 or 1 dot(s) while parsing the number",
            &EXCEPTION,
        )
        .run()
    }
}

pub struct UnknownToken;

impl UnknownToken {
    pub(crate) fn call(start: &Position, end: &Position, file_data: &Rc<FileData>) -> ! {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "unrecognized token",
            "check the docs for valid token(s)",
            &EXCEPTION,
        )
        .run()
    }
}

pub struct InvalidAnnotation;

impl InvalidAnnotation {
    pub(crate) fn call(start: &Position, end: &Position, file_data: &Rc<FileData>) -> ! {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "invalid annotation",
            "check the docs for valid annotations",
            &EXCEPTION,
        )
        .run()
    }
}
