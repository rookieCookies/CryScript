use std::fmt::Display;

use crate::{
    exceptions::{parser_exceptions::UnexpectedToken, Exception},
    lexer::token::{Token, TokenType},
    parser::data::{Data, DataType},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub(crate) enum BinaryOperator {
    Add,
    Remove,
    Multiply,
    Divide,
    Power,

    AddAssign,
    RemoveAssign,
    MultiplyAssign,
    DivideAssign,
    PowerAssign,

    EqualsTo,
    NotEquals,
    GreaterEquals,
    GreaterThan,
    LesserEquals,
    LesserThan,
    And,
    Or,
}

impl BinaryOperator {
    pub(crate) fn operate(
        &self,
        n1: &Data,
        n2: &Data,
        data1: &Data,
        data2: &Data,
    ) -> Result<Data, Exception> {
        match self {
            BinaryOperator::Add => Data::add(n1, n2, data1, data2),
            BinaryOperator::Remove => Data::sub(n1, n2, data1, data2),
            BinaryOperator::Multiply => Data::mul(n1, n2, data1, data2),
            BinaryOperator::Divide => Data::div(n1, n2, data1, data2),
            BinaryOperator::Power => Data::pow(n1, n2, data1, data2),

            BinaryOperator::EqualsTo => Ok(data1.eq(&n2)?),
            BinaryOperator::NotEquals => Ok(data1.ne(&n2)?),
            BinaryOperator::GreaterEquals => Ok(data1.geq(&n2)?),
            BinaryOperator::GreaterThan => Ok(data1.gt(&n2)?),
            BinaryOperator::LesserEquals => Ok(data1.leq(&n2)?),
            BinaryOperator::LesserThan => Ok(data1.lt(&n2)?),
            BinaryOperator::And => Ok(Data::new(
                data1.file_data.clone(),
                data1.start.clone(),
                data2.end.clone(),
                if data1.as_bool()? && data2.as_bool()? {
                    DataType::Integer(1)
                } else {
                    DataType::Integer(0)
                },
            )),
            BinaryOperator::Or => Ok(Data::new(
                data1.file_data.clone(),
                data1.start.clone(),
                data2.end.clone(),
                if data1.as_bool()? || data2.as_bool()? {
                    DataType::Integer(1)
                } else {
                    DataType::Integer(0)
                },
            )),
            _ => panic!(),
        }
    }

    pub(crate) fn from(value: &Token, expected: &[TokenType]) -> Self {
        match value.token_type {
            TokenType::Plus => BinaryOperator::Add,
            TokenType::Minus => BinaryOperator::Remove,
            TokenType::Multiply => BinaryOperator::Multiply,
            TokenType::Slash => BinaryOperator::Divide,
            TokenType::Power => BinaryOperator::Power,

            TokenType::PlusEquals => BinaryOperator::AddAssign,
            TokenType::MinusEquals => BinaryOperator::RemoveAssign,
            TokenType::MultiplyEquals => BinaryOperator::MultiplyAssign,
            TokenType::DivideEquals => BinaryOperator::DivideAssign,
            TokenType::PowerEquals => BinaryOperator::PowerAssign,

            TokenType::EqualsTo => BinaryOperator::EqualsTo,
            TokenType::NotEquals => BinaryOperator::NotEquals,
            TokenType::GreaterEquals => BinaryOperator::GreaterEquals,
            TokenType::RightAngle => BinaryOperator::GreaterThan,
            TokenType::SmallerEquals => BinaryOperator::LesserEquals,
            TokenType::LeftAngle => BinaryOperator::LesserThan,

            TokenType::And => BinaryOperator::And,
            TokenType::Or => BinaryOperator::Or,
            _ => {
                let mut expected_message = String::new();
                for (index, value) in expected.iter().enumerate() {
                    expected_message.push_str(
                        format!(
                            "{}{}",
                            value,
                            if expected.len() > 2 && index == expected.len() - 2 {
                                ", or "
                            } else if index == expected.len() - 1 {
                                ""
                            } else {
                                ", "
                            }
                        )
                        .as_str(),
                    )
                }
                UnexpectedToken::call(
                    value.fetch(),
                    &expected_message,
                    value.token_type.to_string().as_str(),
                )
            }
        }
    }
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BinaryOperator::Add => "add",
                BinaryOperator::Remove => "remove",
                BinaryOperator::Multiply => "multiply",
                BinaryOperator::Divide => "divide",
                BinaryOperator::EqualsTo => "equals",
                BinaryOperator::NotEquals => "not equals",
                BinaryOperator::GreaterEquals => "greater equals",
                BinaryOperator::GreaterThan => "greater",
                BinaryOperator::LesserEquals => "lesser equals",
                BinaryOperator::LesserThan => "lesser",
                BinaryOperator::And => "and",
                BinaryOperator::Or => "or",
                BinaryOperator::Power => "power",
                BinaryOperator::AddAssign => "add assign",
                BinaryOperator::RemoveAssign => "remove assign",
                BinaryOperator::MultiplyAssign => "multiply assign",
                BinaryOperator::DivideAssign => "divide assign",
                BinaryOperator::PowerAssign => "power assign",
            }
        )
    }
}
