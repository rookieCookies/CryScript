#![allow(unused)]
use crate::{lexer::tokens::{Token, TokenKind}, exceptions::{interpreter_errors::{InvalidBinaryOperationNumbers, TokenBinaryOperationConversion, DivisionByZero}, Exception}, utils::{Position, FileData}};

use super::{BinaryOperator, Literal};

impl BinaryOperator {
    pub fn compute(&self, file_data: &FileData, (start, end): (Position, Position), (value1, value2): (Literal, Literal)) -> Literal {
        match self {
            BinaryOperator::Add => self.add(value1, value2, (&start, &end, file_data)),
            BinaryOperator::Subtract => self.subtract(value1, value2, (&start, &end, file_data)),
            BinaryOperator::Multiply => self.multiply(value1, value2, (&start, &end, file_data)),
            BinaryOperator::Divide => self.divide(value1, value2, (&start, &end, file_data)),
            BinaryOperator::Power => self.power(value1, value2, (&start, &end, file_data)),
            _ => match (&value1, &value2) {
                (Literal::Integer(n1), Literal::Integer(n2)) => Literal::Integer(self.compute_binary_operation(n1, n2)),
                (Literal::Integer(n1), Literal::Float(n2)) => Literal::Float(self.compute_binary_operation(&(*n1 as f32), n2)),
                (Literal::Float(n1), Literal::Integer(n2)) => Literal::Float(self.compute_binary_operation(n1, &(*n2 as f32))),
                (Literal::Float(n1), Literal::Float(n2)) => Literal::Float(self.compute_binary_operation(n1, n2)),
                (Literal::String(n1), Literal::String(n2)) => Literal::Bool(n1 == n2),
                _ => InvalidBinaryOperationNumbers::new(start.clone(), end.clone(), file_data, value1, value2).run()
            },
            
        }
    }

    fn compute_binary_operation<T: Number + Comparable + Clone>
    (&self, n1: &T, n2: &T) -> T {
        match self {
            BinaryOperator::Add => n1.add(n2).clone(),
            BinaryOperator::Subtract => n1.rem(n2).clone(),
            BinaryOperator::Multiply => n1.mul(n2).clone(),
            BinaryOperator::Divide => n1.div(n2).clone(),
            BinaryOperator::Power => n1.pow(n2).clone(),
            BinaryOperator::EqualsTo => n1.equals(n2),
            BinaryOperator::NotEqualsTo => n1.not_equals(n2),
            BinaryOperator::GreaterThan => n1.greater(n2),
            BinaryOperator::LesserThan => n1.smaller(n2),
            BinaryOperator::GreaterEquals => n1.greater_equals(n2),
            BinaryOperator::LesserEquals => n1.smaller_equals(n2),
            BinaryOperator::And => T::from_bool(n1.to_bool() && n2.to_bool()),
            BinaryOperator::Or => T::from_bool(n1.to_bool() || n2.to_bool()),
            BinaryOperator::Nothing => n2.clone(),
        }
    }

    pub fn from_token(token: &Token, file_data: &FileData) -> Self {
        Self::from_token_kind(token.kind.clone(), (token.start_position.clone(), token.end_position.clone()), file_data)
    }

    pub fn from_token_kind(token_kind: TokenKind, (start, end): (Position, Position), file_data: &FileData) -> Self {
        match token_kind {
            TokenKind::Equals                                => BinaryOperator::Nothing,
            TokenKind::Plus      | TokenKind::PlusEquals     => BinaryOperator::Add,
            TokenKind::Minus     | TokenKind::MinusEquals    => BinaryOperator::Subtract,
            TokenKind::Multiply  | TokenKind::MultiplyEquals => BinaryOperator::Multiply,
            TokenKind::Slash     | TokenKind::DivideEquals   => BinaryOperator::Divide,
            TokenKind::Power     | TokenKind::PowerEquals    => BinaryOperator::Power,
            TokenKind::EqualsTo                              => BinaryOperator::EqualsTo,
            TokenKind::NotEquals                             => BinaryOperator::NotEqualsTo,
            TokenKind::RightAngle                            => BinaryOperator::GreaterThan,
            TokenKind::LeftAngle                             => BinaryOperator::LesserThan,
            TokenKind::GreaterEquals                         => BinaryOperator::GreaterEquals,
            TokenKind::SmallerEquals                         => BinaryOperator::LesserEquals,
            TokenKind::And                                   => BinaryOperator::And,
            TokenKind::Or                                    => BinaryOperator::Or,
            _ => TokenBinaryOperationConversion::new(&Token::new(start, end, token_kind), file_data).run()
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Power => "^",
            BinaryOperator::EqualsTo => "==",
            BinaryOperator::NotEqualsTo => "!=",
            BinaryOperator::GreaterThan => ">",
            BinaryOperator::LesserThan => "<",
            BinaryOperator::GreaterEquals => ">=",
            BinaryOperator::LesserEquals => "<=",
            BinaryOperator::And => "&&",
            BinaryOperator::Or => "||",
            BinaryOperator::Nothing => "=",
        }.to_string()
    }

