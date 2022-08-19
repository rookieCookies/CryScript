use std::{rc::Rc, cell::RefCell};

pub trait CharUtils {
    fn is_alphabetic(&self) -> bool;
    fn is_number(&self) -> bool;
}

impl CharUtils for char {
    #[inline(always)]
    fn is_number(&self) -> bool {
        matches!(self, '0'..='9')
    }

    #[inline(always)]
    fn is_alphabetic(&self) -> bool {
        matches!(self, 'a'..='z' | 'A'..='Z')
    }
}

pub trait StringUtils {
    fn line_at(&self, idx: usize) -> Option<usize>;
    fn start_of_line(&self, index: usize) -> Option<usize>;
    fn start_of_line_or(&self, index: usize) -> usize;
}

impl StringUtils for String {
    fn line_at(&self, idx: usize) -> Option<usize> {
        let mut current_index = 0;
        for (line_number, line) in self.lines().enumerate() {
            current_index += line.len() + 1;
            // println!("{} {}", current_index, idx);
            if current_index > idx {
                return Some(line_number);
            }
        }
        if current_index == idx {
            return Some(0)
        }
        None
    }

    fn start_of_line(&self, index: usize) -> Option<usize> {
        let mut current_index = 0;
        for (current_line, line) in self.lines().enumerate() {
            if current_line == index {
                return Some(current_index);
            }

            current_index += line.len() as i32 as usize;
            current_index += 1; // new line
        }
        None
    }

    fn start_of_line_or(&self, index: usize) -> usize {
        let mut current_index = 0;
        for (current_line, line) in self.lines().enumerate() {
            if current_line == index {
                return current_index;
            }

            current_index += line.len() as i32 as usize;
            current_index += if cfg!(target = "windows") { 2 } else { 1 }; // new line
        }
        current_index
    }
}

#[test]
fn char_utils_is_number() {
    assert!('0'.is_number());
    assert!('1'.is_number());
    assert!('2'.is_number());
    assert!('3'.is_number());
    assert!('4'.is_number());
    assert!('5'.is_number());
    assert!('6'.is_number());
    assert!('7'.is_number());
    assert!('8'.is_number());
    assert!('9'.is_number());
    assert!(!'@'.is_number());
    assert!(!'a'.is_number());
    assert!(!'&'.is_number());
    assert!(!'g'.is_number());
    assert!(!'æ'.is_number());
    assert!(!'…'.is_number());
}


#[inline(always)]
pub fn wrap<T>(value: T) -> Rc<RefCell<T>> {
    Rc::new(RefCell::new(value))
}