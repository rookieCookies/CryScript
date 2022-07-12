use crate::{utils::{Position, FileData}, exceptions::{lexer_errors::{UnexpectedCharError, InvalidAmountOfDots, UnterminatedString, UnknownTokenError, NumberParseError}, Exception}};

use self::tokens::{Token, TokenKind};

pub mod tokens;

pub struct Lexer<'a> {
    characters: Vec<char>,
    position: Position,
    file_data: &'a FileData,
    reached_eof: bool,
    comment_type: Comment
}

impl<'a> Lexer<'a> {
    pub fn new(file_data: &'a FileData) -> Self {
        let characters = file_data.data.chars().collect();
        Self { 
            characters, 
            position: Position::new(),
            file_data,
            reached_eof: false,
            comment_type: Comment::None,
        }
    }

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(chr) = self.current_char() {
            match self.comment_type {
                Comment::None => {
                    let token = self.token();
                    if let Some(t) = token {
                        tokens.push(t);
                    }
                },
                Comment::SingleLine => if *chr == '\n' {
                    self.comment_type = Comment::None;
                },
                Comment::MultiLine => if *chr == '*' {
                    if let Some(&'/') = self.peak() {
                        self.comment_type = Comment::None;
                        self.advance()
                    }
                }
            }
            self.advance();
        }
        tokens.push(Token::new(self.position.clone(), self.position.clone(), TokenKind::EndOfFile));
        tokens
    }

    #[inline(always)]
    fn advance(&mut self) {
        if self.position.index >= self.characters.len() {
            self.reached_eof = true;
            return;
        }
        self.position.advance(self.characters[self.position.index]);
    }
    
    #[inline(always)]
    fn retreat(&mut self) {
        self.reached_eof = false;
        self.position.retreat(self.characters[self.position.index - 1]);
    }

    #[inline(always)]
    fn peak(&self) -> Option<&char> {
        self.peak_by(1)
    }

    #[inline(always)]
    fn peak_by(&self, by: usize) -> Option<&char> {
        self.characters.get(self.position.index + by)
    }

    #[inline(always)]
    fn current_char(&self) -> Option<&char> {
        if self.reached_eof {
            return None
        }
        self.characters.get(self.position.index)
    }

    // #[inline(always)]
    // fn current_char_nc(&self) -> char {
    //     self.characters[self.position.index]
    // }

    #[inline(always)]
    fn if_next_character_is(&mut self, next_char: char, if_true: TokenKind, if_false: TokenKind) -> TokenKind {
        if let Some(s) = self.peak() {
            return if s == &next_char { self.advance(); if_true } else { if_false }
        } else {
            if_false
        }
    }
}

impl<'a> Lexer<'a> {
    fn token(&mut self) -> Option<Token> {
        let start = self.position.clone();
        let current_char = match self.current_char() {
            Some(v) => v,
            None => return None,
        };

        let token_kind = match current_char {
            // Comments
            '/' => {
                match self.peak() {
                    Some(chr) => match chr {
                        '/' => { self.comment_type = Comment::SingleLine; return None },
                        '*' => { self.comment_type = Comment::MultiLine; return None },
                        _ => {}
                    },
                    None => {},
                }
                self.if_next_character_is('=', TokenKind::DivideEquals, TokenKind::Slash)
            },

            // Multi character tokens
            '"' => self.generate_string(),
            '0'..='9' => self.generate_number(),
            'a'..='z' | 'A'..='Z' => self.generate_keyword(),

            // Double character tokens
            '=' => self.if_next_character_is('=', TokenKind::EqualsTo, TokenKind::Equals),
            '!' => self.if_next_character_is('=', TokenKind::NotEquals, TokenKind::ExclamationMark),
            '&' => self.if_next_character_is('&', TokenKind::And, TokenKind::Ampersand),
            '|' => self.if_next_character_is('|', TokenKind::Or, TokenKind::Bar),
            ':' => self.if_next_character_is(':', TokenKind::DoubleColon, TokenKind::Colon),
            '<' => self.if_next_character_is('=', TokenKind::SmallerEquals, TokenKind::LeftAngle),
            '>' => self.if_next_character_is('=', TokenKind::GreaterEquals, TokenKind::RightAngle),

            '+' => self.if_next_character_is('=', TokenKind::PlusEquals, TokenKind::Plus),
            '-' => self.if_next_character_is('=', TokenKind::MinusEquals, TokenKind::Minus),
            '*' => self.if_next_character_is('=', TokenKind::MultiplyEquals, TokenKind::Multiply),
            '^' => self.if_next_character_is('=', TokenKind::PowerEquals, TokenKind::Power),

            // Single character tokensr,
            '\\' => TokenKind::BackSlash,
            '.' => TokenKind::Dot,
            ',' => TokenKind::Comma,
            '_' => TokenKind::Underscore,
            ';' => TokenKind::SemiColon,
            '[' => TokenKind::LeftSquare,
            ']' => TokenKind::RightSquare,
            '{' => TokenKind::Indent,
            '}' => TokenKind::Dedent,
            '(' => TokenKind::LeftParenthesis,
            ')' => TokenKind::RightParenthesis,
            '\n' => TokenKind::NewLine,
            
            // Ignored tokens
            '\r' | ' ' | '\t' => return None,
            _ => UnknownTokenError::new(start, self.position.clone(), self.file_data).run(),
        };

        Some(Token::new(
            start,
            self.position.clone(),
            token_kind
        ))

    }

