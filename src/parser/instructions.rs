#![allow(unused)]
use std::{fmt::Display, ops::Deref, rc::Rc, cell::{RefMut, RefCell}};

use crate::{utils::{Position, read_file, FileData}, lexer::{tokens::{Token, TokenKind}, Lexer}, exceptions::{interpreter_errors::TokenLiteralConversion, Exception}, run};

use super::{context::Context, functions::Function, Parser};

pub mod unary_operator;
pub mod binary_operation;

#[derive(Debug, Clone)]
pub enum InstructionKind {
    BinaryOperation {
        left: Box<Instruction>,
        right: Box<Instruction>,
        operation: BinaryOperator
    },
    UnaryOperation {
        operator: UnaryOperator,
        value: Box<Instruction>,
    },
    VarAssign {
        identifier: String,
        value: Box<Instruction>,
    },
    VarUpdate {
        identifier: String,
        value: Box<Instruction>,
    },
    VarAccess {
        identifier: String,
    },
    SystemOut {
        value: Box<Instruction>,
    },
    If {
        condition: Option<Box<Instruction>>,
        body: Vec<Instruction>,
        if_else: Option<Box<Instruction>>,
    },
    While {
        condition:Box<Instruction>,
        body: Vec<Instruction>,
    },
    DeclareFunction {
        identifier: String,
        args: Vec<String>,
        body: Vec<Instruction>
    },
    CallFunction {
        identifier: String,
        args: Vec<Instruction>,
    },
    Use {
        file_path: String,
    },
    Return {
        value: Box<Instruction>,
    },
    Indent,
    Dedent,
    Lit(Literal)
}

#[derive(Clone, Debug)]
pub enum Literal {
    Integer(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Null
}

impl Literal {
    pub fn from_token(token: &Token, file_data: &FileData) -> Self {
        match &token.kind {
            TokenKind::Integer(value) => Literal::Integer(*value),
            TokenKind::Float(value) => Literal::Float(*value),
            TokenKind::String(value) => Literal::String(value.clone()),
            TokenKind::Bool(value) => Literal::Bool(*value),
            _ => TokenLiteralConversion::new(token, file_data).run()
        }
    }

    pub fn bool(value: f32) -> Self {
        Literal::Bool(value >= 1.)
    }

    pub fn to_bool(&self) -> bool {
        match self {
            Literal::Integer(v) => v >= &1,
            Literal::Float(v) => v >= &1.,
            Literal::String(_) => panic!(),
            Literal::Bool(v) => *v,
            Literal::Null => false,
        }
    }
    pub fn kind(&self) -> String {
        match self {
            Literal::Integer(v) => "integer".to_string(),
            Literal::Float(v) => "float".to_string(),
            Literal::String(v) => "string".to_string(),
            Literal::Bool(v) => "bool".to_string(),
            Literal::Null => "null".to_string()
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Literal::Integer(v) => v.to_string(),
                Literal::Float(v) => v.to_string(),
                Literal::String(v) => v.to_string(),
                Literal::Bool(v) => v.to_string(),
                Literal::Null => "null".to_string()
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub start_position: Position,
    pub end_position: Position,
    pub kind: InstructionKind,
}

impl<'a> Instruction {
    pub fn new(start_position: Position, end_position: Position, kind: InstructionKind) -> Self { Self { start_position, end_position, kind } }

    pub fn visit(&self, file_data: &FileData, context: Rc<RefCell<Context>>) -> Literal {
        match &self.kind {
            InstructionKind::BinaryOperation { left, right, operation } => 
                operation.compute(
                    file_data, 
                    (
                        left.start_position.clone(), 
                        right.end_position.clone()
                    ), 
                    (
                        left.visit(file_data, context.clone()), 
                        right.visit(file_data, context.clone())
                    )
                ),
            InstructionKind::Lit(v) => v.clone(),
            InstructionKind::UnaryOperation { operator, value } => {
                let literal = value.visit(file_data, context);
                match operator {
                    UnaryOperator::Minus => BinaryOperator::Multiply.compute(
                        file_data,
                        (
                            value.start_position.clone(), 
                            value.end_position.clone()
                        ), 
                        (
                            literal, 
                            Literal::Integer(-1)
                        )
                    ),
                    UnaryOperator::Plus => literal,
                    UnaryOperator::Not => if literal.to_bool() { Literal::Integer(0) } else { { Literal::Integer(1) } },
                }
            },
            InstructionKind::VarAssign { identifier, value } => {
                let value = (**value).visit(file_data, context.clone()).clone();
                context.borrow_mut().symbol_table.set(
                    identifier.to_string(), 
                    value
                );
                Literal::Null
            },
            InstructionKind::VarUpdate { identifier, value } => {
                let value = (**value).visit(file_data, context.clone()).clone();
                context.borrow_mut().update_var(
                    identifier, 
                    value,
                    &self.start_position,
                    &self.end_position,
                    file_data
                );
                Instruction::new(self.start_position.clone(), self.end_position.clone(), InstructionKind::VarAccess { identifier: identifier.clone() }).visit(file_data, context)
            },
            InstructionKind::VarAccess { identifier } => 
                context
                    .borrow()
                    .get_var(
                        identifier,
                        &self.start_position,
                        &self.end_position,
                        file_data
                    )
                    .unwrap()
                    .clone(),
            InstructionKind::Indent | InstructionKind::Dedent => Literal::Null,
            InstructionKind::SystemOut { value } => { print!("{}", value.visit(file_data, context)); Literal::Null},
            InstructionKind::If { condition, if_else, body: inside } => {
                let new_context = Context::new(Some(context.clone()));
                if condition.is_none() || condition.as_ref().unwrap().visit(file_data, context.clone()).to_bool() {
                    run(&inside.to_vec(), file_data, Rc::new(RefCell::new(new_context)))
                } else if let Some(ifelse) = if_else {
                    ifelse.visit(file_data, context.clone())
                } else {
                    Literal::Null
                }
            },
            InstructionKind::While { condition, body: inside } => {
                let new_context = Context::new(Some(context.clone()));
                let rc = Rc::new(RefCell::new(new_context));
                let inside = inside.to_vec();
                while condition.as_ref().visit(file_data, context.clone()).to_bool() {
                    run(&inside, file_data, rc.clone());
                }
                Literal::Null
            },
            InstructionKind::DeclareFunction { identifier, args, body } => {
                context.borrow_mut().function_table.set(identifier.clone(), Function::new(identifier.clone(), args.clone(), body.clone()));
                Literal::Null
            },
            InstructionKind::CallFunction { identifier, args } => {
                Context::call_func(context.clone(), identifier, args.to_vec(), &self.start_position, &self.end_position, file_data)
            },
            InstructionKind::Use { file_path } => {
                Context::import_file(context, file_path);
                Literal::Null
            },
            InstructionKind::Return { value } => value.visit(file_data, context),
        }
    }
}


// BINARY OPERATION

#[derive(PartialEq, Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    EqualsTo,
    NotEqualsTo,
    GreaterThan,
    LesserThan,
    GreaterEquals,
    LesserEquals,
    And,
    Or,
    Nothing,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Minus,
    Plus,
    Not
}