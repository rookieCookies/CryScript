use std::{fmt::Display, path::Path, rc::Rc};

use utils::wrap;

use crate::{
    exceptions::{
        interpreter_exceptions::{CantRunInContext, InvalidFilePath, ReturnFromRoot},
        Exception,
    },
    parser::data::{original_data, Data, DataType},
    run_with_instructions, FileData, Position, Returnable,
};
pub mod binary_op;
pub mod unary_op;

use self::{binary_op::BinaryOperator, unary_op::UnaryOperator};

use super::{
    context::Context,
    function::{Function, Type},
    Class,
};

#[macro_export]
macro_rules! returnable {
    ($expr:expr $(,)?) => {
        match $expr {
            $crate::Returnable::Return(val) => return Ok(crate::Returnable::Return(val)),
            $crate::Returnable::Break(val) => val,
            $crate::Returnable::Evaluate(val) => val,
        }
    };
}

#[derive(Debug, Clone)]
pub(crate) enum InstructionType {
    Pass,
    BinaryOperation {
        left: Box<Instruction>,
        right: Box<Instruction>,
        operator: BinaryOperator,
    },
    UnaryOperation {
        value: Box<Instruction>,
        operator: UnaryOperator,
    },
    UseStatement {
        file_path: String,
    },
    VarAssign {
        identifier: String,
        data: Box<Instruction>,
        type_hint: Type,
        is_final: bool,
    },
    VarUpdate {
        identifier: String,
        data: Box<Instruction>,
    },
    VarAccess {
        identifier: String,
    },
    IfStatement {
        condition: Option<Box<Instruction>>,
        body: Box<Instruction>,
        else_value: Option<Box<Instruction>>,
    },
    WhileStatement {
        condition: Box<Instruction>,
        body: Box<Instruction>,
    },
    FunctionDeclaration {
        identifier: String,
        body: Box<Instruction>,
        arguments: Vec<(String, Type, Option<Instruction>)>,
    },
    FunctionCall {
        identifier: String,
        arguments: Vec<Instruction>,
    },
    Section {
        body: Vec<Instruction>,
    },
    ReturnStatement {
        value: Box<Instruction>,
    },
    BreakStatement {
        value: Box<Instruction>,
    },
    DoCatch {
        do_body: Box<Instruction>,
        catch_body: Box<Instruction>,
    },
    ClassDeclaration {
        identifier: String,
        body: Box<Instruction>,
    },
    ClassInstantiation {
        identifier: String,
        constructor_arguments: Vec<Instruction>,
    },
    InContextOf {
        context_of: Box<Instruction>,
        run: Box<Instruction>,
    },
    As {
        convert_type: Type,
        value: Box<Instruction>,
    },
    DocComment {
        comment: String,
        value: Box<Instruction>,
    },
    Data(Data),
}

#[derive(Debug, Clone)]
pub(crate) struct Instruction {
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) file_data: Rc<FileData>,
    pub(crate) instruction_type: InstructionType,
}

impl Instruction {
    pub(crate) fn new(
        start: Position,
        end: Position,
        file_data: Rc<FileData>,
        instruction_type: InstructionType,
    ) -> Self {
        Self {
            start,
            end,
            file_data,
            instruction_type,
        }
    }

