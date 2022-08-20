#![allow(clippy::cast_ref_to_mut)]
use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    exceptions::{
        interpreter_exceptions::{
            IntegerDivisionByZero, InvalidBinaryOperation, InvalidType, TypeConversion,
        },
        parser_exceptions::UnexpectedToken,
        Exception,
    },
    interpreter::{
        context::Context,
        function::Function,
        type_hint::{Type, TypeHint},
        ClassVariable,
    },
    lexer::token::{Token, TokenType},
    variables::VariableReference,
    FileData, Position,
};

#[derive(Clone, Debug)]
pub(crate) enum DataType {
    Integer(i32),
    Float(f32),
    String(String),
    Function(Box<Function>),
    Class(Box<ClassVariable>),
    Reference(VariableReference),
    Null,
}

impl DataType {
    pub fn data_type(&self) -> String {
        match self {
            DataType::Integer(_) => "integer".to_string(),
            DataType::Float(_) => "float".to_string(),
            DataType::String(_) => "string".to_string(),
            DataType::Null => "null".to_string(),
            DataType::Function(_) => "function".to_string(),
            DataType::Class(v) => v.class_name.clone(),
            DataType::Reference(v) => format!("ref({})", (**v).data.data_type.data_type()),
        }
    }

    pub fn is_of_type(
        &self,
        t: &Type,
        identifier: &String,
        (start, end, file_data): (&Position, &Position, &Rc<FileData>),
    ) -> Result<bool, Exception> {
        match (&t.type_value, &self) {
            (TypeHint::Integer, crate::parser::data::DataType::Integer(_))
            | (TypeHint::String, crate::parser::data::DataType::String(_))
            | (TypeHint::Float, crate::parser::data::DataType::Float(_))
            | (TypeHint::None, _) => Ok(true),
            (TypeHint::Class(identifier), crate::parser::data::DataType::Class(class)) => {
                Ok(&class.class_name == identifier)
            }
            (_, Self::Null) => Ok(true),
            _ => Err(InvalidType::call(
                start,
                end,
                file_data,
                identifier,
                &t.type_value,
                self,
            )),
        }
    }

    pub fn original(&self) -> Self {
        match self {
            DataType::Reference(v) => (**v).data.data_type.original(),
            _ => self.clone(),
        }
    }
}

impl From<bool> for DataType {
    fn from(v: bool) -> Self {
        match v {
            true => DataType::Integer(1),
            false => DataType::Integer(0),
        }
    }
    // fn into(self) -> DataType {
    //     match self {
    //         true => DataType::Integer(1),
    //         false => DataType::Integer(0),
    //     }
    // }
}

// TODO: Maybe try to not clone these values IDK
impl From<&Token> for DataType {
    fn from(value: &Token) -> Self {
        match &value.token_type {
            TokenType::Integer(value) => DataType::Integer(*value),
            TokenType::Float(value) => DataType::Float(*value),
            TokenType::String(value) => DataType::String(value.clone()),
            TokenType::Bool(value) => DataType::Integer(if *value { 1 } else { 0 }),
            TokenType::Null => DataType::Null,
            _ => UnexpectedToken::call(
                value.fetch(),
                "a data type (example: 10)",
                value.token_type.to_string().as_str(),
            ),
        }
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Integer(v) => write!(f, "{}", v),
            DataType::Float(v) => write!(f, "{}", v),
            DataType::String(v) => write!(f, "{}", v),
            DataType::Function(v) => write!(f, "{}", v.identifier),
            DataType::Reference(v) => write!(f, "{}", (**v).data.data_type),
            DataType::Null => write!(f, "null"),
            DataType::Class(v) => write!(f, "{}", v.class_name),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Data {
    pub(crate) file_data: Rc<FileData>,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) data_type: DataType,
}

#[derive(Clone, Debug)]
pub struct DataReference {
    pub(crate) file_data: Rc<FileData>,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) original_data: Rc<RefCell<Data>>,
}

impl DataReference {
    pub(crate) fn new(
        file_data: Rc<FileData>,
        start: Position,
        end: Position,
        original_data: Rc<RefCell<Data>>,
    ) -> Self {
        Self {
            file_data,
            start,
            end,
            original_data,
        }
    }
}

impl Data {
    pub(crate) fn new(
        file_data: Rc<FileData>,
        start: Position,
        end: Position,
        data_type: DataType,
    ) -> Self {
        Self {
            file_data,
            start,
            end,
            data_type,
        }
    }

    pub fn as_bool(&self) -> Result<bool, Exception> {
        Ok(match self.data_type.clone() {
            DataType::Integer(v) => v > 0,
            DataType::Float(v) => v > 0.,
            DataType::String(v) => !v.is_empty(),
            DataType::Function(_) => false,
            DataType::Null => false,
            DataType::Reference(v) => (*v).data.as_bool()?,
            DataType::Class(mut v) => Context::call_function(
                &mut v.context,
                &"as_bool".to_string(),
                vec![],
                (&v.start, &v.end, &v.file_data),
            )?
            .as_bool()?,
        })
    }

