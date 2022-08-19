pub mod data;

use core::panic;
use std::{mem::discriminant, rc::Rc};

use crate::{
    exceptions::parser_exceptions::{
        InvalidInstructionInClass, UnexpectedToken, UnterminatedParenthesis,
    },
    interpreter::function::{Type, TypeHint},
    lexer::token::{Token, TokenType},
    Annotation, FileData, Keyword, Position,
};

use crate::interpreter::instructions::{
    binary_op::BinaryOperator, unary_op::UnaryOperator, Instruction, InstructionType,
};

use self::data::Data;

pub(crate) struct Parser {
    tokens: Vec<Token>,
    current_index: usize,
    file_data: Rc<FileData>,
}

// #############################
// #
// # Parse
// #
// #############################
impl Parser {
    pub(crate) fn parse(file_data: Rc<FileData>, tokens: Vec<Token>) -> Vec<Instruction> {
        if tokens.is_empty() {
            return vec![];
        }
        let mut parser = Parser {
            tokens,
            current_index: 0,
            file_data,
        };

        match parser.parse_section().instruction_type {
            InstructionType::Section { body } => body,
            _ => panic!(),
        }
    }

    pub(crate) fn parse_section(&mut self) -> Instruction {
        if self.current_token().unwrap().token_type == TokenType::Indent {
            self.advance()
        }
        let mut instructions = vec![];
        while self.current_token().is_some() {
            self.skip_new_line();
            if let Some(token) = self.current_token() {
                if token.token_type == TokenType::Dedent || token.token_type == TokenType::EndOfFile
                {
                    break;
                }
            }
            instructions.push(self.parse_once());
            self.skip_new_line();
            // println!("{}", self.current_token().unwrap().token_type);
        }
        // self.advance();
        Instruction::new(
            match instructions.first() {
                Some(v) => v.start.clone(),
                None => Position::new(0),
            },
            match instructions.last() {
                Some(v) => v.end.clone(),
                None => Position::new(0),
            },
            self.file_data.clone(),
            InstructionType::Section { body: instructions },
        )
    }

    fn parse_once(&mut self) -> Instruction {
        self.skip_new_line();
        // println!("{}", self.current_token().unwrap().token_type);
        let value = match &self.current_token().unwrap().token_type {
            TokenType::Identifier(_) => self.identifier_statement(),
            TokenType::Annotation(_) => self.annotation(),
            TokenType::Keyword(v) => match v {
                Keyword::Var | Keyword::Final => self.var_statement(),
                Keyword::Use => self.use_statement(),
                Keyword::Pass => self.pass_statement(),
                Keyword::Function => self.function_declaration(),
                _ => self.expr(),
            },
            _ => self.expr(),
        };
        self.advance();
        // println!("{}", self.current_token().unwrap().token_type);
        if self.current_token().is_some() {
            match &self.current_token().unwrap().token_type {
                TokenType::NewLine => self.advance(),
                _ => {
                    if self.current_token().unwrap().token_type != TokenType::Dedent
                        && self.current_token().unwrap().token_type != TokenType::EndOfFile
                    {
                        UnexpectedToken::call(
                            self.current_token().unwrap().fetch(),
                            "new line or dedent token",
                            self.current_token()
                                .unwrap()
                                .token_type
                                .to_string()
                                .as_str(),
                        )
                    }
                }
            }
        } else if self.peak_behind_by(1).is_none()
            || self.peak_behind_by(1).unwrap().token_type != TokenType::EndOfFile
        {
            UnexpectedToken::call(self.last_token().unwrap().fetch(), "end of file", "nothing")
        };
        value
    }
}

// #############################
// #
// # Utility
// #
// #############################
impl Parser {
    #[inline(always)]
    fn skip_new_line(&mut self) {
        while self.current_token().is_some()
            && self.current_token().unwrap().token_type == TokenType::NewLine
        {
            self.advance()
        }
    }

    #[inline(always)]
    fn advance(&mut self) {
        self.current_index += 1;
    }

    #[inline(always)]
    fn retreat(&mut self) {
        self.current_index -= 1;
    }

