use core::panic;
use std::mem::{discriminant, Discriminant};

use crate::{lexer::tokens::{Token, TokenKind, self}, exceptions::{parser_errors::{UnterminatedParenthesis, InvalidSyntaxExpected, FailedToFindEndOfLine, CriticalError, InvalidSyntax, NewLineError}, Exception}, utils::{Position, FileData}};

use self::instructions::{Instruction, BinaryOperator, InstructionKind, UnaryOperator, Literal};

pub mod instructions;
pub mod context;
pub mod functions;

pub struct Parser<'a> {
    tokens: Vec<Token>,
    index: usize,
    file_data: &'a FileData,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, file_data: &'a FileData) -> Self { Self { tokens, index: 0, file_data } }
    pub fn parse(&mut self) -> Vec<Instruction>{
        self.parse_section()
    }

    fn parse_section(&mut self) -> Vec<Instruction> {
        if self.current_token_nc().kind == TokenKind::Indent {
            self.advance()
        }
        let mut instructions = Vec::new();
        while let Some(token) = self.current_token() {
            if token.kind == TokenKind::Dedent || token.kind == TokenKind::EndOfFile { break; }
            instructions.push(match &self.current_token_nc().kind {
                TokenKind::Let => self.variable_assign(),
                TokenKind::EndOfFile => break,
                TokenKind::NewLine => { self.advance(); continue; },
                TokenKind::If => self.if_statement(),
                TokenKind::Multiply => {
                    let start = self.current_token_nc().start_position.clone();
                    self.advance();
                    let expr = self.expr();
                    Instruction::new(start, expr.end_position.clone(), InstructionKind::SystemOut { value: Box::new(expr) })
                },
                TokenKind::While => self.while_statement(),
                TokenKind::Identifier(v) => self.found_identifier(),
                TokenKind::Function => self.declare_function(),
                TokenKind::Integer(_) | TokenKind::Float(_) => self.expr(),
                TokenKind::Use => self.use_statement(),
                TokenKind::Return => self.return_statement(),
                _ => InvalidSyntax::new(self.current_token_nc().start_position.clone(), self.current_token_nc().end_position.clone(), self.file_data, "unexpected token found".to_string()).run()
            });
        }
        self.advance();
        instructions
    }
    #[inline(always)]
    fn advance(&mut self) {
        self.index += 1;
    }

    #[inline(always)]
    fn retreat(&mut self) {
        self.index -= 1;
    }

    #[inline(always)]
    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    #[inline(always)]
    fn current_token_nc(&self) -> &Token {
        &self.tokens[self.index]
    }
    
    #[inline(always)]
    fn peak(&self) -> Option<&Token> {
        self.peak_by(1)
    }

    #[inline(always)]
    fn peak_by(&self, by: usize) -> Option<&Token> {
        self.tokens.get(self.index + by)
    }

    fn position_at_new_line(&self) -> Position {
        let mut index = 0;
        while let Some(token) = self.peak_by(index) {
            if token.kind == TokenKind::NewLine {
                return token.start_position.clone()
            }
            index += 1;
        }
        return match self.peak_by(index - 2) {
            Some(v) => v,
            None => CriticalError::new().run()
        }.end_position.clone()
    }

    fn expect(&self, token_kind: TokenKind) -> bool {
        if let Some(x) = self.current_token() {
            discriminant(&x.kind) != discriminant(&token_kind)
        } else {
            true
        }
    }
    fn expect_multiple(&mut self, token_kind: &[TokenKind]) -> bool {
        let return_value = if let Some(x) = self.current_token() {
            !token_kind.iter().map(|x| discriminant(x)).collect::<Vec<Discriminant<TokenKind>>>().contains(&discriminant(&x.kind))
        } else {
            true
        };
        return_value
    }
}


// Binary Operation
impl<'a> Parser<'a> {
    fn term(&mut self) -> Instruction {
        self.binary_operation(Parser::factor, Parser::factor, &[TokenKind::Multiply, TokenKind::Slash])
    }
    
