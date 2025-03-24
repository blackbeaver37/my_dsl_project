mod lexer;
mod parser;
mod interpreter;
mod evaluator; // ✅ evaluator 모듈 포함

use lexer::Lexer;
use parser::{Parser, Command};
use interpreter::Interpreter;

use std::fs;

fn main() {
    // ✅ 1. 테스트용 DSL 코드 읽기
    let source_path = "test/test_script.jdl";

    let source = fs::read_to_string(source_path)
        .unwrap_or_else(|e| {
            eprintln!("❌ Failed to read DSL file '{}': {}", source_path, e);
            std::process::exit(1);
        });

    println!("🔹 DSL Script Loaded From '{}':\n", source_path);
    println!("{}", source);
    println!();

    // ✅ 2. 렉서 실행 (토큰화)
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    println!("🔹 Tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  [{:02}] {:?}", i, token);
    }
    println!();

    // ✅ 3. 파서 실행 (AST 생성)
    let mut parser = Parser::new(tokens);
    let commands = match parser.parse() {
        Ok(cmds) => cmds,
        Err(e) => {
            eprintln!("❌ Parser error: {}", e);
            std::process::exit(1);
        }
    };

    println!("🔹 Parsed Commands:");
    for (i, cmd) in commands.iter().enumerate() {
        println!("  [{:02}] {:?}", i, cmd);
    }
    println!();

    // ✅ 4. 인터프리터 실행 (명령어 실행)
    println!("🔹 Interpreter Output:");
    let mut interpreter = Interpreter::new();
    if let Err(e) = interpreter.run(commands) {
        eprintln!("❌ Runtime error: {}", e);
        std::process::exit(1);
    }
}
