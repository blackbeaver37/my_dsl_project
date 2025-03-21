use std::iter::Peekable;
use std::str::Chars;

/// ✅ DSL에서 사용할 모든 토큰 정의
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Input,                      // `input` 키워드
    Output,                     // `output` 키워드
    Update,                     // `update` 키워드
    Print,                      // `print` 키워드

    StringLiteral(String),      // 문자열 (예: "data/sample.jsonl")
    Identifier(String),         // 일반 식별자 (예: line, content, 사용자 정의 변수명)
    Field(String),              // @필드명 (예: @문제, @과목 등)

    Number(usize),              // 숫자 리터럴 (예: 3, 42)

    Plus,                       // `+` 문자열 연결 연산자
    Equal,                      // `=` 대입 연산자
    Semicolon,                  // `;` 명령 구분자
    LeftBrace,                  // `{` 블록 시작
    RightBrace,                 // `}` 블록 종료

    Unknown(char),              // 알 수 없는 문자
    EOF,                        // 입력 종료
}

/// ✅ Lexer 구조체: 입력 문자열을 순차적으로 읽으며 토큰화
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>, // 문자를 하나씩 읽으며, 다음 문자를 미리 볼 수 있는 구조
}

impl<'a> Lexer<'a> {
    /// 🔹 생성자: 새로운 Lexer 인스턴스를 생성
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

    /// 🔹 문자열 리터럴 읽기: "..." 형식
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

    /// 🔹 @필드 읽기 처리
    fn read_field(&mut self) -> Token {
        let mut name = String::new();

        // 첫 글자는 @ 다음 글자여야 하므로 필수
        while let Some(&c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' {
                name.push(self.next_char().unwrap());
            } else {
                break;
            }
        }

        Token::Field(name)
    }

    /// 🔹 숫자 또는 식별자 또는 키워드 처리
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
            "output" => Token::Output,
            "update" => Token::Update,
            "print" => Token::Print,
            _ => {
                // 숫자일 경우
                if let Ok(num) = value.parse::<usize>() {
                    Token::Number(num)
                } else {
                    Token::Identifier(value)
                }
            }
        }
    }

    /// 🔹 다음 토큰 하나 생성
    pub fn next_token(&mut self) -> Token {
        while let Some(c) = self.next_char() {
            match c {
                '"' => return self.read_string(),
                '@' => return self.read_field(),
                '+' => return Token::Plus,
                '=' => return Token::Equal,
                ';' => return Token::Semicolon,
                '{' => return Token::LeftBrace,
                '}' => return Token::RightBrace,

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
