//! ✅ DSL 파서 (Parser)
//!
//! 이 모듈은 토큰 벡터(Token)를 받아서,
//! 각 명령어를 의미 있는 Command 구조로 변환(AST로 생성)한다.
//!
//! 지원 명령어:
//! - input "파일명";
//! - output "파일명";
//! - print;
//! - print line 1;
//! - transform { 필드 = 표현식; ... }

use crate::lexer::Token;

// ==========================================================
// ✅ 표현식(Expression)과 명령어(Command) 구조 정의
// ==========================================================

/// 필드 후처리 기능에 대한 Modifier 정의
#[derive(Debug, Clone, PartialEq)]
pub enum FieldModifier {
    Suffix(String),
    Prefix(String),
    Default(String),
}

/// 후처리 가능한 필드 구조체
#[derive(Debug, Clone, PartialEq)]
pub struct FieldWithModifiers {
    pub name: String,
    pub modifiers: Vec<FieldModifier>, // 여러 modifier 지원
}

/// 표현식(Expression) 구조
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Field(String),
    FieldWithModifiers(FieldWithModifiers), // 구조체로 분리
    Literal(String),
    Concat(Vec<Expression>),
}

/// DSL 명령어(Command) 구조
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Input(String),
    Output(String),
    Print,
    PrintLine(usize),
    Transform(Vec<(String, Expression)>),
}

// ==========================================================
// ✅ Parser 구조체 정의
// ==========================================================

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        match self.current_token() {
            Some(token) if token == expected => {
                self.advance();
                Ok(())
            }
            Some(token) => Err(format!("Expected token {:?}, but found {:?}", expected, token)),
            None => Err("Expected token but found end of input.".to_string()),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Command>, String> {
        let mut commands = Vec::new();

        while let Some(token) = self.current_token() {
            let command = match token {
                Token::Input => self.parse_input()?,
                Token::Output => self.parse_output()?,
                Token::Print => self.parse_print()?,
                Token::Transform => self.parse_transform()?,
                other => return Err(format!("Unexpected token in command position: {:?}", other)),
            };
            commands.push(command);
        }

        Ok(commands)
    }

    fn parse_input(&mut self) -> Result<Command, String> {
        self.advance();
        match self.current_token() {
            Some(Token::StringLiteral(path)) => {
                let path = path.clone();
                self.advance();
                self.expect(&Token::Semicolon)?;
                Ok(Command::Input(path))
            }
            other => Err(format!("Expected string literal after 'input', but found {:?}", other)),
        }
    }

    fn parse_output(&mut self) -> Result<Command, String> {
        self.advance();
        match self.current_token() {
            Some(Token::StringLiteral(path)) => {
                let path = path.clone();
                self.advance();
                self.expect(&Token::Semicolon)?;
                Ok(Command::Output(path))
            }
            other => Err(format!("Expected string literal after 'output', but found {:?}", other)),
        }
    }

    fn parse_print(&mut self) -> Result<Command, String> {
        self.advance();
        match self.current_token() {
            Some(Token::Semicolon) => {
                self.advance();
                Ok(Command::Print)
            }
            Some(Token::Identifier(id)) if id == "line" => {
                self.advance();
                match self.current_token() {
                    Some(Token::Number(n)) => {
                        let line_num = *n;
                        self.advance();
                        self.expect(&Token::Semicolon)?;
                        Ok(Command::PrintLine(line_num))
                    }
                    other => Err(format!("Expected number after 'print line', but found {:?}", other)),
                }
            }
            other => Err(format!("Unexpected token after 'print': {:?}", other)),
        }
    }

    fn parse_transform(&mut self) -> Result<Command, String> {
        self.advance();
        self.expect(&Token::LBrace)?;

        let mut transforms = Vec::new();

        while let Some(token) = self.current_token() {
            match token {
                Token::RBrace => {
                    self.advance();
                    break;
                }
                Token::Identifier(key) => {
                    let key = key.clone();
                    self.advance();
                    self.expect(&Token::Equal)?;
                    let expr = self.parse_expression()?;
                    self.expect(&Token::Semicolon)?;
                    transforms.push((key, expr));
                }
                other => {
                    return Err(format!("Unexpected token inside transform block: {:?}", other));
                }
            }
        }

        Ok(Command::Transform(transforms))
    }

    /// 🔹 필드 또는 수정자를 포함한 필드를 파싱하는 함수
    fn parse_field_with_modifiers(&mut self, name: String) -> Expression {
        let mut modifiers = Vec::new();
    
        // 반복적으로 .modifier("값") 형태를 파싱
        while let Some(Token::Dot) = self.current_token() {
            self.advance(); // consume '.'
    
            let modifier_name = match self.current_token() {
                Some(Token::Identifier(name)) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                other => break, // 잘못된 토큰이면 종료
            };
    
            // 괄호 시작
            if let Err(_) = self.expect(&Token::LParen) {
                break;
            }
    
            // 문자열 리터럴 값 가져오기
            let value = match self.current_token() {
                Some(Token::StringLiteral(s)) => {
                    let s = s.clone();
                    self.advance();
                    s
                }
                _ => break,
            };
    
            // 괄호 닫기
            if let Err(_) = self.expect(&Token::RParen) {
                break;
            }
    
            // Modifier enum에 추가
            let modifier = match modifier_name.as_str() {
                "prefix" => FieldModifier::Prefix(value),
                "suffix" => FieldModifier::Suffix(value),
                "default" => FieldModifier::Default(value),
                _ => break, // 알 수 없는 modifier
            };
    
            modifiers.push(modifier);
        }
    
        if modifiers.is_empty() {
            Expression::Field(name)
        } else {
            Expression::FieldWithModifiers(FieldWithModifiers {
                name,
                modifiers,
            })
        }
    }
    

    /// 🔹 문자열, 필드, 연결 표현식을 파싱하는 함수
    fn parse_expression(&mut self) -> Result<Expression, String> {
        let mut parts = Vec::new();

        loop {
            let expr = match self.current_token() {
                Some(Token::Field(name)) => {
                    let name = name.clone();
                    self.advance();
                    self.parse_field_with_modifiers(name)
                }
                Some(Token::StringLiteral(s)) => {
                    let s = s.clone();
                    self.advance();
                    Expression::Literal(s)
                }
                other => return Err(format!("Unexpected token in expression: {:?}", other)),
            };

            parts.push(expr);

            match self.current_token() {
                Some(Token::Plus) => {
                    self.advance();
                }
                _ => break,
            }
        }

        if parts.len() == 1 {
            Ok(parts.remove(0))
        } else {
            Ok(Expression::Concat(parts))
        }
    }
}
