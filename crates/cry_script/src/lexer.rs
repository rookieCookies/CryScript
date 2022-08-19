pub(crate) mod token;

use std::rc::Rc;

use utils::CharUtils;

use crate::{
    exceptions::lexer_exceptions::{
        InvalidAmountOfDots, UnknownToken, UnmatchedDedentToken, UnterminatedIndentation,
        UnterminatedString,
    },
    Annotation, Keyword, Position,
};

use self::token::{Token, TokenType, TypeHintToken};

use super::FileData;

pub struct Lexer {
    file_data: Rc<FileData>,
    characters: Vec<char>,
    current_index: usize,
    indent_level: usize,
}

// #############################
// #
// # Lex
// #
// #############################
impl Lexer {
    pub(crate) fn lex(file_data: Rc<FileData>) -> Vec<Token> {
        let characters: Vec<char> = file_data.data.chars().collect();

        let mut lexer = Self {
            file_data: file_data.clone(),
            characters,
            current_index: 0,
            indent_level: 0,
        };
        let mut comment_type = Comment::None;
        let mut tokens = Vec::with_capacity(lexer.characters.len());

        while let Some(chr) = lexer.current_char() {
            match comment_type {
                Comment::None => {
                    let token = lexer.token(&mut comment_type);
                    if let Some(token) = token {
                        tokens.push(token)
                    }
                }
                Comment::SingleLine => {
                    if chr == &'\n' {
                        comment_type = Comment::None;
                    }
                }
                Comment::MultiLine => {
                    if chr == &'*' {
                        if let Some(&'/') = lexer.peak() {
                            comment_type = Comment::None;
                            lexer.advance()
                        }
                    }
                }
            }
            lexer.advance()
        }
        let end = match tokens.last() {
            Some(t) => t.end.clone(),
            None => Position::new(0),
        };
        tokens.push(Token::new(
            end.clone(),
            end,
            TokenType::EndOfFile,
            file_data.clone(),
        ));
        if lexer.indent_level != 0 {
            let pos = Position::new(lexer.current_index + 1);
            UnterminatedIndentation::call(&pos, &pos, &file_data)
        }
        tokens
    }

    fn token(&mut self, comment_handle: &mut Comment) -> Option<Token> {
        let start = self.current_index;
        let current_char = self.current_char()?;
        let token_kind = match current_char {
            '/' => match self.peak() {
                Some(chr) => match chr {
                    '/' => {
                        self.advance();
                        *comment_handle = Comment::SingleLine;
                        TokenType::NewLine
                    }
                    '*' => {
                        self.advance();
                        *comment_handle = Comment::MultiLine;
                        TokenType::NewLine
                    }
                    '!' => {
                        self.advance();
                        self.advance();
                        let mut string = String::new();
                        let mut final_string = String::new();
                        while let Some(chr) = self.current_char() {
                            string.push(match chr {
                                '!' => {
                                    if let Some('/') = self.peak() {
                                        break;
                                    } else {
                                        *chr
                                    }
                                }
                                '\n' => {
                                    final_string.push_str(string.trim());
                                    final_string.push('\n');
                                    string.clear();
                                    self.advance();
                                    continue;
                                }
                                _ => *chr,
                            });
                            self.advance();
                        }
                        self.advance();
                        TokenType::Annotation(Annotation::DocComment(final_string))
                    }
                    _ => self.if_next_character_is('=', TokenType::DivideEquals, TokenType::Slash),
                },
                None => self.if_next_character_is('=', TokenType::DivideEquals, TokenType::Slash),
            },

            // -------------------------
            //
            // Multi Character tokens
            //
            // -------------------------
            '"' => self.generate_string(),
            '0'..='9' => self.generate_number(),
            'a'..='z' | 'A'..='Z' => self.generate_keyword(),

            // -------------------------
            //
            // Double Character tokens
            //
            // -------------------------
            '=' => self.if_next_character_is('=', TokenType::EqualsTo, TokenType::Equals),
            '!' => self.if_next_character_is('=', TokenType::NotEquals, TokenType::ExclamationMark),
            '&' => self.if_next_character_is('&', TokenType::And, TokenType::Ampersand),
            '|' => self.if_next_character_is('|', TokenType::Or, TokenType::Bar),
            ':' => self.if_next_character_is(':', TokenType::DoubleColon, TokenType::Colon),
            '<' => self.if_next_character_is('=', TokenType::SmallerEquals, TokenType::LeftAngle),
            '>' => self.if_next_character_is('=', TokenType::GreaterEquals, TokenType::RightAngle),
            '+' => self.if_next_character_is('=', TokenType::PlusEquals, TokenType::Plus),
            '-' => self.if_next_character_is('=', TokenType::MinusEquals, TokenType::Minus),
            '*' => self.if_next_character_is('=', TokenType::MultiplyEquals, TokenType::Multiply),
            '^' => self.if_next_character_is('=', TokenType::PowerEquals, TokenType::Power),

            // -------------------------
            //
            // Single Character tokens
            //
            // -------------------------
            '\\' => TokenType::Backslash,
            '.' => TokenType::Dot,
            ',' => TokenType::Comma,
            '_' => TokenType::Underscore,
            ';' => TokenType::SemiColon,
            '[' => TokenType::LeftSquare,
            ']' => TokenType::RightSquare,
            '{' => {
                self.indent_level += 1;
                TokenType::Indent
            }
            '}' => {
                if self.indent_level == 0 {
                    UnmatchedDedentToken::call(
                        &Position::new(start),
                        &Position::new((self.current_index as i32).max(0) as usize),
                        &self.file_data,
                    )
                } else {
                    self.indent_level -= 1;
                }
                TokenType::Dedent
            }
            '(' => TokenType::LeftParenthesis,
            ')' => TokenType::RightParenthesis,
            '@' => TokenType::Annotation({
                self.advance();
                Annotation::from(
                    self.generate_keyword().to_string(),
                    (
                        &Position::new(start),
                        &Position::new(self.current_index),
                        &self.file_data,
                    ),
                )
            }),
            '\n' => TokenType::NewLine,

            // -------------------------
            //
            // Ignored Character tokens
            //
            // -------------------------
            '\r' | ' ' | '\t' => return None,

            _ => {
                let pos = Position::new((self.current_index as i32).max(0) as usize);
                UnknownToken::call(&pos, &pos, &self.file_data)
            }
        };

        Some(Token::new(
            Position::new(start),
            Position::new(self.current_index),
            token_kind,
            self.file_data.clone(),
        ))
    }
}

