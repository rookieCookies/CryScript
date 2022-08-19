use std::{fmt::Display, rc::Rc};

use crate::{
    exceptions::{
        interpreter_exceptions::{InvalidAmountOfArguments, InvalidArgumentType},
        parser_exceptions::NotATypeHint,
        Exception,
    },
    lexer::token::Token,
    parser::data::original_data,
    FileData, Position,
};

use super::{context::Context, instructions::Instruction, DataRef};

#[derive(Debug, Clone)]
pub(crate) struct Function {
    pub(crate) arguments: Vec<Argument>,
    pub(crate) body: Instruction,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) identifier: String,
}

impl Function {
    pub(crate) fn new(
        arguments: &[(String, Type, Option<Instruction>)],
        body: Instruction,
        start: Position,
        end: Position,
        identifier: String,
        context: *mut Context,
    ) -> Self {
        let v: Vec<Argument> = arguments
            .iter()
            .map(|(i, th, dv)| {
                Argument::new(
                    i.clone(),
                    th.clone(),
                    match dv {
                        Some(v) => match v.visit(context) {
                            Ok(v) => Some(v.unwrap()),
                            Err(v) => v.run(),
                        },
                        None => None,
                    },
                )
            })
            .collect();
        Self {
            arguments: v,
            body,
            start,
            end,
            identifier,
        }
    }

    pub(crate) fn call(
        &self,
        context: *mut Context,
        file_data: Rc<FileData>,
        args: Vec<DataRef>,
    ) -> Result<DataRef, Exception> {
        let mut func_context = Context::new(Some(context), file_data.clone());

        for (i, arg) in self.arguments.iter().enumerate() {
            match &arg.default_value {
                Some(v) => func_context.assign_variable(
                    arg.identifier.clone(),
                    v.clone(),
                    arg.type_hint.clone(),
                    true,
                    (&self.start, &self.end, &file_data),
                )?,
                None => {
                    if args.len() <= i {
                        return Err(InvalidAmountOfArguments::call(
                            &self.start,
                            &self.end,
                            &file_data.clone(),
                            &self.identifier,
                            self.arguments.len(),
                            args.len(),
                        ));
                    }
                }
            }
            if args.len() <= i {
                continue;
            }
            match (
                &arg.type_hint.type_value,
                &original_data(&args[i]).borrow().data_type,
            ) {
                (TypeHint::Integer, crate::parser::data::DataType::Integer(_))
                | (TypeHint::String, crate::parser::data::DataType::String(_))
                | (TypeHint::Float, crate::parser::data::DataType::Float(_))
                | (TypeHint::None, _) => {}
                (TypeHint::Class(v), crate::parser::data::DataType::Class(class)) => {
                    if &class.class_name != v {
                        return Err(InvalidArgumentType::call(
                            &args[i].borrow().start,
                            &args[i].borrow().end,
                            &args[i].borrow().file_data,
                            &arg.identifier,
                            &arg.type_hint.type_value,
                            &args[i].borrow().data_type,
                        ));
                    }
                }
                _ => {
                    return Err(InvalidArgumentType::call(
                        &args[i].borrow().start,
                        &args[i].borrow().end,
                        &args[i].borrow().file_data,
                        &arg.identifier,
                        &arg.type_hint.type_value,
                        &args[i].borrow().data_type,
                    ))
                }
            }
            func_context.assign_variable(
                arg.identifier.clone(),
                args[i].to_owned(),
                arg.type_hint.clone(),
                true,
                (
                    &args[i].borrow().start,
                    &args[i].borrow().end,
                    &args[i].borrow().file_data,
                ),
            )?;
        }
        match self.body.visit(&mut func_context) {
            Ok(v) => Ok(v.unwrap()),
            Err(v) => Err(v),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Argument {
    identifier: String,
    type_hint: Type,
    default_value: Option<DataRef>,
}

impl Argument {
    pub(crate) fn new(identifier: String, type_hint: Type, default_value: Option<DataRef>) -> Self {
        Self {
            identifier,
            type_hint,
            default_value,
        }
    }
}

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
