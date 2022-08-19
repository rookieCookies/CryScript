use std::{fmt::Display, rc::Rc};

use crate::{Annotation, FileData, Keyword, Position};

#[derive(Clone)]
pub(crate) struct Token {
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) token_type: TokenType,
    pub(crate) file_data: Rc<FileData>,
}

impl Token {
    pub(crate) fn new(
        start: Position,
        end: Position,
        token_type: TokenType,
        file_data: Rc<FileData>,
    ) -> Self {
        Self {
            start,
            end,
            token_type,
            file_data,
        }
    }

    pub(crate) fn fetch(&self) -> (&Position, &Position, &Rc<FileData>) {
        (&self.start, &self.end, &self.file_data)
    }
}

#[derive(PartialEq, Clone)]
pub(crate) enum TokenType {
    Plus,
    Minus,
    Multiply,
    Slash,
    Backslash,
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
    Keyword(Keyword),
    TypeHint(TypeHintToken),
    Annotation(Annotation),

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

impl TokenType {
    pub fn token_type(&self) -> String {
        match self {
            TokenType::Plus => "plus",
            TokenType::Minus => "minus",
            TokenType::Multiply => "multiply",
            TokenType::Slash => "slash",
            TokenType::Backslash => "backslash",
            TokenType::Power => "power",
            TokenType::Equals => "equals",
            TokenType::Dot => "dot",
            TokenType::Comma => "comma",
            TokenType::Underscore => "underscore",
            TokenType::ExclamationMark => "exclamation_mark",
            TokenType::Ampersand => "ampersand",
            TokenType::Bar => "bar",
            TokenType::Colon => "colon",
            TokenType::DoubleColon => "double_colon",
            TokenType::SemiColon => "semi_colon",
            TokenType::LeftAngle => "left_angle",
            TokenType::RightAngle => "right_angle",
            TokenType::LeftSquare => "left_square",
            TokenType::RightSquare => "right_square",
            TokenType::LeftParenthesis => "left_parenthesis",
            TokenType::RightParenthesis => "right_parenthesis",
            TokenType::Integer(_) => "integer",
            TokenType::Float(_) => "float",
            TokenType::String(_) => "string",
            TokenType::Bool(_) => "bool",
            TokenType::Identifier(_) => "identifier",
            TokenType::Keyword(_) => "keyword",
            TokenType::And => "and",
            TokenType::Or => "or",
            TokenType::EqualsTo => "equals_to",
            TokenType::NotEquals => "not_equals",
            TokenType::GreaterEquals => "greater_equals",
            TokenType::SmallerEquals => "smaller_equals",
            TokenType::PlusEquals => "plus_equals",
            TokenType::MinusEquals => "minus_equals",
            TokenType::MultiplyEquals => "multiply_equals",
            TokenType::DivideEquals => "divide_equals",
            TokenType::PowerEquals => "power_equals",
            TokenType::Error => "error",
            TokenType::NewLine => "new_line",
            TokenType::EndOfFile => "end_of_file",
            TokenType::Indent => "indent",
            TokenType::Dedent => "dedent",
            TokenType::Null => "null",
            TokenType::TypeHint(_) => "type_hint",
            TokenType::Annotation(_) => "annotation",
        }
        .to_string()
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TokenType::Plus => "+".to_string(),
                TokenType::Minus => "-".to_string(),
                TokenType::Multiply => "*".to_string(),
                TokenType::Slash => "/".to_string(),
                TokenType::Backslash => "\\".to_string(),
                TokenType::Power => "^".to_string(),
                TokenType::Equals => "=".to_string(),
                TokenType::PlusEquals => "+=".to_string(),
                TokenType::MinusEquals => "-=".to_string(),
                TokenType::MultiplyEquals => "*=".to_string(),
                TokenType::DivideEquals => "/=".to_string(),
                TokenType::PowerEquals => "^=".to_string(),
                TokenType::Dot => ".".to_string(),
                TokenType::Comma => ",".to_string(),
                TokenType::Underscore => "_".to_string(),
                TokenType::ExclamationMark => "!".to_string(),
                TokenType::Ampersand => "&".to_string(),
                TokenType::Bar => "|".to_string(),
                TokenType::Colon => ":".to_string(),
                TokenType::DoubleColon => "::".to_string(),
                TokenType::SemiColon => ";".to_string(),
                TokenType::LeftAngle => "<".to_string(),
                TokenType::RightAngle => ">".to_string(),
                TokenType::LeftSquare => "[".to_string(),
                TokenType::RightSquare => "]".to_string(),
                TokenType::LeftParenthesis => "(".to_string(),
                TokenType::RightParenthesis => ")".to_string(),
                TokenType::Integer(value) => value.to_string(),
                TokenType::Float(value) => value.to_string(),
                TokenType::String(value) => value.to_string(),
                TokenType::Identifier(value) => value.to_string(),
                TokenType::Keyword(value) => format!("{}", value),
                TokenType::Bool(value) => value.to_string(),
                TokenType::And => "&&".to_string(),
                TokenType::Or => "||".to_string(),
                TokenType::EqualsTo => "==".to_string(),
                TokenType::NotEquals => "!=".to_string(),
                TokenType::GreaterEquals => ">=".to_string(),
                TokenType::SmallerEquals => "<=".to_string(),
                TokenType::Error => "error!".to_string(),
                TokenType::NewLine => "\\n".to_string(),
                TokenType::EndOfFile => "end of file".to_string(),
                TokenType::Indent => "indent".to_string(),
                TokenType::Dedent => "dedent".to_string(),
                TokenType::Null => "null".to_string(),
                TokenType::TypeHint(v) => v.to_string(),
                TokenType::Annotation(_) => "annotation".to_string(),
            }
        )
    }
}

#[derive(PartialEq, Clone)]
pub(crate) enum TypeHintToken {
    Integer,
    Float,
    String,
}

impl Display for TypeHintToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TypeHintToken::Integer => "integer",
                TypeHintToken::Float => "float",
                TypeHintToken::String => "string",
            }
        )
    }
}