    #[inline(always)]
    fn current_token(&self) -> Option<&Token> {
        self.peak_by(0)
    }

    #[inline(always)]
    fn current_token_type(&self) -> &TokenType {
        &self.current_token().unwrap().token_type
    }

    #[inline(always)]
    fn current_token_type_str(&self) -> String {
        self.current_token().unwrap().token_type.to_string()
    }

    #[inline(always)]
    fn peak(&self) -> Option<&Token> {
        self.peak_by(1)
    }

    #[inline(always)]
    fn peak_by(&self, by: usize) -> Option<&Token> {
        self.tokens.get(self.current_index + by)
    }

    #[inline(always)]
    fn peak_behind_by(&self, by: usize) -> Option<&Token> {
        self.tokens.get(self.current_index - by)
    }

    #[inline(always)]
    fn last_token(&self) -> Option<&Token> {
        let mut x = 0;
        while self.peak_behind_by(x).is_none() {
            x += 1;
        }
        self.peak_behind_by(x)
    }

    #[inline(always)]
    fn expect_without_data(&self, token: TokenType) -> bool {
        discriminant(&self.current_token().unwrap().token_type) == discriminant(&token)
    }

    #[inline(always)]
    fn expect(&self, token: TokenType) -> bool {
        self.current_token().unwrap().token_type == token
    }
}

// #############################
// #
// # Instruction Generators
// #
// #############################
impl Parser {
    fn expr(&mut self) -> Instruction {
        let v = self.binary_op(
            Parser::comparison_expr,
            Parser::comparison_expr,
            &[TokenType::And, TokenType::Or],
        );
        if self.peak().is_some()
            && matches!(
                self.peak().unwrap().token_type,
                TokenType::Keyword(Keyword::As)
            )
        {
            self.advance();
            self.advance();
            let convert_type = Type::from(self.current_token().unwrap());
            return Instruction::new(
                v.start.clone(),
                self.current_token().unwrap().end.clone(),
                self.file_data.clone(),
                InstructionType::As {
                    convert_type,
                    value: Box::new(v),
                },
            );
        }
        v
    }

    fn comparison_expr(&mut self) -> Instruction {
        if self.current_token().is_some()
            && self.current_token().unwrap().token_type == TokenType::ExclamationMark
        {
            let start = self.current_token().unwrap().start.clone();
            self.advance();
            let comp_expr = self.comparison_expr();
            return Instruction::new(
                start,
                comp_expr.end.clone(),
                self.file_data.clone(),
                InstructionType::UnaryOperation {
                    value: Box::new(comp_expr),
                    operator: UnaryOperator::Not,
                },
            );
        }
        self.binary_op(
            Parser::arith_expr,
            Parser::arith_expr,
            &[
                TokenType::EqualsTo,
                TokenType::NotEquals,
                TokenType::GreaterEquals,
                TokenType::SmallerEquals,
                TokenType::LeftAngle,
                TokenType::RightAngle,
            ],
        )
    }

    fn arith_expr(&mut self) -> Instruction {
        self.binary_op(
            Parser::term,
            Parser::term,
            &[TokenType::Plus, TokenType::Minus],
        )
    }

    fn term(&mut self) -> Instruction {
        self.binary_op(
            Parser::factor,
            Parser::factor,
            &[TokenType::Multiply, TokenType::Slash],
        )
    }

    fn power(&mut self) -> Instruction {
        self.binary_op(Parser::atom, Parser::factor, &[TokenType::Power])
    }

    fn factor(&mut self) -> Instruction {
        if self.current_token().is_some()
            && [TokenType::Plus, TokenType::Minus]
                .contains(&self.current_token().unwrap().token_type)
        {
            let start_pos = self.current_token().unwrap().start.clone();
            let operator = UnaryOperator::from(self.current_token().unwrap());
            self.advance();
            let factor = self.factor();
            return Instruction::new(
                start_pos,
                factor.end.clone(),
                self.file_data.clone(),
                InstructionType::UnaryOperation {
                    value: Box::new(factor),
                    operator,
                },
            );
        }
        self.power()
    }

