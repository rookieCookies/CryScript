use std::{
    io::{stdin, stdout, Write},
    rc::Rc,
};

use rand::Rng;
use utils::wrap;

use crate::{
    exceptions::{interpreter_exceptions::FailedToReadInput, Exception},
    parser::data::{original_data, Data, DataType},
    FileData, Position,
};

use super::{context::Context, DataRef};

pub struct BuiltInFunctions;
impl BuiltInFunctions {
    pub(crate) fn run(
        context: *mut Context,
        identifier: &String,
        args: Vec<DataRef>,
        (start, end, file_data): (&Position, &Position, &Rc<FileData>),
    ) -> Result<DataRef, Exception> {
        match identifier.as_str() {
            "std_clone" => Ok(wrap(Data::new(
                file_data.clone(),
                start.clone(),
                end.clone(),
                args[0].try_borrow().unwrap().data_type.clone(),
            ))),
            "std_out" => {
                print!("{}", args[0].try_borrow().unwrap().data_type);
                Ok(wrap(Data::null(file_data.clone())))
            }
            "std_in" => {
                let mut s = String::new();
                stdout().flush().unwrap();
                match stdin().read_line(&mut s) {
                    Ok(_) => {}
                    Err(_) => return Err(FailedToReadInput::call(start, end, file_data)),
                };
                if let Some('\n') = s.chars().next_back() {
                    s.pop();
                }
                if let Some('\r') = s.chars().next_back() {
                    s.pop();
                }
                Ok(wrap(Data::new(
                    file_data.clone(),
                    start.clone(),
                    end.clone(),
                    DataType::String(s),
                )))
            }
            "std_rand_int" => Ok(wrap(Data::new(
                file_data.clone(),
                start.clone(),
                end.clone(),
                DataType::Integer(rand::thread_rng().gen()),
            ))),
            "std_rand_float" => Ok(wrap(Data::new(
                file_data.clone(),
                start.clone(),
                end.clone(),
                DataType::Float(rand::thread_rng().gen()),
            ))),
            "std_sqrt" => match args[0].borrow().data_type.original() {
                DataType::Float(n1) => Ok(wrap(Data::new(
                    file_data.clone(),
                    start.clone(),
                    end.clone(),
                    DataType::Float(n1.sqrt()),
                ))),
                _ => panic!(),
            },
            "std_exit" => std::process::exit(
                match original_data(&args[0]).try_borrow().unwrap().data_type {
                    DataType::Integer(v) => v,
                    DataType::Float(v) => v as i32,
                    _ => panic!(),
                },
            ),
            _ => Context::call_fn_no_std(context, identifier, args, (start, end, file_data)),
        }
    }
}
