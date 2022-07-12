#![allow(unused)]
use std::{env, cell::{RefMut, RefCell}, rc::Rc};

use colored::Colorize;
use environment::{register_environment_variables, ENV_FILE_NAME};
use lexer::Lexer;
use parser::instructions::{Instruction, InstructionKind, self, Literal};
use utils::{read_file, FileData};

use crate::{lexer::tokens::TokenKind, environment::{ENV_DEV_DEBUG_LEXER, ENV_DEV_DEBUG_PARSER}, parser::{Parser, context::Context}};

mod environment;
mod utils;
mod lexer;
mod tests;
mod exceptions;
mod parser;

fn main() {
    register_environment_variables();
    run_app()
}

fn run_app() {
    let root_context = Context::new(None);
    let cell = RefCell::new(root_context);
    Context::import_file(Rc::new(cell), &env::var(ENV_FILE_NAME).unwrap());
}

pub fn run(instructions: &Vec<Instruction>, file_data: &FileData, context: Rc<RefCell<Context>>) -> Literal {
    for instruction in instructions {
        if matches!(instruction.kind, InstructionKind::Use { .. }) {
            instruction.visit(file_data, context.clone());
        }
    }
    for instruction in instructions {
        if matches!(instruction.kind, InstructionKind::DeclareFunction { .. }) {
            instruction.visit(file_data, context.clone());
        }
    }
    for instruction in instructions {
        if !matches!(instruction.kind, InstructionKind::DeclareFunction { .. } | InstructionKind::Use { .. }) {
            if matches!(instruction.kind, InstructionKind::Return { .. }) {
                return instruction.visit(file_data, context.clone());
            }
            instruction.visit(file_data, context.clone());
        }
    }
    return Literal::Null
}