    fn generate_string(&mut self) -> TokenKind {
        let current_char = match self.current_char() {
            Some(v) => v,
            None => &' ', // This will get flagged by the next check
        };
        if current_char != &'"' {
            UnexpectedCharError::new(self.position.clone(), self.position.clone(), self.file_data, '"', *current_char).run()
        }
        let start = self.position.clone();
        self.advance(); // Skip the first quotation mark
        let mut string = String::new();
        while let Some(chr) = self.current_char() {
            if chr == &'\\' {
                string.push(
                    match self.peak().unwrap() {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '\\' => '\\',
                        '0' => '\0',
                        '"' => '\"',
                        _ => {
                            self.advance();
                            continue;
                        }
                    }
                );
                self.advance();
                self.advance();
                continue;
            } else if chr == &'"' {
                break;
            }
            string.push(*chr);
            self.advance();
        }
        if self.current_char().is_none() {
            self.retreat();
            UnterminatedString::new(start, self.position.clone(), self.file_data).run()
        }
        TokenKind::String(string)
    }

    fn generate_number(&mut self) -> TokenKind {
        let start = self.position.clone();
        let mut number_string = String::new();
        let mut dot_count : u8 = 0;
        while let Some(chr) = self.current_char() {
            if !"0123456789._".contains(*chr) {
                break;
            }
            if chr == &'_' {
                continue;
            } else if chr == &'.' {
                dot_count += 1;
            }
            number_string.push(*chr);
            self.advance()
        }
        self.retreat();
        if dot_count == 0 {
            TokenKind::Integer(match number_string.parse() {
                Ok(v) => v,
                Err(_) => NumberParseError::new(start, self.position.clone(), self.file_data, number_string).run(),
            })
        } else if dot_count == 1 {
            TokenKind::Float(match number_string.parse() {
                Ok(v) => v,
                Err(_) => NumberParseError::new(start, self.position.clone(), self.file_data, number_string).run(),
            })
        } else {
            InvalidAmountOfDots::new(start, self.position.clone(), self.file_data).run()
        }
    }

    fn generate_keyword(&mut self) -> TokenKind {
        let mut string = String::new();
        while let Some(chr) = self.current_char() {
            if !chr.is_ascii_alphabetic() {
                break;
            }
            string.push(*chr);
            self.advance()
        }
        self.retreat();
        match string.as_str() {
            "let" => TokenKind::Let,
            "fn" => TokenKind::Function,
            "struct" => TokenKind::Struct,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "use" => TokenKind::Use,
            "return" => TokenKind::Return,
            "true" => TokenKind::Integer(1),
            "false" => TokenKind::Integer(0),
            "null" => TokenKind::Null,
            "while" => TokenKind::While,
            _ => TokenKind::Identifier(string)
        }
    }
}

enum Comment {
    None,
    SingleLine,
    MultiLine,
}