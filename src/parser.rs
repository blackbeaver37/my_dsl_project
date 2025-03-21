use crate::lexer::Token; // lexer 모듈에서 Token 타입 가져오기

// ----------------------------------------------------------------------------------
// ✅ AST 구조 정의 (Command와 Expression)
// ----------------------------------------------------------------------------------

/// ✅ 표현식(Expression) 구조
/// - update 명령어 우측의 값으로 사용됨
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// @필드명 (예: @문제, @과목 등)
    Field(String),

    /// 문자열 리터럴 (예: "_" 또는 "정답입니다")
    Literal(String),

    /// 여러 표현식을 + 연산자로 연결한 복합 표현식
    /// 예: @과목 + "_" + @학년 → Concat([Field, Literal, Field])
    Concat(Vec<Expression>),
}

/// ✅ DSL 명령어(Command) 정의
/// - 각 DSL 명령어는 하나의 Command로 AST에 표현됨
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// input "경로";
    Input(String),

    /// output "경로";
    Output(String),

    /// print;
    Print,

    /// print line 2;
    PrintLine(usize),

    /// update {
    ///     필드명 = 표현식;
    ///     ...
    /// }
    Update(Vec<(String, Expression)>),
}

// ----------------------------------------------------------------------------------
// ✅ Parser 구조체 정의 및 구현
// ----------------------------------------------------------------------------------

/// ✅ Parser 구조체
/// - Token 리스트를 받아 위치를 추적하며 Command 리스트(AST)로 파싱
pub struct Parser {
    tokens: Vec<Token>, // 분석할 토큰 목록
    position: usize,    // 현재 파싱 위치
}

impl Parser {
    /// 🔹 Parser 생성자: Token 리스트를 받아 초기화
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// 🔹 현재 위치의 토큰을 반환 (없으면 None)
    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// 🔹 현재 위치를 한 칸 이동
    fn advance(&mut self) {
        self.position += 1;
    }

    /// 🔹 예상한 토큰이 맞는지 검사하고 통과하면 advance, 아니면 에러 반환
    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        match self.current_token() {
            Some(token) if token == expected => {
                self.advance();
                Ok(())
            }
            Some(token) => Err(format!("예상된 토큰 {:?} 대신 {:?} 발견", expected, token)),
            None => Err("예상된 토큰이지만 입력이 끝났습니다.".to_string()),
        }
    }

    /// 🔹 메인 파서 함수: Token 목록을 읽어 Command 목록으로 변환
    pub fn parse(&mut self) -> Result<Vec<Command>, String> {
        let mut commands = Vec::new();

        // 입력된 모든 토큰을 처리할 때까지 반복
        while let Some(token) = self.current_token() {
            match token {
                // 🔸 input "파일명";
                Token::Input => {
                    self.advance();

                    match self.current_token() {
                        Some(Token::StringLiteral(path)) => {
                            let path = path.clone();
                            self.advance();
                            self.expect(&Token::Semicolon)?;
                            commands.push(Command::Input(path));
                        }
                        other => {
                            return Err(format!("input 명령어 다음에는 문자열이 와야 합니다. 현재: {:?}", other));
                        }
                    }
                }

                // 🔸 output "파일명";
                Token::Output => {
                    self.advance();

                    match self.current_token() {
                        Some(Token::StringLiteral(path)) => {
                            let path = path.clone();
                            self.advance();
                            self.expect(&Token::Semicolon)?;
                            commands.push(Command::Output(path));
                        }
                        other => {
                            return Err(format!("output 명령어 다음에는 문자열이 와야 합니다. 현재: {:?}", other));
                        }
                    }
                }

                // 🔸 print; 또는 print line 2;
                Token::Print => {
                    self.advance();

                    match self.current_token() {
                        Some(Token::Semicolon) => {
                            self.advance();
                            commands.push(Command::Print);
                        }
                        Some(Token::Identifier(id)) if id == "line" => {
                            self.advance();

                            match self.current_token() {
                                Some(Token::Number(n)) => {
                                    let line_num = *n;
                                    self.advance();
                                    self.expect(&Token::Semicolon)?;
                                    commands.push(Command::PrintLine(line_num));
                                }
                                other => {
                                    return Err(format!("print line 다음에는 숫자가 와야 합니다: {:?}", other));
                                }
                            }
                        }
                        other => {
                            return Err(format!("print 명령어 뒤에 잘못된 토큰: {:?}", other));
                        }
                    }
                }

                // 🔸 update { ... }
                Token::Update => {
                    self.advance();
                    self.expect(&Token::LeftBrace)?; // { 시작

                    let mut updates = Vec::new();

                    // 블록 내부 반복 처리
                    while let Some(token) = self.current_token() {
                        match token {
                            Token::RightBrace => {
                                self.advance(); // 블록 종료
                                break;
                            }

                            Token::Identifier(field_name) => {
                                let field = field_name.clone();
                                self.advance();

                                self.expect(&Token::Equal)?; // = 기호 확인

                                let expr = self.parse_expression()?; // 우측 표현식 파싱
                                self.expect(&Token::Semicolon)?;

                                updates.push((field, expr));
                            }

                            other => {
                                return Err(format!("update 블록 내에서 예상하지 못한 토큰: {:?}", other));
                            }
                        }
                    }

                    commands.push(Command::Update(updates));
                }

                // 🔸 알 수 없는 토큰 → 에러 처리
                other => {
                    return Err(format!("알 수 없는 명령어 또는 위치에서 토큰 발견: {:?}", other));
                }
            }
        }

        Ok(commands)
    }

    /// 🔹 표현식 파싱 함수
    /// - 우변에서 @필드, "문자열", @필드 + "..." + @필드 등 표현식을 처리
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
                other => {
                    return Err(format!("표현식 내에서 예상하지 못한 토큰: {:?}", other));
                }
            };

            parts.push(expr);

            // + 기호가 있는 경우 계속 이어붙이기
            match self.current_token() {
                Some(Token::Plus) => {
                    self.advance(); // + 소비하고 다음 표현식으로
                }
                _ => break, // 없으면 종료
            }
        }

        // 표현식이 하나면 그대로, 여러 개면 Concat으로 묶기
        if parts.len() == 1 {
            Ok(parts.remove(0))
        } else {
            Ok(Expression::Concat(parts))
        }
    }
}