    pub(crate) fn visit(&self, context_ptr: *mut Context) -> Result<Returnable, Exception> {
        let context_ref = unsafe { &mut *context_ptr };
        match &self.instruction_type {
            InstructionType::Data(data) => Ok(Returnable::Evaluate(wrap(data.clone()))),
            InstructionType::BinaryOperation {
                left,
                right,
                operator,
            } => {
                let left = returnable!(left.visit(context_ptr)?);
                let right = returnable!(right.visit(context_ptr)?);

                let data1_ref = original_data(&left);
                let data2_ref = original_data(&right);

                Ok(Returnable::Evaluate(match operator {
                    BinaryOperator::AddAssign => {
                        let data = Data::add(
                            left.clone(),
                            right,
                            &*data1_ref.borrow(),
                            &*data2_ref.borrow(),
                        )?;
                        *data1_ref.borrow_mut() = data;
                        left
                    }
                    BinaryOperator::RemoveAssign => {
                        let data = Data::sub(
                            left.clone(),
                            right,
                            &*data1_ref.borrow(),
                            &*data2_ref.borrow(),
                        )?;
                        *data1_ref.borrow_mut() = data;
                        left
                    }
                    BinaryOperator::DivideAssign => {
                        let data = Data::div(
                            left.clone(),
                            right,
                            &*data1_ref.borrow(),
                            &*data2_ref.borrow(),
                        )?;
                        *data1_ref.borrow_mut() = data;
                        left
                    }
                    BinaryOperator::MultiplyAssign => {
                        let data = Data::mul(
                            left.clone(),
                            right,
                            &*data1_ref.borrow(),
                            &*data2_ref.borrow(),
                        )?;
                        *data1_ref.borrow_mut() = data;
                        left
                    }
                    BinaryOperator::PowerAssign => {
                        let data = Data::pow(
                            left.clone(),
                            right,
                            &*data1_ref.borrow(),
                            &*data2_ref.borrow(),
                        )?;
                        *data1_ref.borrow_mut() = data;
                        left
                    }
                    _ => wrap(operator.operate(
                        left,
                        right,
                        &*data1_ref.borrow(),
                        &*data2_ref.borrow(),
                    )?),
                }))
            }
            InstructionType::UnaryOperation { value, operator } => {
                Ok(Returnable::Evaluate(wrap({
                    let data_ref = returnable!(value.visit(context_ptr)?);
                    operator.operate(data_ref.clone(), &*original_data(&data_ref).borrow())?
                })))
            }
            InstructionType::UseStatement { file_path } => {
                if !file_path.starts_with("std_") && !Path::new(file_path).exists() {
                    return Err(InvalidFilePath::call(
                        &self.start,
                        &self.end,
                        &self.file_data,
                        file_path,
                    ));
                }
                Ok(Returnable::Evaluate(returnable!(Context::import_file(
                    context_ptr,
                    file_path,
                    self.file_data.clone()
                )?)))
            }
            InstructionType::VarAccess { identifier } => Ok(Returnable::Evaluate({
                let original = context_ref
                    .access_data(identifier, (&self.start, &self.end, &self.file_data))?;
                wrap(Data::new(
                    self.file_data.clone(),
                    self.start.clone(),
                    self.end.clone(),
                    DataType::Reference(original),
                ))
            })),
            InstructionType::VarAssign {
                identifier,
                data,
                type_hint,
                is_final,
            } => {
                let data = returnable!(data.visit(context_ptr)?); // To prevent runtime borrow errors
                context_ref.assign_variable(
                    identifier.clone(),
                    data,
                    type_hint.clone(),
                    *is_final,
                    (&self.start, &self.end, &self.file_data),
                )?;
                Ok(Returnable::Evaluate(wrap(Data::new(
                    self.file_data.clone(),
                    self.start.clone(),
                    self.end.clone(),
                    DataType::Null,
                ))))
            }
            InstructionType::VarUpdate { identifier, data } => {
                let data = returnable!(data.visit(context_ptr)?); // To prevent runtime borrow errors
                Context::update_variable(
                    context_ptr,
                    identifier,
                    data,
                    (&self.start, &self.end, &self.file_data),
                )?;
                Ok(Returnable::Evaluate(wrap(Data::new(
                    self.file_data.clone(),
                    self.start.clone(),
                    self.end.clone(),
                    DataType::Null,
                ))))
            }
            InstructionType::IfStatement {
                condition,
                body,
                else_value,
            } => {
                if (condition.is_some()
                    && returnable!(condition.as_ref().unwrap().visit(context_ptr)?)
                        .borrow()
                        .as_bool()?)
                    || condition.is_none()
                {
                    body.visit(context_ptr)
                } else if else_value.is_some() {
                    else_value.as_ref().unwrap().visit(context_ptr)
                } else {
                    Ok(Returnable::Evaluate(wrap(Data::new(
                        self.file_data.clone(),
                        self.start.clone(),
                        self.end.clone(),
                        DataType::Null,
                    ))))
                }
            }
            InstructionType::WhileStatement { condition, body } => {
                let mut return_value = Returnable::Evaluate(wrap(Data::new(
                    self.file_data.clone(),
                    self.start.clone(),
                    self.end.clone(),
                    DataType::Null,
                )));
                while returnable!(condition.visit(context_ptr)?)
                    .try_borrow()
                    .unwrap()
                    .as_bool()?
                {
                    return_value = match body.visit(context_ptr)? {
                        Returnable::Return(v) => return Ok(Returnable::Return(v)),
                        Returnable::Evaluate(v) => Returnable::Evaluate(v),
                        Returnable::Break(v) => {
                            return_value = Returnable::Evaluate(v);
                            break;
                        }
                    }
                }
                Ok(return_value)
            }
            InstructionType::Section { body } => Ok(run_with_instructions(
                body,
                &mut Context::new(Some(context_ptr), self.file_data.clone()),
                self.file_data.clone(),
            )?),
            InstructionType::ReturnStatement { value } => {
                if context_ref.depth() == 0 {
                    return Err(ReturnFromRoot::call(
                        &self.start,
                        &self.end,
                        &self.file_data,
                    ));
                }
                let v = value.visit(context_ptr)?;
                Ok(Returnable::Return(returnable!(v)))
            }
            InstructionType::BreakStatement { value } => {
                if context_ref.depth() == 0 {
                    return Err(ReturnFromRoot::call(
                        &self.start,
                        &self.end,
                        &self.file_data,
                    ));
                }
                let v = value.visit(context_ptr)?;
                Ok(Returnable::Break(returnable!(v)))
            }
            InstructionType::FunctionDeclaration {
                identifier,
                body,
                arguments,
            } => {
                context_ref.declare_function(Function::new(
                    arguments,
                    *body.clone(),
                    self.start.clone(),
                    self.end.clone(),
                    identifier.clone(),
                    context_ptr,
                ));
                Ok(Returnable::Evaluate(wrap(Data::new(
                    self.file_data.clone(),
                    self.start.clone(),
                    self.end.clone(),
                    DataType::Null,
                ))))
            }
            InstructionType::FunctionCall {
                identifier,
                arguments,
            } => Ok(Returnable::Evaluate(Context::call_function(
                context_ptr,
                identifier,
                arguments
                    .iter()
                    .map(|x| match x.visit(context_ptr) {
                        Ok(v) => v.unwrap(),
                        Err(v) => v.run(),
                    })
                    .collect(),
                (&self.start, &self.end, &self.file_data),
            )?)),
            InstructionType::DoCatch {
                catch_body,
                do_body,
            } => match do_body.visit(context_ptr) {
                Ok(v) => Ok(v),
                Err(_) => catch_body.visit(context_ptr),
            },
            InstructionType::As {
                convert_type,
                value,
            } => Ok(Returnable::Evaluate(
                returnable!(value.visit(context_ptr)?)
                    .borrow()
                    .convert_to(convert_type)?,
            )),
            InstructionType::DocComment { comment: _, value } => value.visit(context_ptr),
            InstructionType::ClassDeclaration { identifier, body } => {
                let class = Class::new(
                    identifier.clone(),
                    (self.start.clone(), self.end.clone(), self.file_data.clone()),
                    match &body.instruction_type {
                        InstructionType::Section { body } => body.clone(),
                        _ => panic!(),
                    },
                    context_ptr,
                )?;
                context_ref.declare_class(class);
                Ok(Returnable::Evaluate(wrap(Data::new(
                    self.file_data.clone(),
                    self.start.clone(),
                    self.end.clone(),
                    DataType::Null,
                ))))
            }
            InstructionType::ClassInstantiation {
                identifier,
                constructor_arguments,
            } => Ok(Returnable::Evaluate(Context::new_class(
                context_ptr,
                identifier,
                constructor_arguments
                    .iter()
                    .map(|x| match x.visit(context_ptr) {
                        Ok(v) => v.unwrap(),
                        Err(v) => v.run(),
                    })
                    .collect(),
                (
                    &self.start.clone(),
                    &self.end.clone(),
                    &self.file_data.clone(),
                ),
            )?)),
            InstructionType::InContextOf { context_of, run } => {
                let context_of = returnable!(context_of.visit(context_ptr)?);
                let reference = context_of.borrow().data_type.original();
                match reference {
                    DataType::Class(mut v) => {
                        let parent = v.context.parent.clone();
                        v.context.parent = Some(context_ptr);
                        let return_value = run.visit(&mut v.context);
                        v.context.parent = parent;
                        return_value
                    }
                    _ => Err(CantRunInContext::call(
                        &self.start,
                        &self.end,
                        &self.file_data,
                        &reference,
                    )),
                }
            }
            InstructionType::Pass => Ok(Returnable::Evaluate(wrap(Data::new(
                self.file_data.clone(),
                self.start.clone(),
                self.end.clone(),
                DataType::Null,
            )))),
        }
    }
}

