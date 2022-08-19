use std::{cell::RefCell, rc::Rc};

use utils::wrap;

use crate::{
    exceptions::{
        interpreter_exceptions::{
            AccessUndeclaredClass, AccessUndeclaredFunction, AccessUndeclaredVariable,
            UpdateUndeclaredVariable, VariableIsFinal, VariableIsNotAFunction,
        },
        Exception,
    },
    parser::data::{original_data, Data, DataType},
    run_from_file, run_with_data, AsString, FileData, Position, Returnable, STD_DIR, STD_FILES,
};

use super::{
    built_in_functions::BuiltInFunctions,
    function::{Function, Type, TypeHint},
    Class, ClassVariable, DataRef, Table, Variable,
};

#[derive(Debug, Clone)]
pub struct Context {
    pub parent: Option<*mut Context>,
    pub(crate) variables: Table<Rc<RefCell<Variable>>>,
    pub(crate) classes: Table<Rc<Class>>,
    pub(crate) file_data: Rc<FileData>,
    imported_files: Vec<String>,
}

impl Context {
    pub(crate) fn new(parent: Option<*mut Context>, file_data: Rc<FileData>) -> Context {
        Context {
            parent,
            variables: Table::new(),
            file_data,
            imported_files: vec![],
            classes: Table::new(),
        }
    }

    pub(crate) fn declare_function(&mut self, function: Function) {
        let type_hint = Type::new(
            TypeHint::None,
            function.start.clone(),
            function.end.clone(),
            self.file_data.clone(),
        );
        self.variables.map.insert(
            function.identifier.clone(),
            wrap(Variable::new(
                wrap(Data::new(
                    self.file_data.clone(),
                    function.start.clone(),
                    function.end.clone(),
                    DataType::Function(Box::new(function)),
                )),
                type_hint,
                true,
            )),
        );
    }

    pub(crate) fn add_var(
        &mut self,
        identifier: String,
        type_hint: Type,
        is_final: bool,
        data: Rc<RefCell<Data>>,
    ) {
        self.variables
            .map
            .insert(identifier, wrap(Variable::new(data, type_hint, is_final)));
    }

    pub(crate) fn declare_class(&mut self, class: Class) {
        self.classes
            .map
            .insert(class.identifier.clone(), Rc::new(class));
    }

    pub(crate) fn call_function(
        context: *mut Context,
        identifier: &String,
        args: Vec<DataRef>,
        (start, end, file_data): (&Position, &Position, &Rc<FileData>),
    ) -> Result<DataRef, Exception> {
        BuiltInFunctions::run(context, identifier, args, (start, end, file_data))
    }

    pub(crate) fn call_fn_no_std(
        context: *mut Context,
        identifier: &String,
        args: Vec<DataRef>,
        (start, end, file_data): (&Position, &Position, &Rc<FileData>),
    ) -> Result<DataRef, Exception> {
        match unsafe { &*context }.variables.map.get(&identifier.clone()) {
            Some(v) => match &original_data(&v.borrow().data).borrow().data_type {
                DataType::Function(v) => v,
                _ => {
                    return Err(VariableIsNotAFunction::call(
                        start, end, file_data, identifier,
                    ))
                }
            }
            .call(context, file_data.clone(), args),
            None => match unsafe { &mut *context }.parent {
                Some(v) => Context::call_function(v, identifier, args, (start, end, file_data)),
                None => Err(AccessUndeclaredFunction::call(
                    start, end, file_data, identifier,
                )),
            },
        }
    }

    pub(crate) fn call_override_class_fn(
        context: *mut Context,
        identifier: &String,
        args: Vec<DataRef>,
        (start, end, file_data): (&Position, &Position, &Rc<FileData>),
    ) -> Result<Rc<RefCell<Data>>, Exception> {
        if unsafe { &*context }.variables.map.contains_key(identifier) {
            Context::call_fn_no_std(context, identifier, args, (start, end, file_data))
        } else {
            Err(AccessUndeclaredFunction::call(
                start, end, file_data, identifier,
            ))
        }
    }

    pub(crate) fn access_data(
        &self,
        identifier: &String,
        (start, end, file_data): (&Position, &Position, &Rc<FileData>),
    ) -> Result<DataRef, Exception> {
        Ok(self
            .access_variable(identifier, (start, end, file_data))?
            .try_borrow()
            .unwrap()
            .data
            .clone())
    }

    fn access_variable(
        &self,
        identifier: &String,
        (start, end, file_data): (&Position, &Position, &Rc<FileData>),
    ) -> Result<Rc<RefCell<Variable>>, Exception> {
        match self.variables.map.get(&identifier.clone()) {
            Some(v) => Ok(v.clone()),
            None => match &self.parent {
                Some(v) => unsafe { &**v }.access_variable(identifier, (start, end, file_data)),
                None => Err(AccessUndeclaredVariable::call(
                    start, end, file_data, identifier,
                )),
            },
        }
    }

