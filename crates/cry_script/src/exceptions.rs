pub mod interpreter_exceptions;
pub mod lexer_exceptions;
pub mod parser_exceptions;

use std::rc::Rc;

use colored::{Color, Colorize};
use utils::StringUtils;

use crate::{FileData, Position};

macro_rules! crash {
    () => {
        // panic!()
        std::process::exit(1)
    };
}

const ORANGE: Color = Color::TrueColor {
    r: 255,
    g: 160,
    b: 100,
};

pub const EXCEPTION: ErrorColourScheme = ErrorColourScheme {
    arrow_to_message: Color::BrightRed,
    line_number: ORANGE,
    separator: Color::BrightRed,
    arrow_to_error: Color::BrightRed,
    equal: Color::BrightRed,
    note_colour: ORANGE,
    message: "error",
    message_colour: Color::BrightRed,
};

pub struct ErrorColourScheme {
    arrow_to_message: Color,
    line_number: Color,
    separator: Color,
    arrow_to_error: Color,
    equal: Color,
    message: &'static str,
    message_colour: Color,
    note_colour: Color,
}

#[derive(Clone)]
pub struct Exception {
    string: String,
}

impl Exception {
    pub fn new(string: String) -> Self {
        Self { string }
    }

    pub fn run(&self) -> ! {
        println!("{}", self.string);
        crash!()
    }
}

pub(crate) struct PositionException;

impl PositionException {
    pub fn call(
        start: &Position,
        end: &Position,
        teleport_position: &Position,
        file_data: &Rc<FileData>,
        exception_name: &str,
        note: &str,
        colour_scheme: &ErrorColourScheme,
    ) -> Exception {
        let lines: Vec<&str> = file_data.data.lines().collect();

        let start_line = file_data.data.line_at(start.value).unwrap_or_else(|| {
            panic!(
                "filename: {}, exception: {}, start: {}, end: {}, note: {}",
                file_data.path, exception_name, start.value, end.value, note
            )
        });
        let end_line = file_data.data.line_at(end.value).unwrap_or_else(|| {
            panic!(
                "filename: {}, exception: {}, start: {}, end: {}, note: {}",
                file_data.path, exception_name, start.value, end.value, note
            )
        });

        let biggest_line_number_size = end_line.to_string().len();
        let smallest_line_number_size = start_line.to_string().len();

        let empty_line_number_display =
            format!("{} |", " ".repeat(biggest_line_number_size)).color(colour_scheme.separator);

        let mut message = String::new();
        message.push_str(
            format!(
                "\n{}: {}\n",
                colour_scheme
                    .message
                    .to_string()
                    .bold()
                    .color(colour_scheme.message_colour),
                exception_name.bold()
            )
            .as_str(),
        );

        message.push_str(
            format!(
                "{}{} {}:{}:{}\n",
                " ".repeat(smallest_line_number_size),
                " -->".color(colour_scheme.arrow_to_message),
                file_data.path,
                file_data.data.line_at(teleport_position.value).unwrap() + 1,
                teleport_position.value
                    - file_data
                        .data
                        .start_of_line(file_data.data.line_at(teleport_position.value).unwrap())
                        .unwrap()
                    + 1,
            )
            .as_str(),
        );

        message.push_str(format!("{}\n", empty_line_number_display,).as_str());
        if file_data.path != "std" {
            for (line_number, _) in lines.iter().enumerate().take(end_line + 1).skip(start_line) {
                let current_line: String = lines[line_number].to_string();
                let line_number_display = format!(
                    "{}{} {}",
                    " ".repeat(
                        ((biggest_line_number_size as i32
                            - (line_number + 1).to_string().len() as i32)
                            .max(0))
                        .try_into()
                        .unwrap()
                    ),
                    (line_number + 1)
                        .to_string()
                        .color(colour_scheme.line_number),
                    "|".color(colour_scheme.separator)
                );

                message.push_str(format!("{} {}\n", line_number_display, current_line).as_str());

                message.push_str(
                    format!(
                        "{} {}\n",
                        empty_line_number_display,
                        if line_number == start_line {
                            format!(
                                "{}{}",
                                " ".repeat(
                                    (start.value as i32
                                        - file_data.data.start_of_line(line_number).unwrap() as i32)
                                        .max(0) as usize
                                ),
                                "^".repeat(
                                    file_data
                                        .data
                                        .start_of_line_or(line_number + 1)
                                        .min(end.value + 1)
                                        - start.value
                                )
                                .color(colour_scheme.arrow_to_error)
                            )
                        } else if line_number == end_line {
                            format!(
                                "{}",
                                "^".repeat(
                                    end.value - file_data.data.start_of_line(line_number).unwrap()
                                        + 1
                                )
                                .color(colour_scheme.arrow_to_error)
                            )
                        } else {
                            "^".repeat(current_line.len())
                                .color(colour_scheme.arrow_to_error)
                                .to_string()
                        }
                    )
                    .as_str(),
                )
            }
        }
        let mut note = note.to_string();
        if note.contains('\n') {
            let mut first = false;
            for line in note.clone().split('\n') {
                if !first {
                    first = true;
                    note.push_str(line.trim());
                } else {
                    note.push_str(
                        format!(
                            "\n{}         {}",
                            " ".repeat(smallest_line_number_size),
                            line.trim()
                        )
                        .as_str(),
                    )
                }
            }
        } else {
            note = note.clone()
        }
        // Add the note
        if !note.is_empty() {
            message.push_str(
                format!(
                    "{}{} {} {}\n",
                    " ".repeat(smallest_line_number_size),
                    " =".color(colour_scheme.equal),
                    "note:".bold().color(colour_scheme.note_colour),
                    note
                )
                .as_str(),
            );
        }
        Exception::new(message)
    }
}
