use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    exceptions::{parser_exceptions::InvalidInstructionInClass, Exception},
    parser::data::{Data, DataType},
    variables::Variable,
    AsString, FileData, Position,
};

use self::{
    context::Context,
    function::Function,
    instructions::{Instruction, InstructionType},
    type_hint::{Type, TypeHint},
};

mod built_in_functions;
pub mod context;
pub mod function;
pub mod instructions;
pub mod type_hint;

#[derive(Debug, Clone)]
pub(crate) struct Table<T: AsString> {
    map: HashMap<String, T>,
}

impl<T: AsString> Table<T> {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl<T: AsString> Display for Table<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.map
                .iter()
                .map(|x| format!("(name: {}, value {}) ", x.0, x.1.as_string()))
                .collect::<String>()
        )
    }
}

pub type DataRef = Rc<RefCell<Data>>;

#[derive(Debug, Clone)]
pub(crate) struct Class {
    identifier: String,
    variables: Vec<(String, Variable)>,
    start: Position,
    end: Position,
    file_data: Rc<FileData>,
}

impl Class {
    pub(crate) fn new(
        identifier: String,
        (start, end, file_data): (Position, Position, Rc<FileData>),
        body: Vec<Instruction>,
        parent: *mut Context,
    ) -> Result<Self, Exception> {
        let mut context = Context::new(parent, unsafe { &*parent }.file_data.clone());
        let mut variables = vec![];
        for i in body.into_iter() {
            match i.instruction_type {
                InstructionType::VarAssign {
                    identifier,
                    data,
                    type_hint,
                    is_final,
                } => variables.push((
                    identifier.clone(),
                    Variable::new(
                        match data.visit(&mut context)? {
                            crate::Returnable::Return(v) => v,
                            crate::Returnable::Evaluate(v) => v,
                            crate::Returnable::Break(v) => v,
                        },
                        type_hint,
                        is_final,
                        identifier.clone(),
                    ),
                )),
                InstructionType::FunctionDeclaration {
                    identifier,
                    body,
                    arguments,
                } => {
                    variables.push((
                        identifier.clone(),
                        Variable::new(
                            Data::new(
                                body.file_data.clone(),
                                body.start.clone(),
                                body.end.clone(),
                                DataType::Function(Box::new(Function::new(
                                    &arguments,
                                    *body.clone(),
                                    body.start.clone(),
                                    body.end.clone(),
                                    identifier.clone(),
                                    &mut context,
                                ))),
                            ),
                            Type::new(
                                TypeHint::None,
                                body.start.clone(),
                                body.end.clone(),
                                body.file_data.clone(),
                            ),
                            true,
                            identifier,
                        ),
                    ));
                }
                _ => InvalidInstructionInClass::call(&i.start, &i.end, &i.file_data),
            }
        }
        Ok(Self {
            identifier,
            variables,
            start,
            end,
            file_data,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ClassVariable {
    pub(crate) class_name: String,
    pub(crate) context: Context,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) file_data: Rc<FileData>,
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "variables: {}",
            self.variables
                .iter()
                .map(|x| format!("(name: {}, value: {})", x.0, x.1))
                .collect::<String>()
        )
    }
}

impl ClassVariable {
    fn new(
        class: &Class,
        parent: *mut Context,
        args: Vec<Data>,
        (start, end, file_data): (Position, Position, Rc<FileData>),
    ) -> Result<Self, Exception> {
        let mut context = Context::new(parent, unsafe { &*parent }.file_data.clone());
        for variable in class.variables.iter() {
            context.declare_variable(variable.0.clone(), variable.1.clone())?;
        }
        if context
            .variables_defined_in_this_scope
            .contains_key("constructor")
        {
            Context::call_fn_no_std(
                &mut context,
                &"constructor".to_string(),
                args,
                (&start, &end, &file_data),
            )?;
            context
                .variables_defined_in_this_scope
                .remove(&"constructor".to_string());
        }
        Ok(Self {
            class_name: class.identifier.clone(),
            context,
            start,
            end,
            file_data,
        })
    }
}

impl Display for ClassVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} variables: {} classes: {}",
            self.class_name,
            self.context
                .variables_defined_in_this_scope
                .iter()
                .map(|x| format!("{} ", x.0))
                .collect::<String>(),
            self.context.classes
        )
    }
}
