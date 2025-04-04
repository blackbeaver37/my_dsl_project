//! ✅ evaluator.rs
//!
//! 이 모듈은 transform 명령어의 우변에 등장하는 Expression을 실제 JSON 값으로 평가한다.
//! - 문자열 기반 표현식은 Value::String("...") 형태로 반환
//! - raw()는 JSON 객체 그대로 Value::Object(...)로 반환
//! - serial()은 1부터 자동으로 증가하는 문자열 숫자

use crate::parser::{Expression, FieldWithModifiers, FieldModifier};
use crate::utils::unescape_string;
use indexmap::IndexMap;
use serde_json::{Value, Map};

/// ✅ serial()을 위한 상태 저장 구조체
#[derive(Default)]
pub struct EvaluatorState {
    pub serial_counter: usize,
}

impl EvaluatorState {
    pub fn new() -> Self {
        Self { serial_counter: 1 }
    }
}

/// 🔍 표현식을 평가하여 JSON Value로 변환
///
/// # Params
/// - `expr`: 파싱된 Expression
/// - `record`: 한 줄의 JSONL 데이터 (IndexMap<String, Value>)
/// - `state`: serial 카운터를 위한 상태 구조체
pub fn evaluate_expression(
    expr: &Expression,
    record: &IndexMap<String, Value>,
    state: &mut EvaluatorState,
) -> Result<Value, String> {
    match expr {
        // 📌 문자열 리터럴
        Expression::Literal(s) => Ok(Value::String(unescape_string(s))),

        // 📌 일반 필드 (@meta.score 등)
        Expression::FieldPath(path) => {
            let value = get_nested_value_as_string(record, path);
            Ok(Value::String(value.unwrap_or_default()))
        }

        // 📌 필드 + 수정자 (prefix, suffix, default)
        Expression::FieldWithModifiers(field_struct) => {
            let value = evaluate_field_with_modifiers(field_struct, record)?;
            Ok(Value::String(value))
        }

        // 📌 여러 표현식 연결
        Expression::Concat(parts) => {
            let mut result = String::new();
            for part in parts {
                let v = evaluate_expression(part, record, state)?;
                let s = v.as_str().unwrap_or("").to_string();
                result.push_str(&s);
            }
            Ok(Value::String(result))
        }

        // ✅ raw() → 전체 객체 반환
        Expression::RawRecord => {
            let map: Map<String, Value> = record.clone().into_iter().collect();
            Ok(Value::Object(map))
        }

        // ✅ serial() → 자동 증가 문자열 반환
        Expression::Serial => {
            let result = state.serial_counter.to_string();
            state.serial_counter += 1;
            Ok(Value::String(result))
        }
    }
}

/// 🔍 FieldWithModifiers 를 평가하여 문자열로 반환
fn evaluate_field_with_modifiers(
    field: &FieldWithModifiers,
    record: &IndexMap<String, Value>,
) -> Result<String, String> {
    // 경로 따라 실제 값 가져오기
    let mut raw_value: Option<String> = get_nested_value_as_string(record, &field.path);

    // 1️⃣ default() 우선 적용
    for modifier in &field.modifiers {
        if let FieldModifier::Default(default_str) = modifier {
            if raw_value.is_none() || raw_value.as_deref() == Some("") {
                raw_value = Some(unescape_string(default_str));
            }
        }
    }

    let Some(mut value) = raw_value else {
        return Ok(String::new());
    };

    if value.is_empty() {
        return Ok(String::new());
    }

    // 2️⃣ prefix/suffix 적용
    for modifier in &field.modifiers {
        match modifier {
            FieldModifier::Prefix(pre) => {
                value = format!("{}{}", unescape_string(pre), value);
            }
            FieldModifier::Suffix(suf) => {
                value = format!("{}{}", value, unescape_string(suf));
            }
            FieldModifier::Default(_) => {} // 이미 위에서 처리
        }
    }

    Ok(value)
}

/// 🔍 중첩 경로 (["a", "b", "c"]) 에 따라 값을 가져옴
fn get_nested_value_as_string(
    record: &IndexMap<String, Value>,
    path: &[String],
) -> Option<String> {
    let mut current: &Value = record.get(&path[0])?;

    for key in &path[1..] {
        match current {
            Value::Object(map) => {
                current = map.get(key)?;
            }
            _ => return None,
        }
    }

    match current {
        Value::String(s) => Some(s.clone()),
        other => Some(other.to_string()),
    }
}
