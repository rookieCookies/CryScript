use std::fmt::Display;

use utils::wrap;

use crate::{
    exceptions::{parser_exceptions::UnexpectedToken, Exception},
    interpreter::DataRef,
    lexer::token::{Token, TokenType},
    parser::data::{Data, DataType},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub(crate) enum UnaryOperator {
    Minus,
    Plus,
    Not,
}

impl UnaryOperator {
    pub(crate) fn operate(&self, n: DataRef, data: &Data) -> Result<Data, Exception> {
        Ok(match self {
            UnaryOperator::Minus => {
                let rhs = wrap(Data::new(
                    data.file_data.clone(),
                    data.start.clone(),
                    data.end.clone(),
                    DataType::Integer(-1),
                ));
                let borrowed = &*rhs.borrow();
                Data::mul(n, rhs.clone(), data, borrowed)?
            }
            UnaryOperator::Plus => {
                let rhs = wrap(Data::new(
                    data.file_data.clone(),
                    data.start.clone(),
                    data.end.clone(),
                    DataType::Integer(0),
                ));
                let borrowed = &*rhs.borrow();
                Data::add(n, rhs.clone(), data, borrowed)?
            }
            UnaryOperator::Not => Data::new(
                data.file_data.clone(),
                data.start.clone(),
                data.end.clone(),
                (!data.as_bool()?).into(),
            ),
        })
    }
}

impl From<&Token> for UnaryOperator {
    fn from(value: &Token) -> Self {
        match value.token_type {
            TokenType::Plus => UnaryOperator::Plus,
            TokenType::Minus => UnaryOperator::Minus,
            TokenType::ExclamationMark => UnaryOperator::Not,
            _ => UnexpectedToken::call(
                value.fetch(),
                "\'+\', \'-\', or \'!\'",
                value.token_type.to_string().as_str(),
            ),
        }
    }
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UnaryOperator::Minus => "minus",
                UnaryOperator::Plus => "plus",
                UnaryOperator::Not => "not",
            }
        )
    }
}
