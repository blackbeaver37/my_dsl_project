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

/// 표현식(Expression) 구조
/// transform 구문의 우측 값에 해당
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Field(String),              // 예: @문제
    Literal(String),            // 예: "내용"
    Concat(Vec<Expression>),    // 예: @과목 + "_" + @학년
}

/// DSL 명령어(Command) 구조
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Input(String),                      // input "파일";
    Output(String),                     // output "파일";
    Print,                              // print;
    PrintLine(usize),                   // print line 1;
    Transform(Vec<(String, Expression)>),  // transform { key = expr; ... }
}

// ==========================================================
// ✅ Parser 구조체 정의
// ==========================================================

/// DSL 파서: 토큰 벡터를 받아 명령어(Command) 리스트로 변환
pub struct Parser {
    tokens: Vec<Token>,  // 토큰 벡터
    position: usize,     // 현재 파싱 위치 인덱스
}

impl Parser {
    /// Parser 생성자
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// 현재 위치의 토큰 반환
    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// 다음 토큰으로 이동
    fn advance(&mut self) {
        self.position += 1;
    }

    /// 예상 토큰이 맞는지 확인하고 advance
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

    /// 전체 명령어 파싱 시작
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

    /// input "파일";
    fn parse_input(&mut self) -> Result<Command, String> {
        self.advance(); // input

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

    /// output "파일";
    fn parse_output(&mut self) -> Result<Command, String> {
        self.advance(); // output

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

    /// print; 또는 print line N;
    fn parse_print(&mut self) -> Result<Command, String> {
        self.advance(); // print

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

    /// transform { key = expr; ... }
    fn parse_transform(&mut self) -> Result<Command, String> {
        self.advance(); // transform
        self.expect(&Token::LeftBrace)?; // {

        let mut transforms = Vec::new();

        while let Some(token) = self.current_token() {
            match token {
                Token::RightBrace => {
                    self.advance(); // }
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

    /// 표현식 파싱
    /// 예: @필드, "문자열", @필드 + "_" + @필드
    fn parse_expression(&mut self) -> Result<Expression, String> {
        let mut parts = Vec::new();

        loop {
            let expr = match self.current_token() {
                Some(Token::Field(name)) => {
                    let name = name.clone();
                    self.advance();
                    Expression::Field(name)
                }
                Some(Token::StringLiteral(s)) => {
                    let s = s.clone();
                    self.advance();
                    Expression::Literal(s)
                }
                other => return Err(format!("Unexpected token in expression: {:?}", other)),
            };

            parts.push(expr);

            // + 연산이 있는 경우 → Concat 계속 진행
            match self.current_token() {
                Some(Token::Plus) => {
                    self.advance(); // + 소비
                }
                _ => break, // 종료
            }
        }

        if parts.len() == 1 {
            Ok(parts.remove(0))
        } else {
            Ok(Expression::Concat(parts))
        }
    }
}
