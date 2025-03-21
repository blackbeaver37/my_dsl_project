use std::iter::Peekable;
use std::str::Chars;

/// ✅ 확장된 DSL 토큰 정의
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Input,                      // input 키워드
    Print,                      // print 키워드
    StringLiteral(String),      // 문자열 (예: "data/sample.jsonl")
    Identifier(String),         // 일반 식별자 (예: line, content)
    Number(usize),              // 숫자 리터럴 (예: 3, 42)
    Semicolon,                  // ;
    Unknown(char),              // 알 수 없는 문자
    EOF,                        // 입력 종료
}

/// ✅ Lexer 구조체
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            input: source.chars().peekable(),
        }
    }

    fn next_char(&mut self) -> Option<char> {
        self.input.next()
    }

    fn peek_char(&mut self) -> Option<&char> {
        self.input.peek()
    }

    fn read_string(&mut self) -> Token {
        let mut result = String::new();
        while let Some(c) = self.next_char() {
            if c == '"' {
                break;
            }
            result.push(c);
        }
        Token::StringLiteral(result)
    }

    /// ✅ 숫자 또는 식별자 또는 키워드 처리
    fn read_identifier_or_number(&mut self, first_char: char) -> Token {
        let mut value = String::new();
        value.push(first_char);

        while let Some(&c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' {
                value.push(self.next_char().unwrap());
            } else {
                break;
            }
        }

        // 키워드 우선 처리
        match value.as_str() {
            "input" => Token::Input,
            "print" => Token::Print,
            _ => {
                // 숫자라면 숫자로 처리
                if let Ok(num) = value.parse::<usize>() {
                    Token::Number(num)
                } else {
                    Token::Identifier(value)
                }
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        while let Some(c) = self.next_char() {
            match c {
                ';' => return Token::Semicolon,
                '"' => return self.read_string(),
                c if c.is_whitespace() => continue,
                c if c.is_alphanumeric() => return self.read_identifier_or_number(c),
                other => return Token::Unknown(other),
            }
        }

        Token::EOF
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token == Token::EOF {
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}
