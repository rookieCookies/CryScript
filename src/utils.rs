#![allow(unused)]
use std::{fs::File, io::Read};

use rand::{Rng, distributions::Alphanumeric};

use crate::exceptions::{BasicException, Exception};

#[derive(Debug, Clone)]
pub struct Position {
    pub index: usize,
    pub column: usize,
    pub line: usize,
    column_vec: Vec<usize>
}

impl Position {
    pub fn new() -> Self { Self::new_w(0, 0, 0) }
    pub fn new_w(index: usize, column: usize, line: usize) -> Self { 
        Self { 
            index, 
            column, 
            line, 
            column_vec: Vec::new()
        } 
    }
    pub fn construct(data: &String) -> Self {
        let mut pos = Position::new();
        data.chars().into_iter().for_each(|x| pos.advance(x));
        pos
    }

    pub fn advance(&mut self, current_character: char) {
        self.index += 1;
        self.column += 1;
        if current_character == '\n' {
            self.column_vec.push(self.column);
            self.column = 0;
            self.line += 1;
        }
    }
    pub fn retreat(&mut self, current_character: char) {
        self.index -= 1;
        self.column -= 1;
        if current_character == '\n' {
            self.column = *self.column_vec.last().unwrap();
            self.line -= 1;
        }
    }
    pub fn trim(&mut self) {
        if self.column == 0 {
            self.line -= 1;
            self.column = *self.column_vec.last().unwrap();
        }
    }
}

#[derive(Debug)]
pub struct FileData {
    pub data: String,
    pub file_name: String,
}

pub fn read_file(file_path: &str) -> FileData { 
    let mut str = String::new();

    match match File::open(file_path) {
            Ok(file) => file,
            Err(_) => BasicException(format!("Failed to open file {}", file_path)).run(),
        }.read_to_string(&mut str) {
        Ok(_) => {},
        Err(_) => panic!("Failed to read file {}", file_path),
    };
    FileData { data: str, file_name: file_path.to_string() }
}

pub trait LineOfIndex {
    fn line_of_index(&self, index: usize) -> Option<String>;
    fn start_of_line(&self, index: usize) -> Option<usize>;
    fn start_of_line_or(&self, index: usize) -> usize;
}

impl LineOfIndex for String {
    fn line_of_index(&self, index: usize) -> Option<String> {
        let mut current_index = 0;
        for line in self.lines() {
            current_index += line.len() - 1;
            if current_index > index {
                return Some(line.to_string());
            }
        }
        None
    }

    fn start_of_line(&self, index: usize) -> Option<usize> {
        let mut current_index = 0;
        let mut current_line = 0;
        for line in self.lines() {
            if current_line == index {
                return Some(current_index)
            }

            current_index += line.len() as i32 as usize;
            current_index += 1; // new line 
            current_line += 1;
        }
        None
    }

    fn start_of_line_or(&self, index: usize) -> usize {
        let mut current_index = 0;
        let mut current_line = 0;
        for line in self.lines() {
            if current_line == index {
                return current_index
            }

            current_index += line.len() as i32 as usize;
            current_index += if cfg!(target="windows") { 2 } else { 1 }; // new line 
            current_line += 1;
        }
        current_index
    }
}
