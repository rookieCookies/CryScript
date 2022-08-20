use std::{fmt::Display, rc::Rc};

use crate::{exceptions::parser_exceptions::NotATypeHint, lexer::token::Token, FileData, Position};

#[derive(Debug, Clone)]
pub struct Type {
    pub(crate) type_value: TypeHint,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) file_data: Rc<FileData>,
}

impl Type {
    pub(crate) fn new(
        type_value: TypeHint,
        start: Position,
        end: Position,
        file_data: Rc<FileData>,
    ) -> Self {
        Self {
            type_value,
            start,
            end,
            file_data,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum TypeHint {
    Integer,
    String,
    Float,
    Class(String),
    None,
}

impl From<&Token> for Type {
    fn from(tkn: &Token) -> Self {
        Self::new(
            match &tkn.token_type {
                crate::lexer::token::TokenType::TypeHint(v) => match v {
                    crate::lexer::token::TypeHintToken::Integer => TypeHint::Integer,
                    crate::lexer::token::TypeHintToken::Float => TypeHint::Float,
                    crate::lexer::token::TypeHintToken::String => TypeHint::String,
                },
                crate::lexer::token::TokenType::Identifier(v) => TypeHint::Class(v.clone()),
                _ => NotATypeHint::call(&tkn.start, &tkn.end, &tkn.file_data, &tkn.token_type),
            },
            tkn.start.clone(),
            tkn.end.clone(),
            tkn.file_data.clone(),
        )
    }
}

impl Display for TypeHint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TypeHint::Integer => "integer",
                TypeHint::String => "string",
                TypeHint::Float => "float",
                TypeHint::Class(v) => v.as_str(),
                TypeHint::None => "none",
            }
        )
    }
}