    fn access_class(
        &self,
        identifier: &String,
        (start, end, file_data): (&Position, &Position, &Rc<FileData>),
    ) -> Result<Rc<Class>, Exception> {
        match self.classes.map.get(&identifier.clone()) {
            Some(v) => Ok(v.clone()),
            None => match &self.parent {
                Some(v) => unsafe { &**v }.access_class(identifier, (start, end, file_data)),
                None => Err(AccessUndeclaredClass::call(
                    start, end, file_data, identifier,
                )),
            },
        }
    }

    pub(crate) fn new_class(
        parent: *mut Context,
        identifier: &String,
        args: Vec<DataRef>,
        (start, end, file_data): (&Position, &Position, &Rc<FileData>),
    ) -> Result<Rc<RefCell<Data>>, Exception> {
        let class = unsafe { &mut *parent }.access_class(identifier, (start, end, file_data))?;
        let class_var = ClassVariable::new(
            &*class,
            parent,
            args,
            (start.clone(), end.clone(), file_data.clone()),
        )?;

        Ok(wrap(Data::new(
            file_data.clone(),
            start.clone(),
            end.clone(),
            DataType::Class(Box::new(class_var)),
        )))
    }

    pub(crate) fn update_variable(
        context: *mut Context,
        identifier: &String,
        data: DataRef,
        (start, end, file_data): (&Position, &Position, &Rc<FileData>),
    ) -> Result<(), Exception> {
        let ctx = unsafe { &mut *context };
        match ctx.variables.map.contains_key(identifier) {
            true => {
                let var = &mut ctx.variables.map.get_mut(identifier).unwrap().borrow_mut();
                if var.is_final {
                    return Err(VariableIsFinal::call(start, end, file_data, identifier));
                }
                original_data(&data).borrow().data_type.is_of_type(
                    &var.type_hint,
                    identifier,
                    (start, end, file_data),
                )?;
                var.data = data;
            }
            false => match &ctx.parent {
                Some(v) => Context::update_variable(*v, identifier, data, (start, end, file_data))?,
                None => {
                    return Err(UpdateUndeclaredVariable::call(
                        start, end, file_data, identifier,
                    ))
                }
            },
        }
        Ok(())
    }

    pub(crate) fn assign_variable(
        &mut self,
        identifier: String,
        data: DataRef,
        type_hint: Type,
        is_final: bool,
        (start, end, file_data): (&Position, &Position, &Rc<FileData>),
    ) -> Result<(), Exception> {
        original_data(&data).borrow().data_type.is_of_type(
            &type_hint,
            &identifier,
            (start, end, file_data),
        )?;
        match &type_hint.type_value {
            TypeHint::Class(i) => {
                if !self.has_class(i) {
                    return Err(AccessUndeclaredClass::call(start, end, file_data, i));
                }
            }
            _ => {}
        }
        self.declare_variable(identifier, Variable::new(data, type_hint, is_final))
    }

    pub(crate) fn declare_variable(
        &mut self,
        identifier: String,
        variable: Variable,
    ) -> Result<(), Exception> {
        self.variables.map.insert(identifier, wrap(variable));
        Ok(())
    }

    pub(crate) fn import_file(
        context: *mut Context,
        file_path: &str,
        file_data: Rc<FileData>,
    ) -> Result<Returnable, Exception> {
        if STD_FILES.contains(&file_path) {
            Context::import_data(
                context,
                Rc::new(FileData::new(
                    STD_DIR
                        .get_file(format!("{}.cry", file_path))
                        .unwrap()
                        .contents_utf8()
                        .unwrap()
                        .to_string(),
                    file_path.to_string(),
                )),
            )
        } else if unsafe { &*context }
            .imported_files
            .contains(&file_path.to_string())
        {
            Ok(Returnable::Evaluate(wrap(Data::null(file_data))))
        } else {
            unsafe { &mut *context }
                .imported_files
                .push(file_path.to_string());
            run_from_file(file_path, context)
        }
    }

    pub(crate) fn import_data(
        context: *mut Context,
        file_data: Rc<FileData>,
    ) -> Result<Returnable, Exception> {
        if unsafe { &*context }
            .imported_files
            .contains(&file_data.path.to_string())
        {
            Ok(Returnable::Evaluate(wrap(Data::null(file_data))))
        } else {
            unsafe { &mut *context }
                .imported_files
                .push(file_data.path.to_string());
            run_with_data(file_data, context)
        }
    }

    pub(crate) fn has_file(&self, file_path: &str) -> bool {
        if self.imported_files.contains(&file_path.to_string()) {
            true
        } else if self.parent.is_some() {
            unsafe { &*self.parent.unwrap() }.has_file(file_path)
        } else {
            false
        }
    }

    pub(crate) fn has_class(&self, identifier: &String) -> bool {
        if self.classes.map.contains_key(identifier) {
            true
        } else if self.parent.is_some() {
            unsafe { &*self.parent.unwrap() }.has_class(identifier)
        } else {
            false
        }
    }

    pub(crate) fn depth(&self) -> usize {
        match self.parent.as_ref() {
            Some(v) => unsafe { &**v }.depth() + 1,
            None => 0,
        }
    }
}

impl AsString for Rc<RefCell<Variable>> {
    fn as_string(&self) -> String {
        format!("{}", self.borrow())
    }
}

impl AsString for Rc<Class> {


    
    fn as_string(&self) -> String {
        format!("{}", self)
    }
}
