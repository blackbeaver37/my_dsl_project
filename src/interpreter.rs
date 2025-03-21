use crate::parser::Command;
use std::fs::File;
use std::io::{BufRead, BufReader};

// ✅ JSON 순서 유지 위해 IndexMap 사용
use serde_json::Value;

/// ✅ Interpreter 구조체
/// - 실행 중 필요한 상태(예: 로드한 데이터)를 저장
pub struct Interpreter {
    input_file_path: Option<String>, // 현재 로드된 파일 경로
    jsonl_data: Vec<Value>,          // JSONL 파일에서 읽어들인 JSON 객체 리스트
}

impl Interpreter {
    /// 🔹 Interpreter 생성자
    pub fn new() -> Self {
        Self {
            input_file_path: None,
            jsonl_data: Vec::new(),
        }
    }

    /// 🔹 주 실행 함수: 파싱된 명령어(Command) 리스트를 실행
    pub fn run(&mut self, commands: Vec<Command>) -> Result<(), String> {
        for command in commands {
            match command {
                Command::Input(path) => {
                    self.input_file_path = Some(path.clone());
                    self.jsonl_data = Self::read_jsonl_file(&path)?;
                }

                Command::Print => {
                    for value in &self.jsonl_data {
                        // ✅ 순서 유지하며 JSON 한 줄로 출력
                        println!("{}", serde_json::to_string(value).unwrap());
                    }
                }

                Command::PrintLine(line_num) => {
                    if line_num == 0 || line_num > self.jsonl_data.len() {
                        println!("❗ 요청한 라인이 존재하지 않습니다: {}", line_num);
                    } else {
                        let item = &self.jsonl_data[line_num - 1];
                        println!("{}", serde_json::to_string(item).unwrap());
                    }
                }
            }
        }

        Ok(())
    }

    /// 🔹 JSONL 파일을 읽어 Vec<Value>로 반환
    /// - serde_json::from_str()는 내부적으로 IndexMap을 사용하도록 되어 있음
    ///   (Cargo.toml에서 preserve_order feature 활성화 필요)
    fn read_jsonl_file(path: &str) -> Result<Vec<Value>, String> {
        let file = File::open(path).map_err(|e| format!("파일 열기 실패: {}", e))?;
        let reader = BufReader::new(file);

        let mut result = Vec::new();
        for line in reader.lines() {
            let line = line.map_err(|e| format!("파일 읽기 실패: {}", e))?;

            // ✅ JSON 순서 보존하도록 파싱
            let json: Value = serde_json::from_str(&line)
                .map_err(|e| format!("JSON 파싱 실패: {}", e))?;

            result.push(json);
        }

        Ok(result)
    }
}
