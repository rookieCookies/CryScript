use std::{collections::HashMap, borrow::{Borrow, BorrowMut}, rc::Rc, cell::{RefMut, RefCell}};

use crate::{run, utils::{Position, FileData}, exceptions::{interpreter_errors::{AccessUndeclaredVariable, AccessUndeclaredFunction}, Exception}};

use super::{instructions::{Instruction, Literal}, functions::Function};

pub struct Context {
    pub parent: Option<Rc<RefCell<Context>>>,
    pub symbol_table: Table<Literal>,
    pub function_table: Table<Function>,
    imported_files: Vec<String>,
}

impl Context {
    pub fn new(parent: Option<Rc<RefCell<Context>>>) -> Self { 
        Self { 
            parent,
            symbol_table: Table::new(),
            function_table: Table::new(),
            imported_files: Vec::new(),
        } 
    }
    pub fn get_var(&self, identifier: &String, start_position: &Position, end_position: &Position, file_data: &FileData) -> Option<Literal> {
        if let Some(v) = self.symbol_table.get(identifier) {
            Some(v.clone())
        } else {
            if let Some(parent) = &self.parent {
                parent.try_borrow().unwrap().get_var(identifier, start_position, end_position, file_data)
            } else {
                AccessUndeclaredVariable::new(start_position.clone(), end_position.clone(), file_data, identifier).run()
            }
        }
    }
    pub fn update_var(&mut self, identifier: &String, value: Literal, start_position: &Position, end_position: &Position, file_data: &FileData) {
        if self.symbol_table.symbols.contains_key(identifier) {
            self.symbol_table.set(identifier.to_string(), value)
        } else {
            if self.parent.is_some() {
                self.parent.as_ref().unwrap().try_borrow_mut().unwrap().update_var(identifier, value, start_position, end_position, file_data);
            } else {
                AccessUndeclaredVariable::new(start_position.clone(), end_position.clone(), file_data, identifier).run()
            }
        }
    }
    pub fn call_func(context: Rc<RefCell<Context>>, identifier: &String, args: Vec<Instruction>, start_position: &Position, end_position: &Position, file_data: &FileData) -> Literal {
        let cell = context.try_borrow().unwrap();
        let arguments = args.iter().map(|x| x.visit(file_data, context.clone())).collect();
        if cell.function_table.symbols.contains_key(identifier) {
            cell.function_table.get(identifier).unwrap().call(file_data, (start_position.clone(), end_position.clone()), context.clone(), arguments)
        } else {
            if cell.parent.is_some() {
                Context::call_func(cell.parent.as_ref().unwrap().clone(), identifier, args, start_position, end_position, file_data)
            } else {
                AccessUndeclaredFunction::new(start_position.clone(), end_position.clone(), file_data, identifier).run()
            }
        }
    }
    pub fn has_file(context: Rc<RefCell<Context>>, file_path: &String) -> bool {
        let cell = context.try_borrow().unwrap();
        if cell.imported_files.contains(file_path) {
            true
        } else {
            if cell.parent.is_some() {
                Context::has_file(cell.parent.as_ref().unwrap().clone(), file_path)
            } else {
                false
            }
        }
    }
    pub fn import_file(context: Rc<RefCell<Context>>, file_path: &String) -> Literal {
        use crate::{read_file, lexer::Lexer, Parser};
        if context.try_borrow().unwrap().imported_files.contains(file_path) {
            return Literal::Null
        }
        let file_data = read_file(file_path);
        context.try_borrow_mut().unwrap().imported_files.push(file_path.clone());
        run(&Parser::new(Lexer::new(&file_data).lex(), &file_data).parse(), &file_data, context)
    }
    pub fn dump(&self) {
        for (key, value) in &self.symbol_table.symbols {
            println!("{} : {:?}", key, value)
        }
        if let Some(parent) = self.parent.as_ref() {
            parent.try_borrow().unwrap().dump()
        }
    }
}

pub struct Table<T> {
    symbols: HashMap<String, T>
}

impl<T> Table<T> {
    pub fn new() -> Self { 
        Self { 
            symbols: HashMap::new()
        } 
    }

    pub fn get(&self, identifier: &String) -> Option<&T> {
        self.symbols.get(identifier)
    }

    pub fn set(&mut self, identifier: String, instruction: T) {
        self.symbols.insert(identifier, instruction);
    }

    pub fn remove(&mut self, identifier: String) {
        self.symbols.remove(&identifier);
    }
}