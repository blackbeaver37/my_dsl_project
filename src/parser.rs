//! ✅ DSL 파서
//!
//! 토큰(Token) 리스트를 의미 있는 명령어(Command)와 표현식(Expression)으로 변환 (AST 생성)

use crate::lexer::Token;

// ==========================================================
// ✅ DSL 내부 구조 정의
// ==========================================================

#[derive(Debug, Clone, PartialEq)]
pub enum FieldModifier {
    Suffix(String),
    Prefix(String),
    Default(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldWithModifiers {
    pub path: Vec<String>,
    pub modifiers: Vec<FieldModifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    FieldPath(Vec<String>),
    FieldWithModifiers(FieldWithModifiers),
    Literal(String),
    Concat(Vec<Expression>),
    RawRecord,
    Serial,
    Variable(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Input(String),
    Output(String),
    Print,
    PrintLine(usize),
    Transform(Vec<(String, Expression)>),
    Let(String, Expression),
    Const(String, Expression),
}

// ==========================================================
// ✅ Parser 정의
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

    /// 🔹 DSL 전체 파싱
    pub fn parse(&mut self) -> Result<Vec<Command>, String> {
        let mut commands = Vec::new();

        while let Some(token) = self.current_token() {
            match token {
                Token::Comment(_) => {
                    self.advance();
                    continue;
                }
                Token::Input => commands.push(self.parse_input()?),
                Token::Output => commands.push(self.parse_output()?),
                Token::Print => commands.push(self.parse_print()?),
                Token::Transform => commands.push(self.parse_transform()?),
                Token::Let => commands.push(self.parse_let()?),
                Token::Const => commands.push(self.parse_const()?),
                other => return Err(format!("Unexpected token in command position: {:?}", other)),
            };
        }

        Ok(commands)
    }

    /// 🔹 input "파일"; 구문 파싱
    fn parse_input(&mut self) -> Result<Command, String> {
        self.advance();
        if let Some(Token::StringLiteral(path)) = self.current_token().cloned() {
            self.advance();
            self.expect(&Token::Semicolon)?;
            Ok(Command::Input(path))
        } else {
            Err(format!("Expected string literal after 'input', but found {:?}", self.current_token()))
        }
    }

    /// 🔹 output "파일"; 구문 파싱
    fn parse_output(&mut self) -> Result<Command, String> {
        self.advance();
        if let Some(Token::StringLiteral(path)) = self.current_token().cloned() {
            self.advance();
            self.expect(&Token::Semicolon)?;
            Ok(Command::Output(path))
        } else {
            Err(format!("Expected string literal after 'output', but found {:?}", self.current_token()))
        }
    }

    /// 🔹 print; 또는 print line 3; 구문 파싱
    fn parse_print(&mut self) -> Result<Command, String> {
        self.advance();
        match self.current_token() {
            Some(Token::Semicolon) => {
                self.advance();
                Ok(Command::Print)
            }
            Some(Token::Identifier(id)) if id == "line" => {
                self.advance();
                if let Some(Token::Number(n)) = self.current_token().cloned() {
                    self.advance();
                    self.expect(&Token::Semicolon)?;
                    Ok(Command::PrintLine(n))
                } else {
                    Err(format!("Expected number after 'print line', but found {:?}", self.current_token()))
                }
            }
            other => Err(format!("Unexpected token after 'print': {:?}", other)),
        }
    }

    /// 🔹 transform { key = expr; ... } 파싱
    fn parse_transform(&mut self) -> Result<Command, String> {
        self.advance();
        self.expect(&Token::LBrace)?;

        let mut transforms = Vec::new();

        while let Some(token) = self.current_token() {
            match token {
                Token::Comment(_) => {
                    self.advance();
                }
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

    /// 🔹 let name = expr; 파싱
    fn parse_let(&mut self) -> Result<Command, String> {
        self.advance(); // consume 'let'

        let name = match self.current_token() {
            Some(Token::Identifier(id)) => {
                let id = id.clone();
                self.advance();
                id
            }
            other => return Err(format!("Expected identifier after 'let', but found {:?}", other)),
        };

        self.expect(&Token::Equal)?;
        let expr = self.parse_expression()?;
        self.expect(&Token::Semicolon)?;
        Ok(Command::Let(name, expr))
    }

    /// 🔹 const name = expr; 파싱
    fn parse_const(&mut self) -> Result<Command, String> {
        self.advance(); // consume 'const'

        let name = match self.current_token() {
            Some(Token::Identifier(id)) => {
                let id = id.clone();
                self.advance();
                id
            }
            other => return Err(format!("Expected identifier after 'const', but found {:?}", other)),
        };

        self.expect(&Token::Equal)?;
        let expr = self.parse_expression()?;
        self.expect(&Token::Semicolon)?;
        Ok(Command::Const(name, expr))
    }

    /// 🔹 .prefix("x").suffix("y") 등 수정자 파싱
    fn parse_modifiers(&mut self) -> Result<Vec<FieldModifier>, String> {
        let mut modifiers = Vec::new();

        while let Some(Token::Dot) = self.current_token() {
            let lookahead1 = self.tokens.get(self.position + 1).cloned();
            let lookahead2 = self.tokens.get(self.position + 2).cloned();

            match (lookahead1, lookahead2) {
                (Some(Token::Identifier(name)), Some(Token::LParen)) => {
                    self.advance();
                    self.advance();
                    let modifier_name = name;

                    self.expect(&Token::LParen)?;
                    let value = match self.current_token().cloned() {
                        Some(Token::StringLiteral(s)) => {
                            self.advance();
                            s
                        }
                        _ => break,
                    };
                    self.expect(&Token::RParen)?;

                    let modifier = match modifier_name.as_str() {
                        "prefix" => FieldModifier::Prefix(value),
                        "suffix" => FieldModifier::Suffix(value),
                        "default" => FieldModifier::Default(value),
                        _ => break,
                    };

                    modifiers.push(modifier);
                }
                _ => break,
            }
        }

        Ok(modifiers)
    }

    /// 🔹 표현식 파싱: 필드, 리터럴, 함수 호출, 연결 등
    fn parse_expression(&mut self) -> Result<Expression, String> {
        let mut parts = Vec::new();

        loop {
            let expr = match self.current_token() {
                Some(Token::Comment(_)) => {
                    self.advance();
                    continue;
                }

                Some(Token::Field(first)) => {
                    let mut path = vec![first.clone()];
                    self.advance();

                    while let Some(Token::Dot) = self.current_token() {
                        let lookahead1 = self.tokens.get(self.position + 1).cloned();
                        let lookahead2 = self.tokens.get(self.position + 2).cloned();

                        match (lookahead1, lookahead2) {
                            (Some(Token::Identifier(_)), Some(Token::LParen)) => break,
                            (Some(Token::Identifier(id)), _) => {
                                self.advance();
                                self.advance();
                                path.push(id);
                            }
                            _ => break,
                        }
                    }

                    let modifiers = self.parse_modifiers()?;
                    if modifiers.is_empty() {
                        Expression::FieldPath(path)
                    } else {
                        Expression::FieldWithModifiers(FieldWithModifiers { path, modifiers })
                    }
                }

                Some(Token::StringLiteral(s)) => {
                    let s = s.clone();
                    self.advance();
                    Expression::Literal(s)
                }

                Some(Token::Identifier(id)) if id == "raw" => {
                    self.advance();
                    self.expect(&Token::LParen)?;
                    self.expect(&Token::RParen)?;
                    Expression::RawRecord
                }

                Some(Token::Identifier(id)) if id == "serial" => {
                    self.advance();
                    self.expect(&Token::LParen)?;
                    self.expect(&Token::RParen)?;
                    Expression::Serial
                }

                Some(Token::Identifier(id)) => {
                    let var_name = id.clone();
                    self.advance();
                    Expression::Variable(var_name)
                }

                other => return Err(format!("Unexpected token in expression: {:?}", other)),
            };

            parts.push(expr);

            match self.current_token() {
                Some(Token::Plus) => {
                    self.advance();
                }
                Some(Token::Comment(_)) => {
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
