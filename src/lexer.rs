//! ✅ DSL용 Lexer
//!
//! 이 모듈은 사용자 정의 DSL 스크립트를 의미 있는 Token으로 분해하는 역할을 한다.
//! 예: input/output/transform/print 등의 키워드, 문자열, 식별자, 연산자 등을 인식한다.

use std::iter::Peekable;
use std::str::Chars;

/// ✅ DSL에서 사용할 모든 토큰 정의
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // 키워드들
    Input, Output, Transform, Print,

    // 값 또는 참조
    StringLiteral(String),      // 예: "sample.jsonl"
    Identifier(String),         // 예: line, suffix 등
    Field(String),              // 예: @문제

    Number(usize),              // 예: 42

    // 연산자 및 구분자
    Plus,                       // +
    Equal,                      // =
    Semicolon,                  // ;
    LBrace,                  // {
    RBrace,                 // }
    Dot,                        // . (함수 호출 구분자)
    LParen,                     // ( (함수 호출 시작)
    RParen,                     // ) (함수 호출 종료)

    // 예외 및 종료
    Unknown(char),              // 정의되지 않은 문자
    EOF,                        // 입력 종료
}


/// ✅ Lexer 구조체
/// 입력 문자열을 한 글자씩 순회하며 Token을 생성함
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>, // Peekable로 앞 글자 확인 가능하게 처리
}

impl<'a> Lexer<'a> {
    /// 🔹 생성자
    pub fn new(source: &'a str) -> Self {
        Self {
            input: source.chars().peekable(),
        }
    }

    /// 🔹 다음 문자 반환 (consume)
    fn next_char(&mut self) -> Option<char> {
        self.input.next()
    }

    /// 🔹 다음 문자를 미리 보기 (소비하지 않음)
    fn peek_char(&mut self) -> Option<&char> {
        self.input.peek()
    }

    /// 🔹 문자열 리터럴 읽기: "..."
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

    /// 🔹 @필드명 처리: @이후의 식별자 추출
    fn read_field(&mut self) -> Token {
        let mut name = String::new();

        while let Some(&c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' {
                name.push(self.next_char().expect("Lexer error: failed to read character after '@'"));
            } else {
                break;
            }
        }

        Token::Field(name)
    }

    /// 🔹 식별자 또는 숫자 또는 키워드 판별
    fn read_identifier_or_number(&mut self, first_char: char) -> Token {
        let mut value = String::new();
        value.push(first_char);

        while let Some(&c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' {
                value.push(self.next_char().expect("Lexer error: failed to read identifier character"));
            } else {
                break;
            }
        }

        match value.as_str() {
            // 키워드 우선 처리
            "input" => Token::Input,
            "output" => Token::Output,
            "transform" => Token::Transform,
            "print" => Token::Print,
            _ => {
                // 숫자 리터럴 판별
                if let Ok(num) = value.parse::<usize>() {
                    Token::Number(num)
                } else {
                    Token::Identifier(value)
                }
            }
        }
    }

    /// 🔹 입력으로부터 토큰 하나 반환
    pub fn next_token(&mut self) -> Token {
        while let Some(c) = self.next_char() {
            match c {
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

    /// 🔹 전체 입력을 순회하며 토큰 리스트 생성
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