    fn atom(&mut self) -> Instruction {
        let instruction = match self.current_token().cloned() {
            Some(token) => {
                // TODO: Maybe there's a way to avoid cloning the token
                match token.token_type {
                    TokenType::Keyword(keyword) => match keyword {
                        Keyword::If => self.if_statement(),
                        Keyword::Class => self.class_declaration(),
                        Keyword::Return => self.return_statement(),
                        Keyword::Break => self.break_statement(),
                        Keyword::New => self.class_instantiate(),
                        Keyword::While => self.while_statement(),
                        Keyword::Do => self.do_catch(),
                        _ => panic!("how tf"),
                    },
                    TokenType::Indent => {
                        self.advance();
                        self.parse_section()
                    }
                    TokenType::LeftParenthesis => {
                        self.advance();
                        let mut expr = self.expr();
                        self.advance();
                        self.skip_new_line();
                        if self.current_token().is_none()
                            || !matches!(
                                self.current_token().unwrap().token_type,
                                TokenType::RightParenthesis
                            )
                        {
                            UnterminatedParenthesis::call(&expr.start, &expr.end, &expr.file_data)
                        }
                        expr.start = token.start.advance_by_owned(-2);
                        expr.end.advance_by(1);
                        expr
                    }
                    TokenType::Identifier(_) => self.identifier_expression(),
                    TokenType::NewLine => {
                        self.advance();
                        self.atom()
                    }
                    _ => {
                        let data = Data::from(&token);
                        Instruction::new(
                            token.start.clone(),
                            token.end.clone(),
                            token.file_data.clone(),
                            InstructionType::Data(data),
                        )
                    }
                }
            }
            None => UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "an integer, float, or identifier",
                self.current_token_type_str().as_str(),
            ),
        };
        instruction
    }

    fn binary_op(
        &mut self,
        left_func: fn(&mut Parser) -> Instruction,
        right_func: fn(&mut Parser) -> Instruction,
        operation_tokens: &[TokenType],
    ) -> Instruction {
        let mut left = left_func(self);
        self.advance();
        while self.current_token().is_some()
            && operation_tokens.contains(&self.current_token().unwrap().token_type)
        {
            let operator = BinaryOperator::from(self.current_token().unwrap(), operation_tokens);
            self.advance();
            let right = right_func(self);
            self.advance();
            left = Instruction::new(
                left.start.clone(),
                right.end.clone(),
                self.file_data.clone(),
                InstructionType::BinaryOperation {
                    left: Box::new(left),
                    right: Box::new(right),
                    operator,
                },
            );
        }
        self.retreat();
        left
    }
}

