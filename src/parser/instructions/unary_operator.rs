#![allow(unused)]
use crate::{lexer::tokens::{Token, TokenKind}, exceptions::{interpreter_errors::TokenUnaryOperatorConversion, Exception}, utils::FileData};

use super::UnaryOperator;

impl UnaryOperator {
    pub fn from_token(token: &Token, file_data: &FileData) -> UnaryOperator {
        match token.kind {
            TokenKind::Plus => UnaryOperator::Plus,
            TokenKind::Minus => UnaryOperator::Minus,
            _ => TokenUnaryOperatorConversion::new(token, file_data).run(),
        }
    }
}