    pub fn original(&self) -> &Data {
        match &self.data_type {
            DataType::Reference(v) => (*v).data.original(),
            _ => self,
        }
    }

    pub fn original_mut(&mut self) -> &mut Data {
        match &self.data_type {
            DataType::Reference(v) => unsafe {
                &mut *((*v).data.original() as *const Data as *mut Data)
            },
            _ => self,
        }
    }
}

impl From<&Token> for Data {
    fn from(v: &Token) -> Self {
        Data::new(
            v.file_data.clone(),
            v.start.clone(),
            v.end.clone(),
            DataType::from(v),
        )
    }
}

impl Data {
    #[inline(always)]
    pub fn eq(&self, other: &Data) -> Result<Data, Exception> {
        Ok(Data::new(
            self.file_data.clone(),
            self.start.clone(),
            other.end.clone(),
            DataType::Integer(if data_eq(self, other)? { 1 } else { 0 }),
        ))
    }

    #[inline(always)]
    pub fn ne(&self, other: &Data) -> Result<Data, Exception> {
        Ok(Data::new(
            self.file_data.clone(),
            self.start.clone(),
            other.end.clone(),
            DataType::Integer(if !data_eq(self, other)? { 1 } else { 0 }),
        ))
    }

    #[inline(always)]
    pub fn gt(&self, other: &Data) -> Result<Data, Exception> {
        Ok(Data::new(
            self.file_data.clone(),
            self.start.clone(),
            other.end.clone(),
            DataType::Integer(if data_gt(self, other)? { 1 } else { 0 }),
        ))
    }

    #[inline(always)]
    pub fn geq(&self, other: &Data) -> Result<Data, Exception> {
        Ok(Data::new(
            self.file_data.clone(),
            self.start.clone(),
            other.end.clone(),
            DataType::Integer(if data_gt(self, other)? || data_eq(self, other)? {
                1
            } else {
                0
            }),
        ))
    }

    #[inline(always)]
    pub fn lt(&self, other: &Data) -> Result<Data, Exception> {
        Ok(Data::new(
            self.file_data.clone(),
            self.start.clone(),
            other.end.clone(),
            DataType::Integer(if !data_gt(self, other)? && !data_eq(self, other)? {
                1
            } else {
                0
            }),
        ))
    }

    #[inline(always)]
    pub fn leq(&self, other: &Data) -> Result<Data, Exception> {
        Ok(Data::new(
            self.file_data.clone(),
            self.start.clone(),
            other.end.clone(),
            DataType::Integer(if !data_gt(self, other)? { 1 } else { 0 }),
        ))
    }

    #[inline(always)]
    pub(crate) fn add(
        slf: &Data,
        rhs: &Data,
        data1: &Data,
        data2: &Data,
    ) -> Result<Data, Exception> {
        Ok(Data::new(
            data1.file_data.clone(),
            data1.start.clone(),
            data2.end.clone(),
            match (&data1.data_type, &data2.data_type) {
                (DataType::Integer(n1), DataType::Integer(n2)) => DataType::Integer(n1 + n2),
                (DataType::Integer(n1), DataType::Float(n2)) => DataType::Float(*n1 as f32 + n2),
                (DataType::Integer(n1), DataType::String(n2)) => {
                    DataType::String(format!("{}{}", n1, n2))
                }
                (DataType::Float(n1), DataType::Integer(n2)) => DataType::Float(n1 + *n2 as f32),
                (DataType::Float(n1), DataType::Float(n2)) => DataType::Float(n1 + n2),
                (DataType::String(_), _) | (_, DataType::String(_)) => DataType::String({
                    let mut str: String = data1.data_type.to_string();
                    str += data2.data_type.to_string().as_str();
                    str
                }),
                _ => {
                    return Err(InvalidBinaryOperation::call(
                        &slf.start,
                        &rhs.end,
                        &slf.file_data,
                        (&data1.data_type, &data2.data_type),
                        "add",
                    ))
                }
            },
        ))
    }

    #[inline(always)]
    pub(crate) fn sub(
        slf: &Data,
        rhs: &Data,
        data1: &Data,
        data2: &Data,
    ) -> Result<Data, Exception> {
        Ok(Data::new(
            data1.file_data.clone(),
            data1.start.clone(),
            data2.end.clone(),
            match (&data1.data_type, &data2.data_type) {
                (DataType::Integer(n1), DataType::Integer(n2)) => DataType::Integer(n1 - n2),
                (DataType::Integer(n1), DataType::Float(n2)) => DataType::Float(*n1 as f32 - n2),
                (DataType::Float(n1), DataType::Integer(n2)) => DataType::Float(n1 - *n2 as f32),
                (DataType::Float(n1), DataType::Float(n2)) => DataType::Float(n1 - n2),
                _ => {
                    return Err(InvalidBinaryOperation::call(
                        &slf.start,
                        &rhs.end,
                        &slf.file_data,
                        (&data1.data_type, &data2.data_type),
                        "subtract",
                    ))
                }
            },
        ))
    }