fn arrow_with_depth(depth: usize) -> String {
    if depth == 0 {
        String::with_capacity(0)
    } else {
        format!("{}> ", "-".repeat((depth as i32 - 1).max(0) as usize))
    }
}

impl Display for InstructionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ",
            match self {
                InstructionType::BinaryOperation { .. } => "binary op",
                InstructionType::UnaryOperation { .. } => "unary op",
                InstructionType::UseStatement { .. } => "use",
                InstructionType::VarAssign { .. } => "assign var",
                InstructionType::VarUpdate { .. } => "update var",
                InstructionType::VarAccess { .. } => "access var",
                InstructionType::IfStatement { .. } => "if",
                InstructionType::WhileStatement { .. } => "while",
                InstructionType::FunctionDeclaration { .. } => "declare function",
                InstructionType::FunctionCall { .. } => "call function",
                InstructionType::Section { .. } => "section",
                InstructionType::ReturnStatement { .. } => "return",
                InstructionType::BreakStatement { .. } => "break",
                InstructionType::DoCatch { .. } => "do catch",
                InstructionType::ClassDeclaration { .. } => "declare class",
                InstructionType::ClassInstantiation { .. } => "new class",
                InstructionType::InContextOf { .. } => "in context of",
                InstructionType::As { .. } => "as",
                InstructionType::DocComment { .. } => "doc comment",
                InstructionType::Data(_) => "data",
                InstructionType::Pass => "pass",
            }
        )
    }
}
