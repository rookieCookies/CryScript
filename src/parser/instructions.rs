#![allow(unused)]
use core::panic;
use std::{fmt::Display, ops::Deref, rc::Rc, cell::{RefMut, RefCell}, env};

use crate::{utils::{Position, read_file, FileData}, lexer::{tokens::{Token, TokenKind}, Lexer}, exceptions::{interpreter_errors::TokenLiteralConversion, Exception}, run, environment::ENV_DEV_DEBUG_INTERPRETER};

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
    Out {
        value: Box<Instruction>,
    },
    ConvertType {
        value: Box<Instruction>,
        convert_to: ConvertType,
    },
    Indent,
    Dedent,
    Lit(Literal)
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Integer(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Return(Box<Literal>),
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

    pub fn convert(&self, convert_type: &ConvertType) -> Literal {
        match convert_type {
            ConvertType::Integer => self.as_int(),
            ConvertType::Float => self.as_float(),
            ConvertType::String => Self::String(self.to_string()),
            ConvertType::Bool => Literal::Bool(self.as_bool()),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Literal::Integer(v) => v >= &1,
            Literal::Float(v) => v >= &1.,
            Literal::String(v) => (match v.parse() {
                Ok(v) => v,
                Err(_) => false,
            }),
            Literal::Bool(v) => *v,
            Literal::Null => false,
            Literal::Return(l) => l.as_bool(),
        }
    }

    pub fn as_int(&self) -> Literal {
        match self {
            Literal::Integer(v) => Literal::Integer(*v),
            Literal::Float(v) => Literal::Integer(*v as i32),
            Literal::String(v) => Literal::Integer(match v.parse() {
                Ok(v) => v,
                Err(_) => 0,
            }),
            Literal::Bool(v) => panic!(),
            Literal::Null => panic!(),
            Literal::Return(l) => l.as_int(),
        }
    }

    pub fn as_float(&self) -> Literal {
        match self {
            Literal::Integer(v) => Literal::Float(*v as f32),
            Literal::Float(v) => Literal::Float(*v),
            Literal::String(v) => Literal::Float(match v.parse() {
                Ok(v) => v,
                Err(_) => 0.,
            }),
            Literal::Bool(v) => panic!(),
            Literal::Null => panic!(),
            Literal::Return(l) => l.as_float(),
        }
    }

    pub fn kind(&self) -> String {
        match self {
            Literal::Integer(v) => "integer".to_string(),
            Literal::Float(v) => "float".to_string(),
            Literal::String(v) => "string".to_string(),
            Literal::Bool(v) => "bool".to_string(),
            Literal::Null => "null".to_string(),
            Literal::Return(l) => format!("return {}", l.kind()),
        }
    }

    pub fn value(&self) -> Literal {
        match self {
            Literal::Integer(_) => self.clone(),
            Literal::Float(_) => self.clone(),
            Literal::String(_) => self.clone(),
            Literal::Bool(_) => self.clone(),
            Literal::Return(v) => v.value().clone(),
            Literal::Null => self.clone(),
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
                Literal::Null => "null".to_string(),
                Literal::Return(l) => format!("return {}", l.to_string()),
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

    pub fn visit(&self, file_data: &FileData, m_context: Rc<RefCell<Context>>) -> Literal {
        let context = m_context.clone();
        if env::var(ENV_DEV_DEBUG_INTERPRETER).unwrap() == "true" {
            for i in 0..context.clone().try_borrow().unwrap().parent_count() {
                print!("---")
            }
            println!("-> {}", self)
        }
        match &self.kind {
            InstructionKind::BinaryOperation { left, right, operation } => 
                operation.compute(
                    file_data, 
                    (
                        left.start_position.clone(), 
                        right.end_position.clone()
                    ), 
                    (
                        left.visit(file_data, context.clone()).value(), 
                        right.visit(file_data, context.clone()).value()
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
                            literal.value(),
                            Literal::Integer(-1)
                        )
                    ),
                    UnaryOperator::Plus => literal.value(),
                    UnaryOperator::Not => if literal.value().as_bool() { Literal::Bool(false) } else { { Literal::Bool(true) } },
                }
            },
            InstructionKind::VarAssign { identifier, value } => {
                let value = (**value).visit(file_data, context.clone()).clone();
                context.borrow_mut().variable_table.set(
                    identifier.to_string(), 
                    value.value()
                );
                Instruction::new(self.start_position.clone(), self.end_position.clone(), InstructionKind::VarAccess { identifier: identifier.clone() }).visit(file_data, context)
            },
            InstructionKind::VarUpdate { identifier, value } => {
                let value = (**value).visit(file_data, context.clone()).clone();
                context.borrow_mut().update_var(
                    identifier, 
                    value.value(),
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
            InstructionKind::If { condition, if_else, body: inside } => {
                let new_context = Context::new(Some(context.clone()), false);
                let l = if condition.is_none() || condition.as_ref().unwrap().visit(file_data, context.clone()).as_bool() {
                    Literal::Return(Box::new(run(&inside.to_vec(), file_data, Rc::new(RefCell::new(new_context))).value()))
                } else if let Some(ifelse) = if_else {
                    ifelse.visit(file_data, context.clone())
                } else {
                    Literal::Null
                };
                l
            },
            InstructionKind::While { condition, body: inside } => {
                let new_context = Context::new(Some(context.clone()), false);
                let rc = Rc::new(RefCell::new(new_context));
                let inside = inside.to_vec();
                let mut return_value = Literal::Null;
                while condition.as_ref().visit(file_data, context.clone()).as_bool() {
                    let value = run(&inside, file_data, rc.clone());
                    if value != Literal::Null {
                        return_value = value.value();
                        break;
                    }
                }
                return_value
            },
            InstructionKind::DeclareFunction { identifier, args, body } => {
                context.borrow_mut().function_table.set(identifier.clone(), Function::new(identifier.clone(), args.clone(), body.clone()));
                Literal::Null
            },
            InstructionKind::CallFunction { identifier, args } => {
                Context::call_func(context.clone(), identifier, args.iter().map(|x| x.visit(file_data, context.clone())).collect(), &self.start_position, &self.end_position, file_data)
            },
            InstructionKind::Use { file_path } => {
                Context::import_file(context, file_path);
                Literal::Null
            },
            InstructionKind::Return { value } => {
                Literal::Return(Box::new(value.visit(file_data, context)))
            },
            InstructionKind::ConvertType { value, convert_to } => {
                value.visit(file_data, context).convert(&convert_to)
            },
            InstructionKind::Out { value } => value.visit(file_data, m_context),
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



impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.kind
        )
    }
}

impl Display for InstructionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                InstructionKind::BinaryOperation { left, right, operation } => format!("b_operator({} {}, {})", left, right, operation.to_string()),
                InstructionKind::UnaryOperation { operator, value } => format!("u_operator({}, {})", operator, value),
                InstructionKind::VarAssign { identifier, value } => format!("varassign({}, {})", identifier, value),
                InstructionKind::VarUpdate { identifier, value } => format!("varaccess({}, {})", identifier, value),
                InstructionKind::VarAccess { identifier } => format!("varaccess({})", identifier),
                InstructionKind::If { condition, body, if_else } => format!("if({}, [{}], else([{}]))", if let Some(c) = condition { c.to_string() } else { "None".to_string() }, body.iter().map(|i| i.to_string()).collect::<String>(), if let Some(i) = if_else { i.to_string() } else { "None".to_string() }),
                InstructionKind::While { condition, body } => format!("while({}, [{}])", condition, body.iter().map(|i| i.to_string()).collect::<String>()),
                InstructionKind::DeclareFunction { identifier, args, body } => format!("fn({}, [{}], [{}])", identifier, args.iter().map(|i| i.to_string()).collect::<String>(), body.iter().map(|i| i.to_string()).collect::<String>()),
                InstructionKind::CallFunction { identifier, args } => format!("callfn({}, [{}])", identifier, args.iter().map(|i| i.to_string()).collect::<String>()),
                InstructionKind::Use { file_path } => format!("use({})", file_path),
                InstructionKind::Return { value } => format!("return({})", value),
                InstructionKind::Out { value } => format!("out({})", value),
                InstructionKind::ConvertType { value, convert_to } => format!("convert_to({}, {})", value, convert_to),
                InstructionKind::Indent => "indent".to_string(),
                InstructionKind::Dedent => "dedent".to_string(),
                InstructionKind::Lit(v) => format!("{}({})", v.kind(), v.to_string()),
            }
        )
    }
}
#[derive(Debug, Clone)]
pub enum ConvertType {
    Integer,
    Float,
    String,
    Bool
}

impl Display for ConvertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
            match self {
                ConvertType::Integer => "integer",
                ConvertType::Float => "float",
                ConvertType::String => "string",
                ConvertType::Bool => "bool",
            }
        )
    }
}