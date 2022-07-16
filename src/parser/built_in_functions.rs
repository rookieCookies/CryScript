use std::{cell::RefCell, rc::Rc, io::{stdin, stdout, Write}};

use rand::Rng;

use crate::utils::{FileData, Position};

use super::{context::Context, instructions::Literal};

pub struct BuiltInFunction;

impl BuiltInFunction {
    pub fn call(context: Rc<RefCell<Context>>, identifier: &String, args: Vec<Literal>, start_position: &Position, end_position: &Position, file_data: &FileData) -> Literal {
        match identifier.as_str() {
            "systemOut" => ExternalFunction::sysout(args, start_position, end_position, file_data),
            "systemIn" => ExternalFunction::sysin(args, start_position, end_position, file_data),
            "systemRandFloat" => Literal::Float(rand::thread_rng().gen::<f32>()),
            "systemRandInteger" => Literal::Integer(rand::thread_rng().gen::<i32>()),
            _ => Context::call_func_no_std(context, identifier, args, start_position, end_position, file_data),
        }
    }
}

struct ExternalFunction;

impl ExternalFunction {
    fn sysout(args: Vec<Literal>, start_position: &Position, end_position: &Position, file_data: &FileData) -> Literal {
        print!("{}", args.iter().map(|x| x.to_string()).collect::<String>());
        Literal::Null
    }

    fn sysin(args: Vec<Literal>, start_position: &Position, end_position: &Position, file_data: &FileData) -> Literal {
        let mut s = String::new();
        stdout().flush();
        stdin().read_line(&mut s).expect("Failed to read input");
        if let Some('\n')=s.chars().next_back() {
            s.pop();
        }
        if let Some('\r')=s.chars().next_back() {
            s.pop();
        }
        Literal::String(s)
    }
}