    fn expr(&mut self) -> Instruction {
        self.binary_operation(Parser::comparator_expr, Parser::comparator_expr, &[TokenKind::And, TokenKind::Or])
    }

    fn comparator_expr(&mut self) -> Instruction {
        if let Some(token) = self.current_token() {
            if token.kind == TokenKind::ExclamationMark {
                let start = self.current_token_nc().start_position.clone();
                self.advance();
                return Instruction::new(start, self.current_token_nc().end_position.clone(), InstructionKind::UnaryOperation { operator: UnaryOperator::Not, value: Box::new(self.comparator_expr()) })
            }
            return self.binary_operation(Parser::arithmetic_expr, Parser::arithmetic_expr, &[TokenKind::EqualsTo, TokenKind::NotEquals, TokenKind::GreaterEquals, TokenKind::SmallerEquals, TokenKind::LeftAngle, TokenKind::RightAngle])
        }
        CriticalError::new().run()
    }

    fn arithmetic_expr(&mut self) -> Instruction {
        self.binary_operation(Parser::term, Parser::term, &[TokenKind::Plus, TokenKind::Minus])
    }

    fn factor(&mut self) -> Instruction {
        if [TokenKind::Plus, TokenKind::Minus].contains(&self.current_token_nc().kind) {
            let start_pos = self.current_token_nc().start_position.clone();
            let operator = UnaryOperator::from_token(self.current_token_nc(), self.file_data);
            self.advance();
            let factor = self.factor();
            return Instruction::new(
                start_pos,
                factor.end_position.clone(),
                InstructionKind::UnaryOperation { 
                    operator, 
                    value: Box::new(factor) 
                }
            )
        }
        self.power()
    }

    fn power(&mut self) -> Instruction {
        self.binary_operation(Parser::atom, Parser::factor, &[TokenKind::Power])
    }

    fn atom(&mut self) -> Instruction {
        match self.current_token_nc().kind.clone() {
            TokenKind::LeftParenthesis => {
                let start = self.current_token_nc().start_position.clone();
                self.advance();
                let expr = self.expr();
                if self.current_token_nc().kind != TokenKind::RightParenthesis {
                    UnterminatedParenthesis::new(start, self.current_token_nc().end_position.clone(), self.file_data).run()
                }
                Instruction::new(
                    start,
                    self.current_token_nc().end_position.clone(),
                    expr.kind
                )
            }
            TokenKind::Identifier(v) => self.found_identifier(),
            TokenKind::Return => self.return_statement(),
            TokenKind::Null => {
                let instr = Instruction::new(
                    self.current_token_nc().start_position.clone(),
                    self.current_token_nc().end_position.clone(),
                    InstructionKind::Lit(Literal::Null)
                );
                self.advance();
                instr
            },
            _ => {
                let literal = Literal::from_token(self.current_token_nc(), self.file_data);
                let instruction = Instruction::new(
                    self.current_token_nc().start_position.clone(),
                    self.current_token_nc().end_position.clone(),
                    InstructionKind::Lit(literal)
                );
                self.advance();
                instruction
            }
        }
    }