    fn add(&self, n1: Literal, n2: Literal, (start, end, file_data): (&Position, &Position, &FileData)) -> Literal {
        match (&n1, &n2) {
            (Literal::Integer(n1), Literal::Integer(n2)) => Literal::Integer(n1.add(n2)),
            (Literal::Integer(n1), Literal::Float(n2)) => Literal::Float((*n1 as f32).add(n2)),
            (Literal::Float(n1), Literal::Integer(n2)) => Literal::Float(n1.add(&(*n2 as f32))),
            (Literal::Float(n1), Literal::Float(n2)) => Literal::Float(n1.add(n2)),
            (Literal::Integer(v), Literal::String(s)) => Literal::String(format!("{}{}", v, s)),
            (Literal::Float(v), Literal::String(s)) => Literal::String(format!("{}{}", v, s)),
            (Literal::String(s), Literal::Integer(v)) => Literal::String(format!("{}{}", s, v)),
            (Literal::String(s), Literal::Float(v)) => Literal::String(format!("{}{}", s, v)),
            (Literal::String(s), Literal::String(v)) => Literal::String(format!("{}{}", s, v)),
            (Literal::String(s), Literal::Bool(v)) => Literal::String(format!("{}{}", s, v)),
            _ => InvalidBinaryOperationNumbers::new(start.clone(), end.clone(), file_data, n1, n2).run(),
        }
    }
    fn subtract(&self, n1: Literal, n2: Literal, (start, end, file_data): (&Position, &Position, &FileData)) -> Literal {
        match (&n1, &n2) {
            (Literal::Integer(n1), Literal::Integer(n2)) => Literal::Integer(n1.rem(n2)),
            (Literal::Integer(n1), Literal::Float(n2)) => Literal::Float((*n1 as f32).rem(n2)),
            (Literal::Float(n1), Literal::Integer(n2)) => Literal::Float(n1.rem(&(*n2 as f32))),
            (Literal::Float(n1), Literal::Float(n2)) => Literal::Float(n1.rem(n2)),
            _ => InvalidBinaryOperationNumbers::new(start.clone(), end.clone(), file_data, n1, n2).run()
        }
    }
    fn multiply(&self, n1: Literal, n2: Literal, (start, end, file_data): (&Position, &Position, &FileData)) -> Literal {
        match (&n1, &n2) {
            (Literal::Integer(n1), Literal::Integer(n2)) => Literal::Integer(n1.mul(n2)),
            (Literal::Integer(n1), Literal::Float(n2)) => Literal::Float((*n1 as f32).mul(n2)),
            (Literal::Float(n1), Literal::Integer(n2)) => Literal::Float(n1.mul(&(*n2 as f32))),
            (Literal::Float(n1), Literal::Float(n2)) => Literal::Float(n1.mul(n2)),
            _ => InvalidBinaryOperationNumbers::new(start.clone(), end.clone(), file_data, n1, n2).run()
        }
    }
    fn divide(&self, n1: Literal, n2: Literal, (start, end, file_data): (&Position, &Position, &FileData)) -> Literal {
        match (&n1, &n2) {
            (Literal::Integer(n1), Literal::Integer(n2)) => Literal::Integer(n1.div(n2)),
            (Literal::Integer(n1), Literal::Float(n2)) => Literal::Float((*n1 as f32).div(n2)),
            (Literal::Float(n1), Literal::Integer(n2)) => Literal::Float(n1.div(&(*n2 as f32))),
            (Literal::Float(n1), Literal::Float(n2)) => Literal::Float(n1.div(n2)),
            _ => InvalidBinaryOperationNumbers::new(start.clone(), end.clone(), file_data, n1, n2).run()
        }
    }
    fn power(&self, n1: Literal, n2: Literal, (start, end, file_data): (&Position, &Position, &FileData)) -> Literal {
        match (&n1, &n2) {
            (Literal::Integer(n1), Literal::Integer(n2)) => Literal::Integer(n1.pow(n2)),
            (Literal::Integer(n1), Literal::Float(n2)) => Literal::Float((*n1 as f32).pow(n2)),
            (Literal::Float(n1), Literal::Integer(n2)) => Literal::Float(n1.pow(&(*n2 as f32))),
            (Literal::Float(n1), Literal::Float(n2)) => Literal::Float(n1.pow(n2)),
            _ => InvalidBinaryOperationNumbers::new(start.clone(), end.clone(), file_data, n1, n2).run()
        }
    }
}

