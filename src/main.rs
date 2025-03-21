mod lexer;
mod parser;

use lexer::Lexer;
use parser::{Parser, Command, Expression};

use std::fs;

fn main() {
    // ✅ 1. DSL 스크립트 읽기
    let source = fs::read_to_string("test/test_script.jdl")
        .expect("❌ JDL 스크립트 파일을 읽을 수 없습니다.");

    println!("📜 🔹 원본 DSL 스크립트:\n{}\n", source.trim());

    // ✅ 2. Lexer: 토큰화
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    println!("🧱 🔹 생성된 토큰 리스트:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  [{:02}] {:?}", i, token);
    }
    println!();

    // ✅ 3. Parser: AST(Command 리스트) 생성
    let mut parser = Parser::new(tokens);
    let commands = match parser.parse() {
        Ok(cmds) => cmds,
        Err(e) => {
            eprintln!("❌ 파싱 에러 발생: {}", e);
            return;
        }
    };

    println!("🧠 🔹 파싱된 명령어(Command) 리스트:");
    for (i, command) in commands.iter().enumerate() {
        println!("  [{:02}] {:?}", i, command);
    }
    println!();
}
