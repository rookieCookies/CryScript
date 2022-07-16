pub mod lexer_errors;
pub mod parser_errors;
pub mod interpreter_errors;

use std::env;

use colored::{Colorize, Color};

use crate::{environment::ENV_DEV_DEBUG_EXCEPTION, utils::{Position, LineOfIndex, FileData}};

const ORANGE : Color = Color::TrueColor { r: 255, g: 160, b: 100 };

pub const EXCEPTION : ErrorColourScheme = ErrorColourScheme {
    arrow_to_message: Color::BrightRed,
    line_number: ORANGE,
    separator: Color::BrightRed,
    arrow_to_error: Color::BrightRed,
    equal: Color::BrightRed,
    note_colour: ORANGE,
    message: "error",
    message_colour: Color::BrightRed,
};

macro_rules! crash {
    () => {
        std::process::exit(1)
    };
}

pub trait Exception {
    fn run(&self) -> !;
}

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

pub struct BasicException(pub String);

impl Exception for BasicException {
    fn run(&self) -> !{
        println!("{}", self.0);
        if env::var(ENV_DEV_DEBUG_EXCEPTION).unwrap() == "true" {
            env::set_var("RUST_BACKTRACE", "1");
            panic!("dev debug")
        }
        crash!()
    }
}

pub struct PositionException<'a> {
    start_position: Position,
    end_position: Position,
    file_data: &'a FileData,
    exception_name: &'a str,
    note: String,
    colour_scheme: ErrorColourScheme,
}

impl<'a> PositionException<'a> {
    pub fn new(start_position: Position, end_position: Position, file_data: &'a FileData, exception_name: &'a str, note: String, colour_scheme: ErrorColourScheme) -> Self { Self { start_position, end_position, file_data, exception_name, note, colour_scheme } }
}

impl Exception for PositionException<'_> {
    fn run(&self) -> ! {
        let mut message = String::new();
        let lines : Vec<&str> = self.file_data.data.lines().collect();
        
        let biggest_line_number_size = (self.end_position.line + 1).to_string().len();
        let smallest_line_number_size = self.start_position.line.to_string().len();
        let empty_line_number_display = format!("{} |", " ".repeat(biggest_line_number_size))
            .color(self.colour_scheme.separator);

        message.push_str(format!(
            "{}: {}\n", 
            self.colour_scheme.message.to_string().bold().color(self.colour_scheme.message_colour),
            self.exception_name.bold()
        ).as_str());
        message.push_str(format!(
            "{}{} {}:{}:{}\n",
            " ".repeat(smallest_line_number_size),
            " -->".color(self.colour_scheme.arrow_to_message),
            self.file_data.file_name,
            self.start_position.line + 1,
            self.start_position.column + 1,
        ).as_str());

        // buffer
        message.push_str(format!("{}\n", empty_line_number_display).as_str());

        for line_number in self.start_position.line..=self.end_position.line {
            let current_line : String = lines[line_number].to_string();
            let line_number_display = format!(
                "{}{} {}",
                " ".repeat(((biggest_line_number_size as i32 - (line_number + 1).to_string().len() as i32).max(0)).try_into().unwrap()),
                (line_number + 1).to_string().color(self.colour_scheme.line_number),
                "|".color(self.colour_scheme.separator)
            );

            message.push_str(format!("{} {}\n", line_number_display, current_line).as_str());

            message.push_str(
                format!("{} {}\n", empty_line_number_display,
                if line_number == self.start_position.line {
                    format!(
                        "{}{}",
                        " ".repeat(
                            (self.start_position.index as i32  - self.file_data.data.start_of_line(line_number).unwrap() as i32).max(0) as usize
                        ),
                        "^".repeat(self.file_data.data.start_of_line_or(line_number + 1).min(self.end_position.index + 1) - self.start_position.index).color(self.colour_scheme.arrow_to_error)
                    )
                } else if line_number == self.end_position.line {
                    format!(
                        "{}",
                        "^".repeat(self.end_position.index - self.file_data.data.start_of_line(line_number).unwrap() + 1).color(self.colour_scheme.arrow_to_error)
                    )
                } else {
                    "^".repeat(current_line.len()).color(self.colour_scheme.arrow_to_error).to_string()
                }
            ).as_str())
        }
        let mut note = String::new();
        if self.note.contains('\n') {
            let mut first = false;
            for line in self.note.split('\n') {
                if !first {
                    first = true;
                    note.push_str(line.trim());
                } else {
                    note.push_str(format!("\n{}         {}", " ".repeat(smallest_line_number_size), line.trim()).as_str())
                }
            }
        } else {
            note = self.note.clone()
        }
        // Add the note
        if !self.note.is_empty() {
            message.push_str(format!(
                "{}{} {} {}\n",
                " ".repeat(smallest_line_number_size),
                " =".color(self.colour_scheme.equal),
                "note:".bold().color(self.colour_scheme.note_colour),
                note).as_str()
            );
        }

        BasicException(message).run()
    }
}