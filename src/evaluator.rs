//! ✅ evaluator.rs
//!
//! 이 모듈은 transform 명령어의 우변에 등장하는 Expression을 실제 문자열 값으로 변환하는 역할을 한다.
//! 예: @필드명, "문자열", @필드 + "_" + @필드 같은 표현식을 평가할 수 있음.

use crate::parser::Expression;
use indexmap::IndexMap;
use serde_json::Value;

/// 🔍 표현식을 평가하여 문자열로 변환
///
/// # 인자
/// - `expr`: 평가할 Expression (예: Field, Literal, Concat)
/// - `record`: 현재 처리 중인 한 줄의 JSONL 데이터 (key-value 구조)
///
/// # 반환
/// - 성공 시: 평가 결과 문자열
/// - 실패 시: 에러 메시지 (필드 없음 등)
pub fn evaluate_expression(
    expr: &Expression,
    record: &IndexMap<String, Value>,
) -> Result<String, String> {
    match expr {
        // 📌 문자열 리터럴은 그대로 반환
        Expression::Literal(s) => Ok(s.clone()),

        // 📌 @필드 처리: record에서 해당 필드를 찾아서 문자열로 반환
        Expression::Field(field_name) => {
            match record.get(field_name) {
                Some(Value::String(s)) => Ok(s.clone()),        // 문자열 필드
                Some(other_value) => Ok(other_value.to_string()), // 다른 타입은 문자열 변환
                None => Err(format!("Field '{}' not found in record.", field_name)),
            }
        }

        // 📌 여러 표현식을 +로 연결한 경우: Concat 처리
        Expression::Concat(parts) => {
            let mut result = String::new();
            for part in parts {
                let evaluated = evaluate_expression(part, record)?; // 재귀 호출
                result.push_str(&evaluated);
            }
            Ok(result)
        }
    }
}
