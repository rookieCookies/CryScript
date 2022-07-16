use std::{collections::HashMap, borrow::{Borrow, BorrowMut}, rc::Rc, cell::{RefMut, RefCell}, env, fmt::Display};

use crate::{run, utils::{Position, FileData}, exceptions::{interpreter_errors::{AccessUndeclaredVariable, AccessUndeclaredFunction}, Exception}, environment::ENV_DUMP, std_lib::{STD_MATH, STD_RAND}};

use super::{instructions::{Instruction, Literal}, functions::Function, built_in_functions::BuiltInFunction};

#[derive(Debug)]
pub struct Context {
    pub parent: Option<Rc<RefCell<Context>>>,
    pub variable_table: Table<Literal>,
    pub function_table: Table<Function>,
    pub is_root: bool,
    imported_files: Vec<String>,
}

impl Context {
    pub fn new(parent: Option<Rc<RefCell<Context>>>, is_root: bool) -> Self {
        if env::var(ENV_DUMP).unwrap() == "true" {
            println!("created new context");
        }
        Self { 
            parent,
            variable_table: Table::new("variable".to_string()),
            function_table: Table::new("function".to_string()),
            imported_files: Vec::new(),
            is_root,
        } 
    }
    pub fn get_var(&self, identifier: &String, start_position: &Position, end_position: &Position, file_data: &FileData) -> Option<Literal> {
        if env::var(ENV_DUMP).unwrap() == "true" {
            println!("get variable {:?}", &identifier);
        }
        // Some(self.variable_table.get(identifier).unwrap().clone())
        if self.variable_table.symbols.get(&identifier.clone()).clone().is_some() {
            return Some(self.variable_table.symbols.get(&identifier.clone()).unwrap().clone())
        }
        if let Some(v) = self.variable_table.get(identifier) {
            Some(v.clone())
        // } else {
        //     AccessUndeclaredVariable::new(start_position.clone(), end_position.clone(), file_data, identifier).run()
        // }
        } else {
            if let Some(parent) = &self.parent {
                parent.try_borrow().unwrap().get_var(identifier, start_position, end_position, file_data)
            } else {
                AccessUndeclaredVariable::new(start_position.clone(), end_position.clone(), file_data, identifier).run()
            }
        }
    }
    pub fn update_var(&mut self, identifier: &String, value: Literal, start_position: &Position, end_position: &Position, file_data: &FileData) {
        if env::var(ENV_DUMP).unwrap() == "true" {
            println!("update variable {:?}", &identifier);
        }
        if self.variable_table.symbols.contains_key(identifier) {
            self.variable_table.set(identifier.to_string(), value)
        } else {
            if self.parent.is_some() {
                self.parent.as_ref().unwrap().try_borrow_mut().unwrap().update_var(identifier, value, start_position, end_position, file_data);
            } else {
                AccessUndeclaredVariable::new(start_position.clone(), end_position.clone(), file_data, identifier).run()
            }
        }
    }
    pub fn call_func(context: Rc<RefCell<Context>>, identifier: &String, args: Vec<Literal>, start_position: &Position, end_position: &Position, file_data: &FileData) -> Literal {
        BuiltInFunction::call(context, identifier, args, start_position, end_position, file_data)
    }

    pub fn call_func_no_std(context: Rc<RefCell<Context>>, identifier: &String, args: Vec<Literal>, start_position: &Position, end_position: &Position, file_data: &FileData) -> Literal {
        let cell = context.try_borrow().unwrap();
        if cell.function_table.symbols.contains_key(identifier) {
            cell.function_table.get(identifier).unwrap().call(file_data, (start_position.clone(), end_position.clone()), context.clone(), args)
        } else {
            if cell.parent.is_some() {
                Context::call_func_no_std(cell.parent.as_ref().unwrap().clone(), identifier, args, start_position, end_position, file_data)
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
        use crate::read_file;
        use crate::{lexer::Lexer, Parser};
        if context.try_borrow().unwrap().imported_files.contains(file_path) {
            return Literal::Null
        }
        match file_path.as_str() {
            "std_math" => Context::import_string(context, STD_MATH, "std_math"),
            "std_rand" => Context::import_string(context, STD_RAND, "std_rand"),
            _ => {
                let file_data = read_file(file_path);
                context.try_borrow_mut().unwrap().imported_files.push(file_path.clone());
                run(&Parser::new(Lexer::new(&file_data).lex(), &file_data).parse(), &file_data, context)
            }
        }
    }

    pub fn import_string(context: Rc<RefCell<Context>>, data: &str, file_name: &str) -> Literal {
        use crate::{lexer::Lexer, Parser};
        let file_data = FileData { data: data.to_string(), file_name: file_name.to_string() };
        run(&Parser::new(Lexer::new(&file_data).lex(), &file_data).parse(), &file_data, context)
    }

    pub fn dump(&self) {
        for (key, value) in &self.variable_table.symbols {
            println!("{} : {:?}", key, value)
        }
        if let Some(parent) = self.parent.as_ref() {
            parent.try_borrow().unwrap().dump()
        }
    }

    pub fn parent_count(&self) -> usize {
        let mut i = 0;
        let mut p = self.parent.clone();
        while p.is_some() {
            p = p.unwrap().try_borrow().unwrap().parent.clone();
            i += 1;
        }
        i
    }
}

#[derive(Debug)]
pub struct Table<T> {
    pub symbols: HashMap<String, T>,
    display_name: String
}

impl<T> Table<T> {
    pub fn new(display_name: String) -> Self { 
        Self { 
            symbols: HashMap::new(),
            display_name,
        } 
    }

    pub fn get(&self, identifier: &String) -> Option<&T> {
        self.symbols.get(identifier)
    }

    pub fn set(&mut self, identifier: String, instruction: T) {
        if env::var(ENV_DUMP).unwrap() == "true" {
            println!("set {} {:?}", self.display_name, &identifier);
        }
        self.symbols.insert(identifier, instruction);
    }

    pub fn remove(&mut self, identifier: String) {
        self.symbols.remove(&identifier);
    }
}