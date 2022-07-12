use crate::utils::{Position, FileData};

use super::{EXCEPTION, PositionException, BasicException};

pub struct UnterminatedParenthesis;

impl UnterminatedParenthesis {
    pub fn new<'a>(start_position: Position, end_position: Position, file_data: &'a FileData) -> PositionException {
        PositionException::new(start_position, end_position, file_data, "unterminated parenthesis", "consider adding a matching parenthesis".to_string(), EXCEPTION)
    }
}

pub struct InvalidSyntax;

impl InvalidSyntax {
    pub fn new<'a>(start_position: Position, end_position: Position, file_data: &'a FileData, note: String) -> PositionException {
        PositionException::new(start_position, end_position, file_data, "invalid syntax", note, EXCEPTION)
    }
}

pub struct InvalidSyntaxExpected;

impl InvalidSyntaxExpected {
    pub fn new<'a>(start_position: Position, end_position: Position, file_data: &'a FileData, expected: String) -> PositionException {
        InvalidSyntax::new(start_position, end_position, file_data, format!("did you mean: {}", expected))
    }
}

pub struct FailedToFindEndOfLine;

impl FailedToFindEndOfLine {
    pub fn new<'a>(start_position: Position, end_position: Position, file_data: &'a FileData) -> PositionException {
        PositionException::new(start_position, end_position, file_data, 
            "failed to find the end of the line", 
            "something went very wrong.\n please contact to the developers of cryscript".to_string(),
        EXCEPTION)
    }
}

pub struct NewLineError;

impl NewLineError {
    pub fn new<'a>(start_position: Position, end_position: Position, file_data: &'a FileData) -> PositionException {
        PositionException::new(start_position, end_position, file_data, 
            "expected new line", 
            "a new line character is required after this statement".to_string(),
        EXCEPTION)
    }
}

pub struct CriticalError;

impl CriticalError {
    pub fn new() -> BasicException {
        BasicException("Something went critically wrong. Please connect to the creators of CryScript".to_string())
    }
}