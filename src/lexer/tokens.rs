#![allow(unused)]
use std::fmt::Display;

use crate::utils::Position;

#[derive(Debug)]
pub struct Token {
    pub start_position: Position,
    pub end_position: Position,
    pub kind: TokenKind
}

impl Token {
    pub fn new(start_position: Position, end_position: Position, kind: TokenKind) -> Self { Self { start_position, end_position, kind } }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    // Single Character
    Plus,
    Minus,
    Multiply,
    Slash,
    BackSlash,
    Power,
    Equals,
    Dot,
    Comma,
    Underscore,
    ExclamationMark,
    Ampersand,
    Bar,
    Colon,
    DoubleColon,
    SemiColon,

    // Brackets
    LeftAngle,
    RightAngle,
    LeftSquare,
    RightSquare,
    LeftParenthesis,
    RightParenthesis,

    // Multiple Characters
    Integer(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Identifier(String),
    Let,
    Function,
    Struct,
    If,
    Else,
    Use,
    While,
    Return,

    // Operators
    And,
    Or,
    EqualsTo,
    NotEquals,
    GreaterEquals,
    SmallerEquals,

    // Operators With Operations (hehe)
    PlusEquals,
    MinusEquals,
    MultiplyEquals,
    DivideEquals,
    PowerEquals,

    // Misc,
    Error,
    NewLine,
    EndOfFile,
    Indent,
    Dedent,
    Null,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            TokenKind::Plus => "+".to_string(),
            TokenKind::Minus => "-".to_string(),
            TokenKind::Multiply => "*".to_string(),
            TokenKind::Slash => "/".to_string(),
            TokenKind::BackSlash => "\\".to_string(),
            TokenKind::Power => "^".to_string(),
            TokenKind::Equals => "=".to_string(),
            TokenKind::PlusEquals => "+=".to_string(),
            TokenKind::MinusEquals => "-=".to_string(),
            TokenKind::MultiplyEquals => "*=".to_string(),
            TokenKind::DivideEquals => "/=".to_string(),
            TokenKind::PowerEquals => "^=".to_string(),
            TokenKind::Dot => ".".to_string(),
            TokenKind::Comma => ",".to_string(),
            TokenKind::Underscore => "_".to_string(),
            TokenKind::ExclamationMark => "!".to_string(),
            TokenKind::Ampersand => "&".to_string(),
            TokenKind::Bar => "|".to_string(),
            TokenKind::Colon => ":".to_string(),
            TokenKind::DoubleColon => "::".to_string(),
            TokenKind::SemiColon => ";".to_string(),
            TokenKind::LeftAngle => "<".to_string(),
            TokenKind::RightAngle => ">".to_string(),
            TokenKind::LeftSquare => "[".to_string(),
            TokenKind::RightSquare => "]".to_string(),
            TokenKind::LeftParenthesis => "(".to_string(),
            TokenKind::RightParenthesis => ")".to_string(),
            TokenKind::Integer(value) => format!("{}", value),
            TokenKind::Float(value) => format!("{}", value),
            TokenKind::String(value) => format!("{}", value),
            TokenKind::Identifier(value) => format!("{}", value),
            TokenKind::Bool(value) => format!("{}", value),
            TokenKind::Let => "let".to_string(),
            TokenKind::Function => "function".to_string(),
            TokenKind::Struct => "struct".to_string(),
            TokenKind::If => "if".to_string(),
            TokenKind::Else => "else".to_string(),
            TokenKind::Use => "use".to_string(),
            TokenKind::Return => "return".to_string(),
            TokenKind::While => "while".to_string(),
            TokenKind::And => "&&".to_string(),
            TokenKind::Or => "||".to_string(),
            TokenKind::EqualsTo => "==".to_string(),
            TokenKind::NotEquals => "!=".to_string(),
            TokenKind::GreaterEquals => ">=".to_string(),
            TokenKind::SmallerEquals => "<=".to_string(),
            TokenKind::Error => "error!".to_string(),
            TokenKind::NewLine => "\\n".to_string(),
            TokenKind::EndOfFile => "end of file".to_string(),
            TokenKind::Indent => "indent".to_string(),
            TokenKind::Dedent => "dedent".to_string(),
            TokenKind::Null => "null".to_string(),
        })
    }
}