    #[inline(always)]
    pub(crate) fn mul(
        slf: &Data,
        rhs: &Data,
        data1: &Data,
        data2: &Data,
    ) -> Result<Data, Exception> {
        Ok(Data::new(
            data1.file_data.clone(),
            data1.start.clone(),
            data2.end.clone(),
            match (&data1.data_type, &data2.data_type) {
                (DataType::Integer(n1), DataType::Integer(n2)) => DataType::Integer(n1 * n2),
                (DataType::Integer(n1), DataType::Float(n2)) => DataType::Float(*n1 as f32 * n2),
                (DataType::Float(n1), DataType::Integer(n2)) => DataType::Float(n1 * *n2 as f32),
                (DataType::Float(n1), DataType::Float(n2)) => DataType::Float(n1 * n2),
                (DataType::String(n1), DataType::Integer(n2)) => {
                    let mut x = String::new();
                    for _ in 0..*n2 {
                        x = format!("{}{}", x, n1)
                    }
                    DataType::String(x)
                }
                _ => {
                    return Err(InvalidBinaryOperation::call(
                        &slf.start,
                        &rhs.end,
                        &slf.file_data,
                        (&data1.data_type, &data2.data_type),
                        "multiply",
                    ))
                }
            },
        ))
    }

    #[inline(always)]
    pub(crate) fn div(
        slf: &Data,
        rhs: &Data,
        data1: &Data,
        data2: &Data,
    ) -> Result<Data, Exception> {
        if match &data1.data_type {
            DataType::Integer(_) => match &data2.data_type {
                DataType::Integer(v) => v == &0,
                _ => false,
            },
            _ => false,
        } {
            return Err(IntegerDivisionByZero::call(
                &rhs.start,
                &rhs.end,
                &rhs.file_data,
            ));
        }
        Ok(Data::new(
            data1.file_data.clone(),
            data1.start.clone(),
            data2.end.clone(),
            match (&data1.data_type, &data2.data_type) {
                (DataType::Integer(n1), DataType::Integer(n2)) => DataType::Integer(n1 / n2),
                (DataType::Integer(n1), DataType::Float(n2)) => DataType::Float(*n1 as f32 / n2),
                (DataType::Float(n1), DataType::Integer(n2)) => DataType::Float(n1 / *n2 as f32),
                (DataType::Float(n1), DataType::Float(n2)) => DataType::Float(n1 / n2),
                _ => {
                    return Err(InvalidBinaryOperation::call(
                        &slf.start,
                        &rhs.end,
                        &slf.file_data,
                        (&data1.data_type, &data2.data_type),
                        "division",
                    ))
                }
            },
        ))
    }

    #[inline(always)]
    pub(crate) fn pow(
        slf: &Data,
        rhs: &Data,
        data1: &Data,
        data2: &Data,
    ) -> Result<Data, Exception> {
        Ok(Data::new(
            data1.file_data.clone(),
            data1.start.clone(),
            data2.end.clone(),
            match (&data1.data_type, &data2.data_type) {
                (DataType::Integer(n1), DataType::Integer(n2)) => {
                    DataType::Integer(n1.pow((*n2) as u32))
                }
                (DataType::Integer(n1), DataType::Float(n2)) => {
                    DataType::Float((*n1 as f32).powf(*n2))
                }
                (DataType::Float(n1), DataType::Integer(n2)) => DataType::Float(n1.powi(*n2)),
                (DataType::Float(n1), DataType::Float(n2)) => DataType::Float(n1.powf(*n2)),
                (DataType::String(n1), DataType::Integer(n2)) => {
                    let mut x = n1.clone();
                    for _ in 0..*n2 - 1 {
                        x = format!("{}{}", x, x)
                    }
                    DataType::String(x)
                }
                _ => {
                    return Err(InvalidBinaryOperation::call(
                        &slf.start,
                        &rhs.end,
                        &slf.file_data,
                        (&data1.data_type, &data2.data_type),
                        "multiply",
                    ))
                }
            },
        ))
    }
}

impl Data {
    pub(crate) fn null(file_data: Rc<FileData>, start: Position, end: Position) -> Data {
        Self {
            file_data,
            start,
            end,
            data_type: DataType::Null,
        }
    }

