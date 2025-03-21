use crate::lexer::{Token};  // lexer 모듈에서 Token 가져오기

/// ✅ DSL 명령어의 구조를 표현하는 AST 노드 (Command = 추상 명령)
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Input(String), // input "파일경로"
    Print,         // print;
    PrintLine(usize), // 추가: 특정 라인만 출력
    // 향후 확장을 위한 여지: Output(String), Map {...}, 등
}

/// ✅ Parser 구조체
/// - 토큰 벡터를 받아서 위치를 추적하며 구문 분석
pub struct Parser {
    tokens: Vec<Token>, // 분석 대상 토큰 목록
    position: usize,    // 현재 분석 중인 위치
}

impl Parser {
    /// 🔹 Parser 생성자
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// 🔹 현재 토큰 가져오기 (Option으로 반환)
    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// 🔹 다음 토큰으로 이동
    fn advance(&mut self) {
        self.position += 1;
    }

    /// 🔹 현재 토큰이 예상한 토큰인지 검사하고 통과하면 advance
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

    /// 🔹 파싱 시작: 전체 토큰을 읽고 Command 리스트로 반환
    pub fn parse(&mut self) -> Result<Vec<Command>, String> {
        let mut commands = Vec::new();

        while let Some(token) = self.current_token() {
            match token {
                Token::Input => {
                    self.advance(); // input 소비

                    // 다음은 문자열 리터럴이어야 함
                    match self.current_token() {
                        Some(Token::StringLiteral(path)) => {
                            let path = path.clone(); // 소유권 이전을 위해 복사
                            self.advance();

                            self.expect(&Token::Semicolon)?; // 세미콜론 확인

                            commands.push(Command::Input(path));
                        }
                        other => return Err(format!("input 명령어 다음에는 문자열이 와야 합니다. 현재: {:?}", other)),
                    }
                }

                Token::Print => {
                    self.advance(); // print 소비
                
                    match self.current_token() {
                        Some(Token::Semicolon) => {
                            self.advance();
                            commands.push(Command::Print);
                        }
                        Some(Token::Identifier(id)) if id == "line" => {
                            self.advance(); // line 소비
                
                            match self.current_token() {
                                Some(Token::Number(n)) => {
                                    let line_num = *n; // 여기서 n은 &usize → *n으로 가져오기
                                    self.advance();
                                    self.expect(&Token::Semicolon)?;
                                    commands.push(Command::PrintLine(line_num));
                                }
                                other => {
                                    return Err(format!("print line 다음에는 숫자가 와야 합니다: {:?}", other));
                                }
                            }
                        }
                        other => return Err(format!("print 뒤에 잘못된 토큰: {:?}", other)),
                    }
                }
                other => {
                    return Err(format!("알 수 없는 명령어 또는 위치에서 토큰 발견: {:?}", other));
                }
            }
        }

        Ok(commands)
    }
}


#[cfg(test)]
mod tests {
    use super::*; // parser.rs 내 구조 사용
    use crate::lexer::{Lexer, Token}; // lexer 사용

    #[test]
    fn test_parser_input_and_print() {
        let source = r#"
            input "file.jsonl";
            print;
        "#;

        // 1. Lexer로 토큰화
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        // 2. Parser로 AST 만들기
        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        // 3. 기대 결과 구성
        let expected = vec![
            Command::Input("file.jsonl".to_string()),
            Command::Print,
        ];

        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_parser_missing_semicolon() {
        let source = r#"
            input "file.jsonl"
            print;
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        assert!(result.is_err()); // 세미콜론 빠졌으므로 에러가 나야 함
    }

    #[test]
    fn test_parser_invalid_input_argument() {
        let source = r#"
            input print;
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        assert!(result.is_err()); // input 뒤에 문자열이 아니라 print가 오면 에러
    }
}
