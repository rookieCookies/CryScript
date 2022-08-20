use std::rc::Rc;

use crate::{interpreter::function::TypeHint, parser::data::DataType, FileData, Position};

use super::{Exception, PositionException, EXCEPTION};

pub struct FailedToOpenFile;

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
            "failed to open file",
            &format!(
                "failed to open file at {}, maybe it doesn't exist",
                data_type.data_type()
            ),
            &EXCEPTION,
        )
    }
}
