use std::rc::Rc;

use crate::{interpreter::type_hint::TypeHint, parser::data::DataType, FileData, Position};

use super::{Exception, PositionException, EXCEPTION};

pub struct InvalidBinaryOperation;

impl InvalidBinaryOperation {
    pub(crate) fn call(
        start: &Position,
        end: &Position,
        file_data: &Rc<FileData>,
        (num1, num2): (&DataType, &DataType),
        operation: &str,
    ) -> Exception {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "binary operation type error",
            format!(
                "invalid binary operation ({}) between data of type {} and {}",
                operation,
                num1.data_type(),
                num2.data_type()
            )
            .as_str(),
            &EXCEPTION,
        )
    }
}

pub struct TypeConversion;

impl TypeConversion {
    pub(crate) fn call(
        start: &Position,
        end: &Position,
        file_data: &Rc<FileData>,
        current_type: &DataType,
        convert_type: &str,
    ) -> Exception {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "type conversion",
            format!(
                "can't convert a {} to a {}",
                current_type.data_type(),
                convert_type
            )
            .as_str(),
            &EXCEPTION,
        )
    }
}

pub struct IntegerDivisionByZero;

impl IntegerDivisionByZero {
    pub(crate) fn call(start: &Position, end: &Position, file_data: &Rc<FileData>) -> Exception {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "division by zero",
            "dividing an integer with 0 will result in an unknown value",
            &EXCEPTION,
        )
    }
}

pub struct InvalidFilePath;

impl InvalidFilePath {
    pub(crate) fn call(
        start: &Position,
        end: &Position,
        file_data: &Rc<FileData>,
        file_path: &String,
    ) -> Exception {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "invalid file path",
            format!("unable to locate a file at \"{}\"", file_path,).as_str(),
            &EXCEPTION,
        )
    }
}

pub struct AccessUndeclaredVariable;

impl AccessUndeclaredVariable {
    pub(crate) fn call(
        start: &Position,
        end: &Position,
        file_data: &Rc<FileData>,
        identifier: &String,
    ) -> Exception {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "access undeclared variable",
            format!(
                "can't access {} since it does not exist in the current scope",
                identifier
            )
            .as_str(),
            &EXCEPTION,
        )
    }
}

pub struct AccessUndeclaredClass;

impl AccessUndeclaredClass {
    pub(crate) fn call(
        start: &Position,
        end: &Position,
        file_data: &Rc<FileData>,
        identifier: &String,
    ) -> Exception {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "access undeclared class",
            format!(
                "can't access {} since it does not exist in the current scope",
                identifier
            )
            .as_str(),
            &EXCEPTION,
        )
    }
}

pub struct AccessUndeclaredFunction;

impl AccessUndeclaredFunction {
    pub(crate) fn call(
        start: &Position,
        end: &Position,
        file_data: &Rc<FileData>,
        identifier: &String,
    ) -> Exception {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "access undeclared function",
            format!(
                "can't call {} since it does not exist in the current scope",
                identifier
            )
            .as_str(),
            &EXCEPTION,
        )
    }
}

pub struct UpdateUndeclaredVariable;

impl UpdateUndeclaredVariable {
    pub(crate) fn call(
        start: &Position,
        end: &Position,
        file_data: &Rc<FileData>,
        identifier: &String,
    ) -> Exception {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "update undeclared variable",
            format!(
                "can't update {} since it does not exist in the current scope",
                identifier
            )
            .as_str(),
            &EXCEPTION,
        )
    }
}

pub struct ReturnFromRoot;

impl ReturnFromRoot {
    pub(crate) fn call(start: &Position, end: &Position, file_data: &Rc<FileData>) -> Exception {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "return from root",
            "can't return from a root context, try exit() to quit the application",
            &EXCEPTION,
        )
    }
}

pub struct InvalidAmountOfArguments;

impl InvalidAmountOfArguments {
    pub(crate) fn call(
        start: &Position,
        end: &Position,
        file_data: &Rc<FileData>,
        function_name: &String,
        argument_count: usize,
        provided_count: usize,
    ) -> Exception {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "invalid argument amount",
            format!(
                "function {} requires {} arguments but {} arguments were provided",
                function_name, argument_count, provided_count
            )
            .as_str(),
            &EXCEPTION,
        )
    }
}

pub struct InvalidArgumentType;

impl InvalidArgumentType {
    pub(crate) fn call(
        start: &Position,
        end: &Position,
        file_data: &Rc<FileData>,
        argument_name: &String,
        type_hint: &TypeHint,
        provided_argument_type: &DataType,
    ) -> Exception {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "unexpected argument type",
            format!(
                "argument {} has the type hint {} but the provided value was of type {}",
                argument_name,
                type_hint,
                provided_argument_type.data_type()
            )
            .as_str(),
            &EXCEPTION,
        )
    }
}

pub struct InvalidType;

impl InvalidType {
    pub(crate) fn call(
        start: &Position,
        end: &Position,
        file_data: &Rc<FileData>,
        identifier: &String,
        type_hint: &TypeHint,
        provided_argument_type: &DataType,
    ) -> Exception {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "unexpected variable type",
            format!(
                "variable {} has the type {} but the provided value was of type {}",
                identifier,
                type_hint,
                provided_argument_type.data_type()
            )
            .as_str(),
            &EXCEPTION,
        )
    }
}

pub struct FailedToReadInput;

impl FailedToReadInput {
    pub(crate) fn call(start: &Position, end: &Position, file_data: &Rc<FileData>) -> Exception {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "failed to read input",
            "this is likely a os related issue",
            &EXCEPTION,
        )
    }
}

pub struct VariableIsNotAFunction;

impl VariableIsNotAFunction {
    pub(crate) fn call(
        start: &Position,
        end: &Position,
        file_data: &Rc<FileData>,
        identifier: &str,
    ) -> Exception {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "variable isn't a function",
            &format!("variable {} exists but it is not a function", identifier),
            &EXCEPTION,
        )
    }
}

pub struct VariableIsFinal;

impl VariableIsFinal {
    pub(crate) fn call(
        start: &Position,
        end: &Position,
        file_data: &Rc<FileData>,
        identifier: &str,
    ) -> Exception {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "update final variable",
            &format!(
                "variable {} is declared as final and thus can't be updated",
                identifier
            ),
            &EXCEPTION,
        )
    }
}

pub struct CantRunInContext;

impl CantRunInContext {
    pub(crate) fn call(
        start: &Position,
        end: &Position,
        file_data: &Rc<FileData>,
        data_type: &DataType,
    ) -> Exception {
        PositionException::call(
            start,
            end,
            start,
            file_data,
            "data type doesn't have a context",
            &format!(
                "data type {} doesn't have a context you can access",
                data_type.data_type()
            ),
            &EXCEPTION,
        )
    }
}