    pub(crate) fn null_zero(file_data: Rc<FileData>) -> Data {
        Self::null(file_data, Position::new(0), Position::new(0))
    }

    #[inline(always)]
    pub fn convert_to(&self, convert_type: &Type) -> Result<Data, Exception> {
        // TODO: bruh
        let data = self.data_type.original();
        Ok(Data::new(
            self.file_data.clone(),
            self.start.clone(),
            convert_type.end.clone(),
            match (&convert_type.type_value, &data.original()) {
                (TypeHint::Integer, DataType::Integer(_)) => self.data_type.clone(),
                (TypeHint::Integer, DataType::Float(i)) => DataType::Integer(*i as i32),
                (TypeHint::Integer, DataType::String(i)) => DataType::Integer(match i.parse() {
                    Ok(v) => v,
                    Err(_) => return convert_exception(self, convert_type),
                }),
                (TypeHint::Integer, DataType::Null) => {
                    return convert_exception(self, convert_type)
                }
                (TypeHint::String, DataType::Integer(i)) => DataType::String(i.to_string()),
                (TypeHint::String, DataType::Float(i)) => DataType::String(i.to_string()),
                (TypeHint::String, DataType::String(_)) => self.data_type.clone(),
                (TypeHint::String, DataType::Null) => DataType::String("null".to_string()),
                (TypeHint::Float, DataType::Integer(i)) => DataType::Float(*i as f32),
                (TypeHint::Float, DataType::Float(_)) => self.data_type.clone(),
                (TypeHint::Float, DataType::String(i)) => DataType::Float(match i.parse() {
                    Ok(v) => v,
                    Err(_) => return convert_exception(self, convert_type),
                }),
                (TypeHint::Float, DataType::Null) => return convert_exception(self, convert_type),
                _ => return convert_exception(self, convert_type),
            },
        ))
    }
}

fn convert_exception(data: &Data, convert_type: &Type) -> Result<Data, Exception> {
    Err(TypeConversion::call(
        &data.start,
        &convert_type.end,
        &convert_type.file_data,
        &data.data_type.original(),
        convert_type.type_value.to_string().as_str(),
    ))
}

fn data_eq(n1: &Data, n2: &Data) -> Result<bool, Exception> {
    Ok(match (n1.data_type.original(), &n2.data_type.original()) {
        (DataType::Integer(v1), DataType::Integer(v2)) => &v1 == v2,
        (DataType::Integer(v1), DataType::Float(v2)) => v1 as f32 == *v2,
        (DataType::Integer(v1), DataType::String(v2)) => &v1.to_string() == v2,
        (DataType::Float(v1), DataType::Integer(v2)) => v1 == (*v2) as f32,
        (DataType::Float(v1), DataType::Float(v2)) => &v1 == v2,
        (DataType::Float(v1), DataType::String(v2)) => &v1.to_string() == v2,
        (DataType::String(v1), DataType::Integer(v2)) => v1 == v2.to_string(),
        (DataType::String(v1), DataType::Float(v2)) => v1 == v2.to_string(),
        (DataType::String(v1), DataType::String(v2)) => &v1 == v2,
        (DataType::Null, DataType::Null) => true,
        (DataType::Class(_), DataType::Null) | (DataType::Null, DataType::Class(_)) => false,
        (DataType::Class(mut v), _) => Context::call_override_class_fn(
            &mut v.context,
            &"equals".to_string(),
            vec![n2.clone()],
            (&n1.start, &n2.end, &n1.file_data),
        )?
        .as_bool()?,
        _ => false,
    })
}

fn data_gt(n1: &Data, n2: &Data) -> Result<bool, Exception> {
    Ok(match (n1.data_type.original(), &n2.data_type.original()) {
        (DataType::Integer(v1), DataType::Integer(v2)) => &v1 > v2,
        (DataType::Integer(v1), DataType::Float(v2)) => (v1) as f32 > *v2,
        (DataType::Integer(v1), DataType::String(v2)) => v1 > (v2.len() as i32),
        (DataType::Float(v1), DataType::Integer(v2)) => v1 > (*v2) as f32,
        (DataType::Float(v1), DataType::Float(v2)) => &v1 > v2,
        (DataType::Float(v1), DataType::String(v2)) => v1 > (v2.len() as f32),
        (DataType::String(v1), DataType::Integer(v2)) => &(v1.len() as i32) > v2,
        (DataType::String(v1), DataType::Float(v2)) => &(v1.len() as f32) == v2,
        (DataType::String(v1), DataType::String(v2)) => v1.len() > v2.len(),
        (DataType::Class(mut v), _) => Context::call_fn_no_std(
            &mut v.context,
            &"greater".to_string(),
            vec![n2.clone()],
            (&n1.start, &n2.end, &n1.file_data),
        )?
        .as_bool()?,
        _ => false,
    })
}
