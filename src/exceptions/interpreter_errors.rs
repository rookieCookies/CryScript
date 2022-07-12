use crate::{utils::{Position, FileData}, parser::instructions::Literal, lexer::tokens::{TokenKind, Token}};

use super::{EXCEPTION, PositionException};

pub struct InvalidBinaryOperationNumbers;

impl InvalidBinaryOperationNumbers {
    pub fn new<'a>(start_position: Position, end_position: Position, file_data: &'a FileData, number1: Literal, number2: Literal) -> PositionException{
        PositionException::new(
            start_position, 
            end_position, 
            file_data, 
            "invalid binary operation", 
            format!("expected a number type, found {}", number2.kind()), 
            EXCEPTION
        )
    }
    pub fn new_single<'a>(start_position: Position, end_position: Position, file_data: &'a FileData, number1: Literal) -> PositionException{
        PositionException::new(
            start_position, 
            end_position, 
            file_data, 
            "invalid binary operation", 
            format!("expected a number type, found {}", number1), 
            EXCEPTION
        )
    }
}

pub struct TokenBinaryOperationConversion;

impl TokenBinaryOperationConversion {
    pub fn new<'a>(token: &'a Token, file_data: &'a FileData) -> PositionException<'a> {
        PositionException::new(
            token.start_position.clone(),
            token.end_position.clone(), 
            file_data, 
            "binary operator conversion",
            format!("failed to convert token ({}) to a binary operator", token.kind),
            EXCEPTION
        )
    }
}

pub struct TokenLiteralConversion;

impl TokenLiteralConversion {
    pub fn new<'a>(token: &'a Token, file_data: &'a FileData) -> PositionException<'a> {
        PositionException::new(
            token.start_position.clone(),
            token.end_position.clone(), 
            file_data, 
            "unexpected token",
            format!("failed to convert token ({}) to a literal;\nexpected a string, bool, integer or a float", token.kind),
            EXCEPTION
        )
    }
}

pub struct TokenUnaryOperatorConversion;

impl TokenUnaryOperatorConversion {
    pub fn new<'a>(token: &'a Token, file_data: &'a FileData) -> PositionException<'a> {
        PositionException::new(
            token.start_position.clone(),
            token.end_position.clone(), 
            file_data, 
            "unary operator conversion",
            format!("failed to convert token ({}) to a unary operator; expected minus or plus", token.kind),
            EXCEPTION
        )
    }
}

pub struct DivisionByZero;

impl DivisionByZero {
    pub fn new<'a>(start_position: Position, end_position: Position, file_data: &'a FileData) -> PositionException<'a> {
        PositionException::new(
            start_position,
            end_position,
            file_data, 
            "attempted to divide by zero",
            "can not divide integers by zero".to_string(),
            EXCEPTION
        )
    }
}

pub struct AccessUndeclaredVariable;

impl AccessUndeclaredVariable {
    pub fn new<'a>(start_position: Position, end_position: Position, file_data: &'a FileData, variable_identifier: &String) -> PositionException<'a> {
        PositionException::new(
            start_position,
            end_position,
            file_data, 
            "undeclared variable",
            format!("can't find a variable with the name {} in the current scope", variable_identifier),
            EXCEPTION
        )
    }
}

pub struct AccessUndeclaredFunction;

impl AccessUndeclaredFunction {
    pub fn new<'a>(start_position: Position, end_position: Position, file_data: &'a FileData, variable_identifier: &String) -> PositionException<'a> {
        PositionException::new(
            start_position,
            end_position,
            file_data, 
            "undeclared function",
            format!("can't find a function with the name {} in the current scope", variable_identifier),
            EXCEPTION
        )
    }
}

pub struct InvalidAmountOfArguments;

impl InvalidAmountOfArguments {
    pub fn new<'a>(start_position: Position, end_position: Position, file_data: &'a FileData, expected: usize, found: usize) -> PositionException<'a> {
        PositionException::new(
            start_position,
            end_position,
            file_data, 
            "invalid amount of arguments",
            format!("expected {} argument(s), found {}", expected, found),
            EXCEPTION
        )
    }
}