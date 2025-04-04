//! ✅ main.rs
//!
//! DSL 실행기의 진입점 (Command Line Interface)
//! 사용 예시:
//!     $ mydsl script.jdl

mod lexer;
mod parser;
mod evaluator;
mod interpreter;
mod utils;

use lexer::Lexer;
use parser::Parser;
use interpreter::Interpreter;

use std::env;
use std::fs;

/// ✅ 디버그 출력용 전역 플래그
const DEBUG: bool = false;

fn main() {
    // 🔹 명령줄 인자 확인: mydsl <파일명>
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("❌ Usage: mydsl <script.jdl>");
        std::process::exit(1);
    }

    let source_path = &args[1];

    // 🔹 DSL 파일 읽기
    let source = fs::read_to_string(source_path).unwrap_or_else(|e| {
        eprintln!("❌ Failed to read DSL file '{}': {}", source_path, e);
        std::process::exit(1);
    });

    if DEBUG {
        println!("🔹 DSL Script Loaded From '{}':\n", source_path);
        println!("{}", source);
        println!();
    }

    // 🔹 렉싱: 소스 → 토큰 리스트
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    if DEBUG {
        println!("🔹 Tokens:");
        for (i, token) in tokens.iter().enumerate() {
            println!("  [{:02}] {:?}", i, token);
        }
        println!();
    }

    // 🔹 파싱: 토큰 리스트 → 명령어 리스트
    let mut parser = Parser::new(tokens);
    let commands = match parser.parse() {
        Ok(cmds) => cmds,
        Err(e) => {
            eprintln!("❌ Parser error: {}", e);
            std::process::exit(1);
        }
    };

    if DEBUG {
        println!("🔹 Parsed Commands:");
        for (i, cmd) in commands.iter().enumerate() {
            println!("  [{:02}] {:?}", i, cmd);
        }
        println!();
    }

    // 🔹 실행: 명령어 리스트 실행
    if DEBUG {
        println!("🔹 Interpreter Output:");
    }

    let mut interpreter = Interpreter::new();
    if let Err(e) = interpreter.run(commands) {
        eprintln!("❌ Runtime error: {}", e);
        std::process::exit(1);
    }
}
