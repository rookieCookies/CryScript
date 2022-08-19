use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use utils::wrap;

use crate::{
    exceptions::{parser_exceptions::InvalidInstructionInClass, Exception},
    parser::data::{Data, DataType},
    AsString, FileData, Position,
};

use self::{
    context::Context,
    function::{Function, Type, TypeHint},
    instructions::{Instruction, InstructionType},
};

mod built_in_functions;
pub mod context;
pub mod function;
pub mod instructions;

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

#[derive(Debug, Clone)]
pub(crate) struct Variable {
    data: DataRef,
    type_hint: Type,
    is_final: bool,
}

impl Variable {
    fn new(data: DataRef, type_hint: Type, is_final: bool) -> Self {
        Self {
            data,
            type_hint,
            is_final,
        }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "data: {} type: {} final: {}",
            self.data.borrow().data_type,
            self.type_hint.type_value,
            self.is_final
        )
    }
}

pub type DataRef = Rc<RefCell<Data>>;

#[derive(Debug, Clone)]
pub(crate) struct Class {
    identifier: String,
    functions: Vec<(String, Rc<RefCell<Data>>)>,
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
        let mut context = Context::new(Some(parent), unsafe { &*parent }.file_data.clone());
        let mut functions = vec![];
        let mut variables = vec![];
        for i in body.into_iter() {
            match i.instruction_type {
                InstructionType::VarAssign {
                    identifier,
                    data,
                    type_hint,
                    is_final,
                } => variables.push((
                    identifier,
                    Variable::new(
                        match data.visit(&mut context)? {
                            crate::Returnable::Return(v) => v,
                            crate::Returnable::Evaluate(v) => v,
                            crate::Returnable::Break(v) => v,
                        },
                        type_hint,
                        is_final,
                    ),
                )),
                InstructionType::FunctionDeclaration {
                    identifier,
                    body,
                    arguments,
                } => {
                    functions.push((
                        identifier.clone(),
                        wrap(Data::new(
                            body.file_data.clone(),
                            body.start.clone(),
                            body.end.clone(),
                            DataType::Function(Box::new(Function::new(
                                &arguments,
                                *body.clone(),
                                body.start,
                                body.end,
                                identifier,
                                &mut context,
                            ))),
                        )),
                    ));
                }
                _ => InvalidInstructionInClass::call(&i.start, &i.end, &i.file_data),
            }
        }
        Ok(Self {
            identifier,
            functions,
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
        args: Vec<DataRef>,
        (start, end, file_data): (Position, Position, Rc<FileData>),
    ) -> Result<Self, Exception> {
        let mut context = Context::new(Some(parent), unsafe { &*parent }.file_data.clone());
        let none = Type::new(
            TypeHint::None,
            class.start.clone(),
            class.end.clone(),
            class.file_data.clone(),
        );
        for variable in class.variables.iter() {
            context.declare_variable(variable.0.clone(), variable.1.clone())?;
        }
        for function in class.functions.iter() {
            context.add_var(function.0.clone(), none.clone(), true, function.1.clone());
        }
        if context.variables.map.contains_key("constructor") {
            Context::call_fn_no_std(
                &mut context,
                &"constructor".to_string(),
                args,
                (&start, &end, &file_data),
            )?;
            context.variables.map.remove(&"constructor".to_string());
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
            self.class_name, self.context.variables, self.context.classes
        )
    }
}
