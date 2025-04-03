use crate::lexer::Token;

// ==========================================================
// ✅ 표현식(Expression)과 명령어(Command) 구조 정의
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
    FieldPath(Vec<String>),                       // ✅ 단일 필드 or 중첩 경로
    FieldWithModifiers(FieldWithModifiers),       // ✅ 경로 + 수정자
    Literal(String),
    Concat(Vec<Expression>),
    RawRecord,
    Serial,
}

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

    /// 🔹 수정자 파싱: .prefix("x").suffix("y") 등
    fn parse_modifiers(&mut self) -> Result<Vec<FieldModifier>, String> {
        let mut modifiers = Vec::new();

        while let Some(Token::Dot) = self.current_token() {
            // 미리 다음 두 토큰을 clone 해서 immutable 참조 끊기
            let lookahead1 = self.tokens.get(self.position + 1).cloned();
            let lookahead2 = self.tokens.get(self.position + 2).cloned();

            match (lookahead1, lookahead2) {
                (Some(Token::Identifier(name)), Some(Token::LParen)) => {
                    self.advance(); // Dot
                    self.advance(); // Identifier
                    let modifier_name = name;

                    self.expect(&Token::LParen)?;
                    let value = match self.current_token() {
                        Some(Token::StringLiteral(s)) => {
                            let s = s.clone();
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

    /// 🔹 표현식 파싱 (문자열, 필드, 함수, 연결 등)
    fn parse_expression(&mut self) -> Result<Expression, String> {
        let mut parts = Vec::new();

        loop {
            let expr = match self.current_token() {
                Some(Token::Field(first)) => {
                    let mut path = vec![first.clone()];
                    self.advance();

                    while let Some(Token::Dot) = self.current_token() {
                        let lookahead1 = self.tokens.get(self.position + 1).cloned();
                        let lookahead2 = self.tokens.get(self.position + 2).cloned();

                        match (lookahead1, lookahead2) {
                            (Some(Token::Identifier(id)), Some(Token::LParen)) => {
                                // modifier 시작이므로 루프 탈출
                                break;
                            }
                            (Some(Token::Identifier(id)), _) => {
                                self.advance(); // Dot
                                self.advance(); // Identifier
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
