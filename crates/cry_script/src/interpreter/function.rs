use std::rc::Rc;

use crate::{
    exceptions::{
        interpreter_exceptions::{InvalidAmountOfArguments, InvalidArgumentType},
        Exception,
    },
    parser::data::Data,
    FileData, Position,
};

use super::{
    context::Context,
    instructions::Instruction,
    type_hint::{Type, TypeHint},
};

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
        args: Vec<Data>,
    ) -> Result<Data, Exception> {
        let mut func_context = Context::new(context, file_data.clone());

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
            match (&arg.type_hint.type_value, &args[i].original().data_type) {
                (TypeHint::Integer, crate::parser::data::DataType::Integer(_))
                | (TypeHint::String, crate::parser::data::DataType::String(_))
                | (TypeHint::Float, crate::parser::data::DataType::Float(_))
                | (TypeHint::None, _)
                | (_, crate::parser::data::DataType::Null) => {}
                (TypeHint::Class(v), crate::parser::data::DataType::Class(class)) => {
                    if &class.class_name != v {
                        return Err(InvalidArgumentType::call(
                            &args[i].start,
                            &args[i].end,
                            &args[i].file_data,
                            &arg.identifier,
                            &arg.type_hint.type_value,
                            &args[i].data_type,
                        ));
                    }
                }
                _ => {
                    return Err(InvalidArgumentType::call(
                        &args[i].start,
                        &args[i].end,
                        &args[i].file_data,
                        &arg.identifier,
                        &arg.type_hint.type_value,
                        &args[i].data_type,
                    ))
                }
            }
            func_context.assign_variable(
                arg.identifier.clone(),
                args[i].to_owned(),
                arg.type_hint.clone(),
                true,
                (&args[i].start, &args[i].end, &args[i].file_data),
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
    default_value: Option<Data>,
}

impl Argument {
    pub(crate) fn new(identifier: String, type_hint: Type, default_value: Option<Data>) -> Self {
        Self {
            identifier,
            type_hint,
            default_value,
        }
    }
}