// #############################
// #
// # Utility functions
// #
// #############################
impl Lexer {
    #[inline(always)]
    fn advance(&mut self) {
        self.current_index += 1;
    }

    #[inline(always)]
    fn retreat(&mut self) {
        self.current_index -= 1;
    }

    #[inline(always)]
    fn current_char(&self) -> Option<&char> {
        self.peak_by(0)
    }

    #[inline(always)]
    fn peak(&self) -> Option<&char> {
        self.peak_by(1)
    }

    #[inline(always)]
    fn peak_by(&self, by: usize) -> Option<&char> {
        let chr = self.characters.get(self.current_index + by);
        match chr {
            Some(v) => {
                if v == &'\0' {
                    None
                } else {
                    Some(v)
                }
            }
            None => None,
        }
    }

    #[inline(always)]
    fn if_next_character_is(
        &mut self,
        next_char: char,
        if_true: TokenType,
        if_false: TokenType,
    ) -> TokenType {
        if let Some(s) = self.peak() {
            if s == &next_char {
                self.advance();
                if_true
            } else {
                if_false
            }
        } else {
            if_false
        }
    }
}

// #############################
// #
// # Token generators
// #
// #############################
impl Lexer {
    fn generate_number(&mut self) -> TokenType {
        let start = self.current_index;
        let mut number_string = String::new();
        let mut dot_count = 0;
        while let Some(chr) = self.characters.get(self.current_index) {
            if chr == &'.' {
                dot_count += 1;
            } else if chr == &'_' {
                self.advance();
                continue;
            } else if !chr.is_number() {
                break;
            }
            number_string.push(*chr);
            self.advance();
        }
        self.retreat();
        match dot_count {
            0 => TokenType::Integer(number_string.parse().unwrap()),
            1 => TokenType::Float(number_string.parse().unwrap()),
            _ => InvalidAmountOfDots::call(
                &Position::new(start),
                &Position::new(self.current_index - 1),
                &self.file_data,
            ),
        }
    }

    fn generate_string(&mut self) -> TokenType {
        let start = self.current_index;
        let mut string = String::new();
        self.advance();
        while let Some(chr) = self.current_char() {
            let chr = *chr;
            if "\n\r\t\0".contains(chr) {
                self.advance();
                continue;
            } else if chr == '\\' && self.peak().is_some() {
                string.push(match self.peak().unwrap() {
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
                });
                self.advance();
                self.advance();
                continue;
            } else if chr == '"' {
                break;
            }
            string.push(chr);
            self.advance();
        }
        if self.current_char() != Some(&'"') {
            UnterminatedString::call(
                &Position::new(start),
                &Position::new(self.current_index),
                &self.file_data,
            )
        }
        TokenType::String(string)
    }

    fn generate_keyword(&mut self) -> TokenType {
        let mut string = String::new();
        while let Some(chr) = self.characters.get(self.current_index) {
            if !chr.is_alphabetic() && !chr.is_number() && chr != &'_' {
                break;
            }
            string.push(*chr);
            self.advance();
        }
        self.retreat();
        match string.as_str() {
            "var" => TokenType::Keyword(Keyword::Var),
            "if" => TokenType::Keyword(Keyword::If),
            "else" => TokenType::Keyword(Keyword::Else),
            "fn" => TokenType::Keyword(Keyword::Function),
            "use" => TokenType::Keyword(Keyword::Use),
            "return" => TokenType::Keyword(Keyword::Return),
            "break" => TokenType::Keyword(Keyword::Break),
            "while" => TokenType::Keyword(Keyword::While),
            "catch" => TokenType::Keyword(Keyword::Catch),
            "do" => TokenType::Keyword(Keyword::Do),
            "as" => TokenType::Keyword(Keyword::As),
            "final" => TokenType::Keyword(Keyword::Final),
            "class" => TokenType::Keyword(Keyword::Class),
            "new" => TokenType::Keyword(Keyword::New),

            "true" => TokenType::Bool(true),
            "false" => TokenType::Bool(false),
            "null" => TokenType::Null,

            "int" => TokenType::TypeHint(TypeHintToken::Integer),
            "integer" => TokenType::TypeHint(TypeHintToken::Integer),

            "float" => TokenType::TypeHint(TypeHintToken::Float),

            "str" => TokenType::TypeHint(TypeHintToken::String),
            "string" => TokenType::TypeHint(TypeHintToken::String),
            _ => TokenType::Identifier(string),
        }
    }
}

enum Comment {
    SingleLine,
    MultiLine,
    None,
}