trait Number: Sized {
    #[inline(always)]
    fn add(&self, x: &Self) -> Self;
    #[inline(always)]
    fn rem(&self, x: &Self) -> Self;
    #[inline(always)]
    fn mul(&self, x: &Self) -> Self;
    #[inline(always)]
    fn div(&self, x: &Self) -> Self;
    #[inline(always)]
    fn pow(&self, x: &Self) -> Self;
    #[inline(always)]
    fn n_abs(&self) -> Self;
}

trait Comparable: Sized {
    #[inline(always)]
    fn equals(&self, x: &Self) -> Self;
    #[inline(always)]
    fn greater(&self, x: &Self) -> Self;
    
    #[inline(always)]
    fn bool_true() -> Self;
    #[inline(always)]
    fn bool_false() -> Self;
    #[inline(always)]
    fn to_bool(&self) -> bool;
    #[inline(always)]
    fn from_bool(v: bool) -> Self {
        if v { Self::bool_true() } else { Self::bool_false() }
    }
    #[inline(always)]
    fn not_equals(&self, x: &Self) -> Self {
        Self::from_bool(!self.equals(x).to_bool())
    }
    #[inline(always)]
    fn greater_equals(&self, x: &Self) -> Self {
        Self::from_bool(self.greater(x).to_bool() || self.equals(x).to_bool())
    }

    #[inline(always)]
    fn smaller(&self, x: &Self) -> Self {
        Self::from_bool(!self.greater_equals(x).to_bool())
    }

    #[inline(always)]
    fn smaller_equals(&self, x: &Self) -> Self {
        Self::from_bool(self.smaller(x).to_bool() || self.equals(x).to_bool())
    }
}

impl Number for i32 {
    fn add(&self, x: &Self) -> Self {
        self + x
    }

    fn rem(&self, x: &Self) -> Self {
        self - x
    }

    fn mul(&self, x: &Self) -> Self {
        self * x
    }

    fn div(&self, x: &Self) -> Self {
        self / x
    }

    fn pow(&self, x: &Self) -> Self {
        if *x == x.n_abs() {
            self.wrapping_pow(x.n_abs().try_into().unwrap())
        } else {
            if *self == 1 {
                1
            } else {
                0
            }
        }
    }

    fn n_abs(&self) -> Self {
        self.abs()
    }
}

impl Comparable for i32 {
    fn bool_true() -> Self {
        1
    }

    fn bool_false() -> Self {
        0
    }

    fn to_bool(&self) -> bool {
        self >= &Self::bool_true()
    }
    
    fn equals(&self, x: &Self) -> Self {
        Self::from_bool(self == x)
    }

    fn greater(&self, x: &Self) -> Self {
        Self::from_bool(self > x)
    }
}

impl Number for f32 {
    fn add(&self, x: &Self) -> Self {
        self + x
    }

    fn rem(&self, x: &Self) -> Self {
        self - x
    }

    fn mul(&self, x: &Self) -> Self {
        self * x
    }

    fn div(&self, x: &Self) -> Self {
        self / x
    }

    fn pow(&self, x: &Self) -> Self {
        self.powf(*x)
    }

    fn n_abs(&self) -> Self {
        self.abs()
    }
}

impl Comparable for f32 {
    fn bool_true() -> Self {
        1.
    }

    fn bool_false() -> Self {
        0.
    }

    fn to_bool(&self) -> bool {
        self >= &Self::bool_true()
    }
    
    fn equals(&self, x: &Self) -> Self {
        Self::from_bool(self == x)
    }

    fn greater(&self, x: &Self) -> Self {
        Self::from_bool(self > x)
    }
}