impl Parser {
    fn identifier_statement(&mut self) -> Instruction {
        fn change_assign(slf: &mut Parser) -> Instruction {
            let start = slf.current_token().unwrap().start.clone();
            let identifier = match slf.current_token_type() {
                TokenType::Identifier(v) => v.clone(),
                _ => UnexpectedToken::call(
                    slf.current_token().unwrap().fetch(),
                    "[identifier]",
                    slf.current_token_type_str().as_str(),
                ),
            };
            slf.advance();
            let operator = BinaryOperator::from(
                slf.current_token().unwrap(),
                &[
                    TokenType::PlusEquals,
                    TokenType::MinusEquals,
                    TokenType::MultiplyEquals,
                    TokenType::DivideEquals,
                    TokenType::PowerEquals,
                ],
            );
            slf.advance();
            let expr = slf.expr();
            Instruction::new(
                start.clone(),
                expr.end.clone(),
                slf.file_data.clone(),
                InstructionType::BinaryOperation {
                    left: Box::new(Instruction::new(
                        start,
                        expr.end.clone(),
                        slf.file_data.clone(),
                        InstructionType::VarAccess { identifier },
                    )),
                    right: Box::new(expr),
                    operator,
                },
            )
        }
        match &match self.current_token() {
            Some(v) => v,
            None => UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "[identifier]",
                self.current_token_type_str().as_str(),
            ),
        }
        .token_type
        {
            TokenType::Identifier(v) => v,
            _ => UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "[identifier]",
                self.current_token_type_str().as_str(),
            ),
        };
        if let Some(x) = self.peak() {
            match x.token_type {
                TokenType::PlusEquals
                | TokenType::MinusEquals
                | TokenType::MultiplyEquals
                | TokenType::PowerEquals
                | TokenType::DivideEquals => change_assign(self),
                TokenType::Equals => self.update_variable(),
                _ => self.expr(),
            }
        } else {
            self.expr()
        }
    }

    fn identifier_expression(&mut self) -> Instruction {
        let initial_value = match &self.current_token().unwrap().token_type {
            TokenType::Identifier(identifier) => match self.peak().unwrap().token_type {
                TokenType::LeftParenthesis => self.function_call(),
                _ => Instruction::new(
                    self.current_token().unwrap().start.clone(),
                    self.current_token().unwrap().end.clone(),
                    self.file_data.clone(),
                    InstructionType::VarAccess {
                        identifier: identifier.clone(),
                    },
                ),
            },
            _ => UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "[identifier]",
                self.current_token_type_str().as_str(),
            ),
        };
        if self.peak().is_some() && self.peak().unwrap().token_type == TokenType::Dot {
            self.advance();
            self.advance();
            let run = self.identifier_statement();
            Instruction::new(
                initial_value.start.clone(),
                run.end.clone(),
                self.file_data.clone(),
                InstructionType::InContextOf {
                    context_of: Box::new(initial_value),
                    run: Box::new(run),
                },
            )
        } else {
            initial_value
        }
    }
    fn update_variable(&mut self) -> Instruction {
        enum Type {
            Equals,
            PlusEquals,
            MinusEquals,
            MultiplyEquals,
            DivideEquals,
            PowerEquals,
        }
        let start = self.current_token().unwrap().start.clone();
        let identifier = match &match self.current_token() {
            Some(v) => v,
            None => UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "[identifier] = [expression]",
                self.current_token_type_str().as_str(),
            ),
        }
        .token_type
        {
            TokenType::Identifier(v) => v,
            _ => UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "[identifier] = [expression]",
                self.current_token_type_str().as_str(),
            ),
        }
        .clone();
        self.advance();
        let conversion_type = match self.current_token().unwrap().token_type {
            TokenType::Equals => Type::Equals,
            TokenType::PlusEquals => Type::PlusEquals,
            TokenType::MinusEquals => Type::MinusEquals,
            TokenType::DivideEquals => Type::DivideEquals,
            TokenType::MultiplyEquals => Type::MultiplyEquals,
            TokenType::PowerEquals => Type::PowerEquals,
            _ => UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                format!("{} = [expression]", identifier).as_str(),
                self.current_token_type_str().as_str(),
            ),
        };
        self.advance();
        let expr = self.expr();
        Instruction::new(
            start.clone(),
            self.current_token().unwrap().end.clone(),
            self.file_data.clone(),
            InstructionType::VarUpdate {
                identifier: identifier.clone(),
                data: Box::new(match conversion_type {
                    Type::Equals => expr,
                    Type::PlusEquals
                    | Type::MinusEquals
                    | Type::MultiplyEquals
                    | Type::PowerEquals
                    | Type::DivideEquals => Instruction::new(
                        start.clone(),
                        self.current_token().unwrap().end.clone(),
                        self.file_data.clone(),
                        InstructionType::BinaryOperation {
                            left: Box::new(Instruction::new(
                                start,
                                self.current_token().unwrap().end.clone(),
                                self.file_data.clone(),
                                InstructionType::VarAccess {
                                    identifier: identifier.clone(),
                                },
                            )),
                            right: Box::new(expr),
                            operator: match conversion_type {
                                Type::PlusEquals => BinaryOperator::Add,
                                Type::MinusEquals => BinaryOperator::Remove,
                                Type::MultiplyEquals => BinaryOperator::Multiply,
                                Type::DivideEquals => BinaryOperator::Divide,
                                Type::Equals => UnexpectedToken::call(
                                    self.current_token().unwrap().fetch(),
                                    format!("{} = [expression]", identifier).as_str(),
                                    self.current_token()
                                        .unwrap()
                                        .token_type
                                        .to_string()
                                        .as_str(),
                                ),
                                Type::PowerEquals => BinaryOperator::Power,
                            },
                        },
                    ),
                }),
            },
        )
    }
    fn use_statement(&mut self) -> Instruction {
        let start = self.current_token().unwrap().start.clone();
        if !self.expect(TokenType::Keyword(Keyword::Use)) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "use \"[file path]\"",
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        let file_path = match &match self.current_token() {
            Some(v) => v,
            None => UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "use \"[file path]\"",
                self.current_token_type_str().as_str(),
            ),
        }
        .token_type
        {
            TokenType::String(v) => v,
            _ => UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "use \"[file path]\"",
                self.current_token_type_str().as_str(),
            ),
        }
        .clone();
        Instruction::new(
            start,
            self.current_token().unwrap().end.clone(),
            self.file_data.clone(),
            InstructionType::UseStatement { file_path },
        )
    }

    fn pass_statement(&mut self) -> Instruction {
        Instruction::new(
            self.current_token().unwrap().start.clone(),
            self.current_token().unwrap().end.clone(),
            self.file_data.clone(),
            InstructionType::Pass,
        )
    }

    fn var_statement(&mut self) -> Instruction {
        let start = self.current_token().unwrap().start.clone();
        let is_final = if self.expect(TokenType::Keyword(Keyword::Final)) {
            self.advance();
            if !self.expect(TokenType::Keyword(Keyword::Var)) {
                UnexpectedToken::call(
                    self.current_token().unwrap().fetch(),
                    "final var [identifier] = [expression]",
                    self.current_token_type_str().as_str(),
                )
            }
            true
        } else {
            if !self.expect(TokenType::Keyword(Keyword::Var)) {
                UnexpectedToken::call(
                    self.current_token().unwrap().fetch(),
                    "var [identifier] = [expression]",
                    self.current_token_type_str().as_str(),
                )
            }
            false
        };
        self.advance();
        let identifier = match &match self.current_token() {
            Some(v) => v,
            None => UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "var [identifier] = [expression]",
                self.current_token_type_str().as_str(),
            ),
        }
        .token_type
        {
            TokenType::Identifier(v) => v,
            _ => UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "var [identifier] = [expression]",
                self.current_token_type_str().as_str(),
            ),
        }
        .clone();
        let type_hint =
            if self.peak().is_some() && self.peak().unwrap().token_type == TokenType::Colon {
                self.advance();
                self.advance();
                let x = Type::from(self.current_token().unwrap());
                x
            } else {
                Type::new(
                    TypeHint::None,
                    start.clone(),
                    self.current_token().unwrap().end.clone(),
                    self.file_data.clone(),
                )
            };
        self.advance();
        let data = Box::new(if !self.expect(TokenType::Equals) {
            if !self.expect(TokenType::NewLine) && !self.expect(TokenType::EndOfFile) {
                UnexpectedToken::call(
                    self.current_token().unwrap().fetch(),
                    &format!("var {} = [expression]", identifier),
                    self.current_token()
                        .unwrap()
                        .token_type
                        .to_string()
                        .as_str(),
                )
            }
            self.retreat();
            Instruction::new(
                start.clone(),
                self.current_token().unwrap().end.clone(),
                self.file_data.clone(),
                InstructionType::Data(Data::new(
                    self.file_data.clone(),
                    start.clone(),
                    self.current_token().unwrap().end.clone(),
                    data::DataType::Null,
                )),
            )
        } else {
            self.advance();
            self.expr()
        });
        Instruction::new(
            start,
            self.current_token().unwrap().end.clone(),
            self.file_data.clone(),
            InstructionType::VarAssign {
                identifier,
                data,
                type_hint,
                is_final,
            },
        )
    }

    fn if_statement(&mut self) -> Instruction {
        self.if_statement_w_start(self.current_token().unwrap().start.clone())
    }

    fn if_statement_w_start(&mut self, start: Position) -> Instruction {
        if !self.expect(TokenType::Keyword(Keyword::If)) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "if [expression] { ... }",
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        let expr = self.expr();
        self.advance();
        self.skip_new_line();
        if !self.expect(TokenType::Indent) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "if [expression] { ... }",
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        let body = self.parse_section();
        self.advance();
        let mut else_value: Option<Box<Instruction>> = None;
        while let Some(token) = self.current_token() {
            let start = token.start.clone();
            if !self.expect(TokenType::Keyword(Keyword::Else)) {
                break;
            }
            else_value = Some(Box::new(if let Some(token) = self.peak() {
                if token.token_type != TokenType::Keyword(Keyword::If) {
                    self.else_statement()
                } else {
                    self.advance();
                    self.if_statement_w_start(start)
                }
            } else {
                self.else_statement()
            }));
        }
        self.retreat();
        Instruction::new(
            start,
            self.current_token().unwrap().end.clone(),
            self.file_data.clone(),
            InstructionType::IfStatement {
                condition: Some(Box::new(expr)),
                body: Box::new(body),
                else_value,
            },
        )
    }

    fn else_statement(&mut self) -> Instruction {
        // println!("else");
        let start = self.current_token().unwrap().start.clone();
        if !self.expect(TokenType::Keyword(Keyword::Else)) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "else { ... }",
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        self.skip_new_line();
        if !self.expect(TokenType::Indent) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "else { ... }",
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        let body = self.parse_section();
        self.advance();
        Instruction::new(
            start,
            body.end.clone(),
            self.file_data.clone(),
            InstructionType::IfStatement {
                condition: None,
                body: Box::new(body),
                else_value: None,
            },
        )
    }

    fn while_statement(&mut self) -> Instruction {
        let start = self.current_token().unwrap().start.clone();
        if !self.expect(TokenType::Keyword(Keyword::While)) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "while [expression] { ... }",
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        let expr = self.expr();
        self.advance();
        self.skip_new_line();
        if !self.expect(TokenType::Indent) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "while [expression] { ... }",
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        let body = self.parse_section();
        self.retreat();
        Instruction::new(
            start,
            self.current_token().unwrap().end.clone(),
            self.file_data.clone(),
            InstructionType::WhileStatement {
                condition: Box::new(expr),
                body: Box::new(body),
            },
        )
    }

    fn return_statement(&mut self) -> Instruction {
        let start = self.current_token().unwrap().start.clone();
        if !self.expect(TokenType::Keyword(Keyword::Return)) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "return [expression]",
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        let expr = self.expr();
        Instruction::new(
            start,
            self.current_token().unwrap().end.clone(),
            self.file_data.clone(),
            InstructionType::ReturnStatement {
                value: Box::new(expr),
            },
        )
    }

    fn break_statement(&mut self) -> Instruction {
        let start = self.current_token().unwrap().start.clone();
        if !self.expect(TokenType::Keyword(Keyword::Break)) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "break",
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        if self.current_token_type() == &TokenType::NewLine {
            self.retreat();
            return Instruction::new(
                start.clone(),
                self.current_token().unwrap().end.clone(),
                self.file_data.clone(),
                InstructionType::BreakStatement {
                    value: Box::new(Instruction::new(
                        start.clone(),
                        self.current_token().unwrap().end.clone(),
                        self.file_data.clone(),
                        InstructionType::Data(Data::new(
                            self.file_data.clone(),
                            start,
                            self.current_token().unwrap().end.clone(),
                            data::DataType::Null,
                        )),
                    )),
                },
            );
        }
        let expr = self.expr();
        Instruction::new(
            start,
            self.current_token().unwrap().end.clone(),
            self.file_data.clone(),
            InstructionType::BreakStatement {
                value: Box::new(expr),
            },
        )
    }

    fn function_declaration(&mut self) -> Instruction {
        let start = self.current_token().unwrap().start.clone();
        if !self.expect(TokenType::Keyword(Keyword::Function)) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "function [identifier]() { ... }",
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        self.skip_new_line();
        let identifier = match &self.current_token().unwrap().token_type {
            TokenType::Identifier(identifier) => identifier.clone(),
            _ => UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "function [identifier]() { ... }",
                self.current_token_type_str().as_str(),
            ),
        };
        self.advance();
        if !self.expect(TokenType::LeftParenthesis) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "function [identifier]() { ... }",
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        let mut arguments = vec![];
        while self.current_token().is_some() {
            if !arguments.is_empty() {
                match self.current_token_type() {
                    TokenType::Comma => self.advance(),
                    _ => UnexpectedToken::call(
                        self.current_token().unwrap().fetch(),
                        ",",
                        &self.current_token_type_str(),
                    ),
                }
            }
            match &self.current_token().unwrap().token_type {
                TokenType::Identifier(identifier) => {
                    arguments.push((
                        identifier.clone(),
                        if self.peak().is_some()
                            && self.peak().unwrap().token_type == TokenType::Colon
                        {
                            self.advance();
                            self.advance();
                            match self.current_token() {
                                Some(v) => match v.token_type {
                                    TokenType::TypeHint(_) | TokenType::Identifier(_) => {
                                        Type::from(self.current_token().unwrap())
                                    }
                                    _ => UnexpectedToken::call(
                                        self.current_token().unwrap().fetch(),
                                        "a type hint",
                                        self.current_token()
                                            .unwrap()
                                            .token_type
                                            .to_string()
                                            .as_str(),
                                    ),
                                },
                                None => UnexpectedToken::call(
                                    self.current_token().unwrap().fetch(),
                                    "a type hint",
                                    self.current_token_type_str().as_str(),
                                ),
                            }
                        } else {
                            Type::new(
                                TypeHint::None,
                                start.clone(),
                                self.current_token().unwrap().end.clone(),
                                self.file_data.clone(),
                            )
                        },
                        if self.peak().is_some()
                            && self.peak().unwrap().token_type == TokenType::Equals
                        {
                            self.advance();
                            self.advance();
                            Some(self.expr())
                        } else {
                            None
                        },
                    ));
                }
                TokenType::RightParenthesis => break,
                _ => UnexpectedToken::call(
                    self.current_token().unwrap().fetch(),
                    ")",
                    &self.current_token_type_str(),
                ),
            }
            self.advance();
            if self.current_token_type() == &TokenType::RightParenthesis {
                break;
            }
        }
        if !self.expect(TokenType::RightParenthesis) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "function [identifier]() { ... }",
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        if !self.expect(TokenType::Indent) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "function [identifier]() { ... }",
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        let body = self.parse_section();
        Instruction::new(
            start,
            body.end.clone(),
            self.file_data.clone(),
            InstructionType::FunctionDeclaration {
                identifier,
                body: Box::new(body),
                arguments,
            },
        )
    }

    fn function_call(&mut self) -> Instruction {
        let start = self.current_token().unwrap().start.clone();
        let identifier = match self.current_token_type() {
            TokenType::Identifier(v) => v,
            _ => UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "[identifier]()",
                self.current_token_type_str().as_str(),
            ),
        }
        .clone();
        self.advance();
        if !self.expect(TokenType::LeftParenthesis) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                &format!("{}()", identifier),
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        let arguments = self.function_arguments();
        self.advance();
        if !self.expect(TokenType::RightParenthesis) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "[identifier]()",
                self.current_token_type_str().as_str(),
            )
        }
        Instruction::new(
            start,
            self.current_token().unwrap().end.clone(),
            self.file_data.clone(),
            InstructionType::FunctionCall {
                identifier,
                arguments,
            },
        )
    }

    fn do_catch(&mut self) -> Instruction {
        let start = self.current_token().unwrap().start.clone();
        if !self.expect(TokenType::Keyword(Keyword::Do)) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "do { ... } catch { ... }",
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        if !self.expect(TokenType::Indent) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "do { ... } catch { ... }",
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        let do_body = self.parse_section();
        self.advance();
        if !self.expect(TokenType::Keyword(Keyword::Catch)) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "do { ... } catch { ... }",
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        if !self.expect(TokenType::Indent) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "do { ... } catch { ... }",
                self.current_token_type_str().as_str(),
            )
        }
        let catch_body = self.parse_section();
        Instruction::new(
            start,
            self.current_token().unwrap().end.clone(),
            self.file_data.clone(),
            InstructionType::DoCatch {
                catch_body: Box::new(catch_body),
                do_body: Box::new(do_body),
            },
        )
    }

    fn class_declaration(&mut self) -> Instruction {
        let start = self.current_token().unwrap().start.clone();
        if !self.expect(TokenType::Keyword(Keyword::Class)) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "class [identifier] { ... }",
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        let identifier = match self.current_token_type() {
            TokenType::Identifier(v) => v,
            _ => UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "class [identifier] { ... }",
                self.current_token_type_str().as_str(),
            ),
        }
        .clone();
        self.advance();
        self.advance();
        self.advance();
        // println!("{}", self.current_token_type_str());
        let mut instructions = vec![];
        while self.current_token().is_some() {
            if let Some(token) = self.current_token() {
                if token.token_type == TokenType::Dedent || token.token_type == TokenType::EndOfFile
                {
                    break;
                }
            }
            let instruction = self.parse_once();
            // println!("{:?}", instruction.instruction_type);
            match instruction.instruction_type {
                InstructionType::FunctionDeclaration { .. } | InstructionType::VarAssign { .. } => {
                    instructions.push(instruction)
                }
                _ => {
                    println!("{}", instruction.instruction_type);
                    InvalidInstructionInClass::call(
                        &instruction.start,
                        &instruction.end,
                        &self.file_data,
                    )
                }
            }
        }
        let body = Box::new(Instruction::new(
            start.clone(),
            self.current_token().unwrap().end.clone(),
            self.file_data.clone(),
            InstructionType::Section { body: instructions },
        ));
        Instruction::new(
            start,
            self.current_token().unwrap().end.clone(),
            self.file_data.clone(),
            InstructionType::ClassDeclaration { identifier, body },
        )
    }

    fn class_instantiate(&mut self) -> Instruction {
        let start = self.current_token().unwrap().start.clone();
        if !self.expect(TokenType::Keyword(Keyword::New)) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "new [identifier]()",
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        let identifier = match self.current_token_type() {
            TokenType::Identifier(v) => v,
            _ => UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                "new [identifier]()",
                self.current_token_type_str().as_str(),
            ),
        }
        .clone();
        self.advance();
        if !self.expect(TokenType::LeftParenthesis) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                &format!("new {}()", identifier),
                self.current_token_type_str().as_str(),
            )
        }
        self.advance();
        let constructor_arguments = self.function_arguments();
        self.advance();
        if !self.expect(TokenType::RightParenthesis) {
            UnexpectedToken::call(
                self.current_token().unwrap().fetch(),
                &format!("new {}()", identifier),
                self.current_token_type_str().as_str(),
            )
        }
        Instruction::new(
            start,
            self.current_token().unwrap().end.clone(),
            self.file_data.clone(),
            InstructionType::ClassInstantiation {
                identifier,
                constructor_arguments,
            },
        )
    }

    fn annotation(&mut self) -> Instruction {
        match match self.current_token_type() {
            TokenType::Annotation(v) => v,
            _ => panic!(),
        }
        .clone()
        {
            Annotation::DocComment(v) => {
                let start = self.current_token().unwrap().start.clone();
                self.advance();
                let value = Box::new(self.parse_once());
                Instruction::new(
                    start,
                    self.current_token().unwrap().end.clone(),
                    self.file_data.clone(),
                    InstructionType::DocComment { comment: v, value },
                )
            }
        }
    }

    fn function_arguments(&mut self) -> Vec<Instruction> {
        let mut arguments = vec![];
        while self.current_token().is_some() {
            self.skip_new_line();
            if !arguments.is_empty() {
                match self.current_token_type() {
                    TokenType::RightParenthesis => break,
                    TokenType::Comma => self.advance(),
                    _ => UnexpectedToken::call(
                        self.current_token().unwrap().fetch(),
                        ",",
                        &self.current_token_type_str(),
                    ),
                }
            }
            match &self.current_token().unwrap().token_type {
                TokenType::RightParenthesis => break,
                _ => arguments.push(self.expr()),
            }
            self.advance()
        }
        self.retreat();
        arguments
    }
}