    fn binary_operation(
        &mut self,
        left_func: fn(&mut Parser<'a>) -> Instruction,
        right_func: fn(&mut Parser<'a>) -> Instruction,
        operation_tokens: &[TokenKind],
    ) -> Instruction {
        let mut left = left_func(self);
        while let Some(token) = self.current_token() {
            if !operation_tokens.contains(&token.kind) {
                break;
            }
            let operation = BinaryOperator::from_token(token, self.file_data);
            self.advance();
            let right = right_func(self);
            left = Instruction::new(
                left.start_position.clone(),
                right.end_position.clone(),
                InstructionKind::BinaryOperation { left: Box::new(left), right: Box::new(right), operation }
            )
        }
        left
    }
}

impl<'a> Parser<'a> {
    fn variable_assign(&mut self) -> Instruction {
        let start = self.current_token_nc().start_position.clone();
        if self.expect(TokenKind::Let) { InvalidSyntaxExpected::new(start, self.current_token_nc().end_position.clone(), self.file_data, "let [identifier] = [expression]".to_string()).run() }
        self.advance();
        if self.expect(TokenKind::Identifier(String::new())) { self.retreat(); InvalidSyntaxExpected::new(start, self.current_token_nc().end_position.clone(), self.file_data, "let [identifier] = [expression]".to_string()).run() }
        // The Non Checked here can be dismissed since any none value will be catched above
        let identifier = match &self.current_token_nc().kind {
            TokenKind::Identifier(v) => v,
            _ => InvalidSyntaxExpected::new(start, self.current_token_nc().end_position.clone(), self.file_data, "an identifier".to_string()).run(),
        }.clone();
        self.advance();
        if self.expect(TokenKind::Equals) {
            if self.expect(TokenKind::NewLine) {
                InvalidSyntaxExpected::new(start, self.current_token_nc().end_position.clone(), self.file_data, format!("let {} = [expression]", identifier)).run() 
            }    
            return Instruction::new(start.clone(), self.current_token_nc().end_position.clone(), InstructionKind::VarAssign { identifier, value: Box::new(Instruction::new(start, self.current_token_nc().end_position.clone(), InstructionKind::Lit(Literal::Null))) })
        }
        self.advance();
        let expr = self.expr();
        let end = self.current_token_nc().end_position.clone();
        if self.expect_multiple(&[TokenKind::NewLine, TokenKind::EndOfFile]) { NewLineError::new(start, end, self.file_data).run() }
        self.advance();
        Instruction::new(start, end, InstructionKind::VarAssign { identifier, value: Box::new(expr) })
    }

    fn declare_function(&mut self) -> Instruction {
        let start = self.current_token_nc().start_position.clone();
        if self.expect(TokenKind::Function) { InvalidSyntaxExpected::new(start, self.current_token_nc().end_position.clone(), self.file_data, "fn [identifier]() { ... }".to_string()).run() }
        self.advance();
        if self.expect(TokenKind::Identifier(String::new())) { InvalidSyntaxExpected::new(start, self.current_token_nc().end_position.clone(), self.file_data, "fn [identifier]() { ... }".to_string()).run() }
        let identifier = match &self.current_token_nc().kind {
            TokenKind::Identifier(v) => v,
            _ => InvalidSyntaxExpected::new(start, self.current_token_nc().end_position.clone(), self.file_data, "an identifier".to_string()).run(),
        }.clone();
        self.advance();
        if self.expect(TokenKind::LeftParenthesis) { InvalidSyntaxExpected::new(start, self.current_token_nc().end_position.clone(), self.file_data, format!("fn {}() {{ ... }}", identifier)).run() }
        self.advance();
        let mut args = Vec::new();
        while self.current_token().is_some() {
            if self.current_token_nc().kind == TokenKind::RightParenthesis {
                break;
            }
            if !args.is_empty() {
                if self.expect(TokenKind::Comma) { InvalidSyntax::new(start, self.current_token_nc().end_position.clone(), self.file_data, "commas are used to separate between function arguments".to_string()).run() }
                self.advance()
            }
            match &self.current_token_nc().kind.clone() {
                TokenKind::Identifier(v) => args.push(v.clone()),
                _ => break,
            }
            self.advance();
        }
        if self.current_token_nc().kind != TokenKind::RightParenthesis {
            dbg!(&self.current_token_nc().kind);
            UnterminatedParenthesis::new(start, self.current_token_nc().end_position.clone(), self.file_data).run()
        }
        self.advance();
        let body = self.parse_section();
        Instruction::new(
            start,
            self.current_token_nc().end_position.clone(),
            InstructionKind::DeclareFunction { 
                identifier: identifier.clone(), 
                args, 
                body
            }
        )
    }

    fn found_identifier(&mut self) -> Instruction {
        let start = self.current_token_nc().start_position.clone();
        if self.expect(TokenKind::Identifier(String::new())) { InvalidSyntax::new(start, self.current_token_nc().end_position.clone(), self.file_data, "check the docs for valid usages".to_string()).run() }
        let identifier = match &self.current_token_nc().kind {
            TokenKind::Identifier(v) => v,
            _ => InvalidSyntaxExpected::new(start, self.current_token_nc().end_position.clone(), self.file_data, "an identifier".to_string()).run(),
        }.clone();
        self.advance();
        match self.current_token_nc().kind {
            TokenKind::LeftParenthesis => self.function_call(&start, &identifier),
            TokenKind::Equals => self.update_variable(&start, &identifier),
            _ => Instruction::new(start, self.current_token_nc().end_position.clone(), InstructionKind::VarAccess { identifier: identifier.clone() }),
        }
    }

    fn function_call(&mut self, start: &Position, identifier: &String) -> Instruction {
        if self.expect(TokenKind::LeftParenthesis) { InvalidSyntaxExpected::new(start.clone(), self.current_token_nc().end_position.clone(), self.file_data, format!("{}(args..)", identifier)).run() }
        self.advance();
        let mut args = Vec::new();
        while self.current_token().is_some() && self.current_token_nc().kind != TokenKind::RightParenthesis {
            let token_kind = self.current_token_nc().kind.clone();
            if token_kind == TokenKind::NewLine {
                self.advance();
                continue;
            } else if !args.is_empty() {
                if self.expect(TokenKind::Comma) { println!("{}", self.current_token_nc().kind.clone()); InvalidSyntax::new(start.clone(), self.current_token_nc().end_position.clone(), self.file_data, "commas are used to separate between function arguments".to_string()).run() }
                self.advance()
            }
            args.push(self.expr());
        }
        if self.current_token_nc().kind != TokenKind::RightParenthesis {
            dbg!(&self.current_token_nc().kind);
            UnterminatedParenthesis::new(start.clone(), self.current_token_nc().end_position.clone(), self.file_data).run()
        }
        let end = self.current_token_nc().end_position.clone();
        self.advance();
        Instruction::new(start.clone(), end, InstructionKind::CallFunction { identifier: identifier.clone(), args })
    }

    fn update_variable(&mut self, start: &Position, identifier: &String) -> Instruction {
        let expr_type = match &match self.current_token() {
            Some(s) => s,
            None => InvalidSyntaxExpected::new(start.clone(), self.current_token_nc().end_position.clone(), self.file_data, format!("{} = [expression]", identifier)).run(),
        }.kind {
            | TokenKind::Equals
            | TokenKind::PlusEquals
            | TokenKind::MinusEquals
            | TokenKind::DivideEquals
            | TokenKind::MultiplyEquals
            | TokenKind::PowerEquals => self.current_token_nc().kind.clone(),
            _ => InvalidSyntaxExpected::new(start.clone(), self.current_token_nc().end_position.clone(), self.file_data, format!("{} = [expression]", identifier)).run(),
        };
        self.advance();
        let expr = self.expr();
        self.retreat();
        let end = self.current_token_nc().end_position.clone();
        self.advance();
        Instruction::new(
            start.clone(), 
            end.clone(), 
            InstructionKind::VarUpdate { 
                identifier: identifier.clone(), 
                value: Box::new(
                    Instruction::new(
                        start.clone(),
                        end.clone(),
                        InstructionKind::BinaryOperation { 
                            left: Box::new(Instruction::new(start.clone(), end.clone(), InstructionKind::VarAccess { identifier: identifier.clone() })),
                            right: Box::new(expr),
                            operation: BinaryOperator::from_token_kind(expr_type, (start.clone(), end.clone()), self.file_data)
                        }
                    )
                )
            }
        )
    }

    fn if_statement(&mut self) -> Instruction {
        self.if_statement_w_start(self.current_token_nc().start_position.clone())
    }

    fn if_statement_w_start(&mut self, start: Position) -> Instruction {
        let start = self.current_token_nc().start_position.clone();
        if self.expect(TokenKind::If) { InvalidSyntaxExpected::new(start, self.current_token_nc().end_position.clone(), self.file_data, "if [expression] { ... }".to_string()).run() }
        self.advance();
        let expr = self.expr();
        if self.expect(TokenKind::Indent) { InvalidSyntaxExpected::new(start, self.current_token_nc().end_position.clone(), self.file_data, format!("if {} {{...}}", "[expression]")).run() }
        self.advance();
        let instructions = self.parse_section();
        let mut if_else : Option<Box<Instruction>> = None;
        while self.current_token().is_some() {
            let start = self.current_token_nc().start_position.clone();
            if self.current_token_nc().kind != TokenKind::Else {
                break;
            }
            if_else = Some(Box::new(
                if let Some(token) = self.peak() {
                    if token.kind != TokenKind::If {
                        self.else_statement()
                    } else {
                        self.if_statement_w_start(start)
                    }
                } else {
                    self.else_statement()
                }
            ))
        }
        Instruction::new(
            start,
            self.current_token_nc().end_position.clone(), 
            InstructionKind::If { 
                condition: Some(Box::new(expr)), 
                body: instructions,
                if_else,
            }
        )
    }

    fn else_statement(&mut self) -> Instruction {
        let start = self.current_token_nc().start_position.clone();
        if self.expect(TokenKind::Else) { InvalidSyntaxExpected::new(start, self.current_token_nc().end_position.clone(), self.file_data, "else { ... }".to_string()).run() }
        self.advance();
        if self.expect(TokenKind::Indent) { InvalidSyntaxExpected::new(start, self.current_token_nc().end_position.clone(), self.file_data, format!("if {} {{...}}", "[expression]")).run() }
        self.advance();
        let instructions = self.parse_section();
        Instruction::new(
            start,
            self.current_token_nc().end_position.clone(),
            InstructionKind::If { 
                condition: None, 
                body: instructions, 
                if_else: None
            }
        )
    }

    fn while_statement(&mut self) -> Instruction {
        let start = self.current_token_nc().start_position.clone();
        if self.expect(TokenKind::While) { InvalidSyntaxExpected::new(start, self.current_token_nc().end_position.clone(), self.file_data, "while [expression] { ... }".to_string()).run() }
        self.advance();
        let expr = self.expr();
        self.advance();
        let inside = self.parse_section();
        Instruction::new(
            start,
            self.current_token_nc().start_position.clone(),
            InstructionKind::While { 
                condition: Box::new(expr),
                body: inside
            }
        )
    }

    fn use_statement(&mut self) -> Instruction {
        let start = self.current_token_nc().start_position.clone();
        if self.expect(TokenKind::Use) { InvalidSyntaxExpected::new(start, self.current_token_nc().end_position.clone(), self.file_data, "use <[filepath]>".to_string()).run() }
        self.advance();
        if self.expect(TokenKind::LeftAngle) { InvalidSyntaxExpected::new(start, self.current_token_nc().end_position.clone(), self.file_data, "use <[filepath]>".to_string()).run() }
        self.advance();
        let mut file_path = String::new();
        while self.current_token().is_some() {
            if self.current_token_nc().kind == TokenKind::RightAngle {
                break;
            }
            file_path.push_str(self.current_token_nc().kind.to_string().as_str());
            self.advance();
        }
        if self.expect(TokenKind::RightAngle) { InvalidSyntaxExpected::new(start, self.current_token_nc().end_position.clone(), self.file_data, format!("use <{}>", file_path)).run() }
        self.advance();
        Instruction::new(
            start,
            self.current_token_nc().start_position.clone(),
            InstructionKind::Use {
                file_path,
            }
        )
    }

    fn return_statement(&mut self) -> Instruction {
        let start = self.current_token_nc().start_position.clone();
        if self.expect(TokenKind::Return) { InvalidSyntaxExpected::new(start, self.current_token_nc().end_position.clone(), self.file_data, "return [expression]".to_string()).run() }
        self.advance();
        let expr = self.expr();
        Instruction { 
            start_position: start, 
            end_position: self.current_token_nc().start_position.clone(), 
            kind: InstructionKind::Return { 
                value: Box::new(expr) 
            } 
        }
    }
}