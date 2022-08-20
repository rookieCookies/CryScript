use std::{
    fs::{File, OpenOptions},
    io::{stdin, stdout, Read, Write},
    rc::Rc,
};

use rand::Rng;

use crate::{
    exceptions::{
        interpreter_exceptions::{FailedToReadInput, InvalidFilePath},
        Exception,
    },
    parser::data::{Data, DataType},
    FileData, Position,
};

use super::{context::Context};

pub struct BuiltInFunctions;
impl BuiltInFunctions {
    pub(crate) fn run(
        context: *mut Context,
        identifier: &String,
        args: Vec<Data>,
        (start, end, file_data): (&Position, &Position, &Rc<FileData>),
    ) -> Result<Data, Exception> {
        match identifier.as_str() {
            "std_clone" => Ok(Data::new(
                file_data.clone(),
                start.clone(),
                end.clone(),
                args[0].data_type.original().clone(),
            )),
            "std_out" => {
                print!("{}", args[0].data_type);
                Ok(Data::null_zero(file_data.clone()))
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
                Ok(Data::new(
                    file_data.clone(),
                    start.clone(),
                    end.clone(),
                    DataType::String(s),
                ))
            }
            "std_rand_int" => Ok(Data::new(
                file_data.clone(),
                start.clone(),
                end.clone(),
                DataType::Integer(rand::thread_rng().gen()),
            )),
            "std_rand_float" => Ok(Data::new(
                file_data.clone(),
                start.clone(),
                end.clone(),
                DataType::Float(rand::thread_rng().gen()),
            )),
            "std_contents_of_file" => {
                let mut s = String::new();
                match match File::open(args[0].data_type.to_string()) {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(InvalidFilePath::call(
                            &start,
                            &end,
                            &file_data,
                            &args[0].data_type.to_string(),
                        ))
                    }
                }
                .read_to_string(&mut s)
                {
                    Ok(_) => {}
                    Err(_) => {
                        return Err(InvalidFilePath::call(
                            &start,
                            &end,
                            &file_data,
                            &args[0].data_type.to_string(),
                        ))
                    }
                };
                Ok(Data::new(
                    file_data.clone(),
                    start.clone(),
                    end.clone(),
                    DataType::String(s),
                ))
            }
            "std_write_to_file" => {
                let mut file = match OpenOptions::new()
                    .write(true)
                    .open(args[0].data_type.to_string())
                {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(InvalidFilePath::call(
                            &start,
                            &end,
                            &file_data,
                            &args[0].data_type.to_string(),
                        ))
                    }
                };
                // file.set_len(1).unwrap();
                file.write_all(args[1].data_type.to_string().as_bytes())
                    .unwrap();
                Ok(Data::new(
                    file_data.clone(),
                    start.clone(),
                    end.clone(),
                    DataType::Null,
                ))
            }
            "std_create_file" => {
                match File::create(args[0].data_type.to_string()) {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(InvalidFilePath::call(
                            &start,
                            &end,
                            &file_data,
                            &args[0].data_type.to_string(),
                        ))
                    }
                };
                Ok(Data::new(
                    file_data.clone(),
                    start.clone(),
                    end.clone(),
                    DataType::Null,
                ))
            }
            "std_clear_file" => {
                match OpenOptions::new()
                    .write(true)
                    .open(args[0].data_type.to_string())
                {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(InvalidFilePath::call(
                            &start,
                            &end,
                            &file_data,
                            &args[0].data_type.to_string(),
                        ))
                    }
                }
                .set_len(0)
                .unwrap();
                Ok(Data::new(
                    file_data.clone(),
                    start.clone(),
                    end.clone(),
                    DataType::Null,
                ))
            }
            "std_sqrt" => match args[0].data_type.original() {
                DataType::Float(n1) => Ok(Data::new(
                    file_data.clone(),
                    start.clone(),
                    end.clone(),
                    DataType::Float(n1.sqrt()),
                )),
                _ => panic!(),
            },
            "std_exit" => std::process::exit(
                match args[0].original().data_type {
                    DataType::Integer(v) => v,
                    DataType::Float(v) => v as i32,
                    _ => panic!(),
                },
            ),
            _ => Context::call_fn_no_std(context, identifier, args, (start, end, file_data)),
        }
    }
}
