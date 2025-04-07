//! ✅ DSL용 Lexer
//!
//! 이 모듈은 사용자 정의 DSL 스크립트를 의미 있는 Token으로 분해하는 역할을 한다.
//! - 예: input, output, transform, print 등의 키워드
//! - 문자열, 필드(@key), 연산자, 중괄호, 함수 호출 등 처리

use std::iter::Peekable;
use std::str::Chars;

/// ✅ DSL에서 사용할 토큰 정의
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // 🔹 키워드
    Input,
    Output,
    Transform,
    Print,

    // 🔹 리터럴 / 참조
    StringLiteral(String),   // 예: "data.jsonl"
    Identifier(String),      // 예: suffix, line
    Field(String),           // 예: @문제
    Number(usize),           // 예: 42

    // 🔹 연산자 및 구분자
    Plus,                    // +
    Equal,                   // =
    Semicolon,              // ;
    LBrace, RBrace,         // {, }
    Dot,                    // .
    LParen, RParen,         // (, )

    // 🔹 주석
    Comment(String),        // // 또는 /* */ 주석

    // 🔹 예외
    Unknown(char),          // 알 수 없는 문자
    EOF,                    // 입력 종료
}

/// ✅ 입력 문자열을 순회하며 Token을 생성하는 구조체
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    /// 🔹 Lexer 생성자
    pub fn new(source: &'a str) -> Self {
        Self {
            input: source.chars().peekable(),
        }
    }

    /// 🔹 문자 하나 읽기 (consume)
    fn next_char(&mut self) -> Option<char> {
        self.input.next()
    }

    /// 🔹 다음 문자 미리보기 (peek)
    fn peek_char(&mut self) -> Option<&char> {
        self.input.peek()
    }

    /// 🔹 문자열 리터럴 파싱 (예: "...")
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

    /// 🔹 @필드 처리 (예: @문제)
    fn read_field(&mut self) -> Token {
        let mut name = String::new();

        while let Some(&c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' {
                name.push(self.next_char().unwrap());
            } else {
                break;
            }
        }

        Token::Field(name)
    }

    /// 🔹 식별자 / 숫자 / 키워드 파싱
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

        match value.as_str() {
            "input" => Token::Input,
            "output" => Token::Output,
            "transform" => Token::Transform,
            "print" => Token::Print,
            _ => {
                if let Ok(num) = value.parse::<usize>() {
                    Token::Number(num)
                } else {
                    Token::Identifier(value)
                }
            }
        }
    }

    /// 🔹 라인 주석 파싱 (예: // ...)
    fn read_line_comment(&mut self) -> Token {
        let mut result = String::new();

        while let Some(&c) = self.peek_char() {
            if c == '\n' {
                break;
            }
            result.push(self.next_char().unwrap());
        }

        Token::Comment(result.trim().to_string())
    }

    /// 🔹 블록 주석 파싱 (예: /* ... */)
    fn read_block_comment(&mut self) -> Token {
        let mut result = String::new();

        while let Some(c) = self.next_char() {
            if c == '*' {
                if let Some(&'/') = self.peek_char() {
                    self.next_char(); // consume '/'
                    break;
                }
            }
            result.push(c);
        }

        Token::Comment(result.trim().to_string())
    }

    /// 🔹 입력에서 토큰 하나 파싱
    pub fn next_token(&mut self) -> Token {
        while let Some(c) = self.next_char() {
            match c {
                '/' => {
                    if let Some(&'/') = self.peek_char() {
                        self.next_char(); // consume second '/'
                        return self.read_line_comment();
                    } else if let Some(&'*') = self.peek_char() {
                        self.next_char(); // consume '*'
                        return self.read_block_comment();
                    } else {
                        return Token::Unknown(c);
                    }
                }

                '"' => return self.read_string(),
                '@' => return self.read_field(),
                '+' => return Token::Plus,
                '=' => return Token::Equal,
                ';' => return Token::Semicolon,
                '{' => return Token::LBrace,
                '}' => return Token::RBrace,
                '.' => return Token::Dot,
                '(' => return Token::LParen,
                ')' => return Token::RParen,
                c if c.is_whitespace() => continue,
                c if c.is_alphanumeric() => return self.read_identifier_or_number(c),
                other => return Token::Unknown(other),
            }
        }

        Token::EOF
    }

    /// 🔹 전체 입력을 토큰 리스트로 변환
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
