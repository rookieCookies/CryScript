use std::fs::File;

use crate::{utils::{Position, FileData}};

use super::{PositionException, EXCEPTION};

pub struct UnexpectedCharError;

impl UnexpectedCharError {
    pub fn new<'a>(start_position: Position, end_position: Position, file_data: &'a FileData, expected: char, found: char) -> PositionException{
        PositionException::new(start_position, end_position, file_data, "unexpected character", format!("expected {} found {}", expected, found), EXCEPTION)
    }
}

pub struct UnknownTokenError;

impl UnknownTokenError {
    pub fn new<'a>(start_position: Position, end_position: Position, file_data: &'a FileData) -> PositionException{
        PositionException::new(start_position, end_position, file_data, "unknown token", "check the docs for valid tokens".to_string(), EXCEPTION)
    }
}

pub struct InvalidAmountOfDots;

impl InvalidAmountOfDots {
    pub fn new<'a>(start_position: Position, end_position: Position, file_data: &'a FileData) -> PositionException{
        PositionException::new(start_position, end_position, file_data, "invalid amount of dots", "found too many dots while parsing a number".to_string(), EXCEPTION)
    }
}

pub struct NumberParseError;

impl NumberParseError {
    pub fn new<'a>(start_position: Position, end_position: Position, file_data: &'a FileData, number_string: String) -> PositionException{
        PositionException::new(
            start_position, 
            end_position, 
            file_data,
            "number parse error", 
            format!("something went wrong while parsing the following number \"{}\" \n(this is a interpreter related issue, please contact the developers of cryscript)", number_string), 
            EXCEPTION
        )
    }
}
pub struct UnterminatedString;

impl UnterminatedString {
    pub fn new<'a>(start_position: Position, end_position: Position, file_data: &'a FileData) -> PositionException{
        PositionException::new(start_position, end_position, file_data, "unterminated string", "consider adding a quotation mark".to_string(), EXCEPTION)
    }
}