use std::{cell::RefCell, rc::Rc, collections::HashMap};

use crate::{run, exceptions::{interpreter_errors::InvalidAmountOfArguments, Exception}, utils::{Position, FileData}};

use super::{instructions::{Instruction, Literal}, context::Context};

#[derive(Debug)]
pub struct Function {
    id: String,
    args_ids: Vec<String>,
    inside: Vec<Instruction>,
}

impl Function {
    pub fn new(id: String, args_ids: Vec<String>, inside: Vec<Instruction>) -> Self { Self { id, args_ids, inside } }
    pub fn call(&self, file_data: &FileData, (start, end): (Position, Position), parent_context: Rc<RefCell<Context>>, arguments: Vec<Literal>) -> Literal {
        if arguments.len() != self.args_ids.len() {
            InvalidAmountOfArguments::new(start, end, file_data, self.args_ids.len(), arguments.len()).run()
        }
        let mut context = Context::new(Some(parent_context), true);
        let mut inside = self.inside.clone();
        for i in 0..arguments.len() {
            context.variable_table.set(self.args_ids[i].clone(), arguments[i].clone())
        }
        let l = run(&self.inside, file_data, Rc::new(RefCell::new(context)));
        l
    }
}

pub struct FunctionTable {
    map: HashMap<String, Function>
}