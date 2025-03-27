//! ✅ evaluator.rs
//!
//! 이 모듈은 transform 명령어의 우변에 등장하는 Expression을 실제 JSON 값으로 평가한다.
//! - 문자열 기반 표현식은 Value::String("...") 형태로 반환
//! - raw()는 JSON 객체 그대로 Value::Object(...)로 반환

use crate::parser::{Expression, FieldWithModifiers, FieldModifier};
use crate::utils::unescape_string;
use indexmap::IndexMap;
use serde_json::{Value, Map};

/// 🔍 표현식을 평가하여 JSON 값(Value)으로 변환
///
/// # 인자
/// - `expr`: 평가할 Expression
/// - `record`: 현재 처리 중인 한 줄의 JSONL 데이터
///
/// # 반환
/// - Ok(Value): 평가 결과 (문자열 또는 객체)
/// - Err(String): 에러 메시지
pub fn evaluate_expression(
    expr: &Expression,
    record: &IndexMap<String, Value>,
) -> Result<Value, String> {
    match expr {
        // 📌 문자열 리터럴
        Expression::Literal(s) => Ok(Value::String(unescape_string(s))),

        // 📌 일반 필드 (@필드)
        Expression::Field(name) => {
            let value = evaluate_field_with_modifiers(
                &FieldWithModifiers {
                    name: name.clone(),
                    modifiers: vec![],
                },
                record,
            )?;
            Ok(Value::String(value))
        }

        // 📌 확장 필드 (@필드.suffix(...) 등)
        Expression::FieldWithModifiers(field_struct) => {
            let value = evaluate_field_with_modifiers(field_struct, record)?;
            Ok(Value::String(value))
        }

        // 📌 여러 표현식을 +로 연결 (문자열 연결)
        Expression::Concat(parts) => {
            let mut result = String::new();
            for part in parts {
                let v = evaluate_expression(part, record)?;
                let s = v.as_str().unwrap_or("").to_string();
                result.push_str(&s);
            }
            Ok(Value::String(result))
        }

        // ✅ raw() → 전체 레코드 반환
        Expression::RawRecord => {
            let map: Map<String, Value> = record.clone().into_iter().collect(); // 🔹 변환
            Ok(Value::Object(map))
        }
    }
}

/// 🔍 확장 필드(FieldWithModifiers) 평가
///
/// - 필드가 없거나 비어 있으면 default 처리
/// - 값이 존재하면 prefix/suffix 적용
fn evaluate_field_with_modifiers(
    field: &FieldWithModifiers,
    record: &IndexMap<String, Value>,
) -> Result<String, String> {
    // 🔸 원본 필드 값 가져오기
    let mut raw_value: Option<String> = match record.get(&field.name) {
        Some(Value::String(s)) => Some(s.clone()),
        Some(other) => Some(other.to_string()),
        None => None,
    };

    // 🔸 default 우선 적용
    for modifier in &field.modifiers {
        if let FieldModifier::Default(default_str) = modifier {
            if raw_value.is_none() || raw_value.as_deref() == Some("") {
                raw_value = Some(unescape_string(default_str));
            }
        }
    }

    // 🔸 값이 없으면 빈 문자열 반환
    let Some(mut value) = raw_value else {
        return Ok(String::new());
    };

    if value.is_empty() {
        return Ok(String::new());
    }

    // 🔸 prefix → suffix 순서로 적용
    for modifier in &field.modifiers {
        match modifier {
            FieldModifier::Prefix(pre) => {
                value = format!("{}{}", unescape_string(pre), value);
            }
            FieldModifier::Suffix(suf) => {
                value = format!("{}{}", value, unescape_string(suf));
            }
            FieldModifier::Default(_) => {}
        }
    }

    Ok(value)
}
