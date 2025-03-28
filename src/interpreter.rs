use crate::parser::Command;
use crate::evaluator::{evaluate_expression, EvaluatorState}; // 🔹 EvaluatorState 포함

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

use serde_json::Value;
use indexmap::IndexMap;

/// ✅ Interpreter 구조체
/// - DSL 명령어(Command)를 해석하고 실행
pub struct Interpreter {
    input_file_path: Option<String>,                 // 입력 파일 경로
    output_file_path: Option<String>,                // 출력 파일 경로
    jsonl_data: Vec<IndexMap<String, Value>>,        // 입력 JSONL 데이터
    transformed_data: Vec<IndexMap<String, Value>>,  // transform 결과 데이터
}

impl Interpreter {
    /// 🔹 Interpreter 인스턴스 생성
    pub fn new() -> Self {
        Self {
            input_file_path: None,
            output_file_path: None,
            jsonl_data: Vec::new(),
            transformed_data: Vec::new(),
        }
    }

    /// 🔹 DSL 명령어(Command) 실행
    pub fn run(&mut self, commands: Vec<Command>) -> Result<(), String> {
        // ✅ serial() 처리를 위한 evaluator 상태 초기화
        let mut eval_state = EvaluatorState::new();

        for command in commands {
            match command {
                // 📌 input "파일명";
                Command::Input(path) => {
                    self.input_file_path = Some(path.clone());
                    self.jsonl_data = Self::read_jsonl_file(&path)?;
                }

                // 📌 output "파일명";
                Command::Output(path) => {
                    self.output_file_path = Some(path.clone());
                }

                // 📌 print;
                Command::Print => {
                    for value in &self.jsonl_data {
                        println!("{}", serde_json::to_string(value).unwrap());
                    }
                }

                // 📌 print line N;
                Command::PrintLine(line_num) => {
                    if line_num == 0 || line_num > self.jsonl_data.len() {
                        println!("⚠️ Line number {} is out of range.", line_num);
                    } else {
                        let item = &self.jsonl_data[line_num - 1];
                        println!("{}", serde_json::to_string(item).unwrap());
                    }
                }

                // 📌 transform { ... }
                Command::Transform(assignments) => {
                    self.transformed_data.clear();

                    for original in &self.jsonl_data {
                        let mut new_record = IndexMap::new();

                        for (field_name, expr) in &assignments {
                            let value = evaluate_expression(expr, original, &mut eval_state)?; // ✅ 상태 전달
                            new_record.insert(field_name.clone(), value);
                        }

                        self.transformed_data.push(new_record);
                    }
                }
            }
        }

        // 🔹 결과 저장
        if let Some(path) = &self.output_file_path {
            let data = if !self.transformed_data.is_empty() {
                &self.transformed_data
            } else {
                &self.jsonl_data
            };

            Self::save_to_output_file(path, data)?;
        }

        Ok(())
    }

    fn read_jsonl_file(path: &str) -> Result<Vec<IndexMap<String, Value>>, String> {
        let file = File::open(path)
            .map_err(|e| format!("Failed to open file '{}': {}", path, e))?;
        let reader = BufReader::new(file);

        let mut result = Vec::new();
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            let json_map: IndexMap<String, Value> = serde_json::from_str(&line)
                .map_err(|e| format!("JSON parsing error: {}", e))?;
            result.push(json_map);
        }

        Ok(result)
    }

    fn save_to_output_file(
        path: &str,
        data: &Vec<IndexMap<String, Value>>,
    ) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(|e| format!("Failed to open output file '{}': {}", path, e))?;

        for record in data {
            let line = serde_json::to_string(record)
                .map_err(|e| format!("Failed to serialize record: {}", e))?;
            writeln!(file, "{}", line)
                .map_err(|e| format!("Failed to write to output file: {}", e))?;
        }

        println!("✅ Output saved to '{}'", path);
        Ok(())
    }
}
