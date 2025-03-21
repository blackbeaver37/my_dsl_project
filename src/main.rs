mod lexer;
mod parser;
mod interpreter;

use lexer::Lexer;
use parser::Parser;
use interpreter::Interpreter;

use std::fs;

fn main() {
    // ✅ 1. JDL 파일 읽기
    let source = fs::read_to_string("test/test_script.jdl")
        .expect("📂 JDL 스크립트 파일을 읽을 수 없습니다.");

    println!("📜 🔹 원본 소스:\n{}\n", source.trim());

    // ✅ 2. 렉싱 (토큰화)
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    println!("🧱 🔹 토큰(Token) 리스트:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  [{}] {:?}", i, token);
    }
    println!();

    // ✅ 3. 파싱 (AST 생성)
    let mut parser = Parser::new(tokens);
    let commands = match parser.parse() {
        Ok(cmds) => cmds,
        Err(e) => {
            eprintln!("❌ 파싱 실패: {}", e);
            return;
        }
    };

    println!("🧠 🔹 파싱된 명령(Command) 리스트:");
    for (i, command) in commands.iter().enumerate() {
        println!("  [{}] {:?}", i, command);
    }
    println!();

    // ✅ 4. Interpreter 실행
    println!("🚀 🔹 Interpreter 실행 결과:");
    let mut interpreter = Interpreter::new();
    if let Err(e) = interpreter.run(commands) {
        eprintln!("❌ 실행 중 오류 발생: {}", e);
    }
}
