//! ✅ evaluator.rs
//!
//! 이 모듈은 transform 명령어의 우변에 등장하는 Expression을 실제 문자열 값으로 변환하는 역할을 한다.
//! - 기본 표현식: @필드명, "문자열", @필드 + "_" + @필드
//! - 확장 표현식: @필드.suffix("_").default("기본값")

use crate::parser::{Expression, FieldWithModifiers, FieldModifier};
use crate::utils::unescape_string;
use indexmap::IndexMap;
use serde_json::Value;

/// 🔍 표현식을 평가하여 문자열로 변환
///
/// # 인자
/// - `expr`: 평가할 Expression (Field, Literal, Concat 등)
/// - `record`: 현재 처리 중인 한 줄의 JSONL 데이터
///
/// # 반환
/// - Ok(String): 변환된 문자열
/// - Err(String): 에러 메시지
pub fn evaluate_expression(
    expr: &Expression,
    record: &IndexMap<String, Value>,
) -> Result<String, String> {
    match expr {
        // 📌 문자열 리터럴은 그대로 반환
        Expression::Literal(s) => Ok(unescape_string(s)),

        // 📌 일반 필드 (@필드)
        Expression::Field(name) => {
            evaluate_field_with_modifiers(
                &FieldWithModifiers {
                    name: name.clone(),
                    modifiers: vec![],
                },
                record,
            )
        }

        // 📌 확장 필드 (@필드.suffix(...).default(...) 등)
        Expression::FieldWithModifiers(field_struct) => {
            evaluate_field_with_modifiers(field_struct, record)
        }

        // 📌 여러 표현식을 +로 연결한 경우 (예: @과목 + "_" + @학년)
        Expression::Concat(parts) => {
            let mut result = String::new();
            for part in parts {
                let v = evaluate_expression(part, record)?; // 하위 표현식 재귀적으로 평가
                result.push_str(&v);
            }
            Ok(result)
        }
    }
}

/// 🔍 확장 필드(FieldWithModifiers) 평가
///
/// - 필드가 존재하지 않거나 값이 비어 있을 경우 default 처리
/// - 값이 존재하면 prefix, suffix 순서대로 modifier를 적용
///
/// # modifier 처리 순서
/// 1. Default: 값이 없을 경우 기본값으로 대체
/// 2. Prefix: 앞에 문자열 추가
/// 3. Suffix: 뒤에 문자열 추가
fn evaluate_field_with_modifiers(
    field: &FieldWithModifiers,
    record: &IndexMap<String, Value>,
) -> Result<String, String> {
    // 🔸 필드의 원본 값 가져오기
    let mut raw_value: Option<String> = match record.get(&field.name) {
        Some(Value::String(s)) => Some(s.clone()),
        Some(other) => Some(other.to_string()), // 숫자나 불리언도 문자열로 변환
        None => None,
    };

    // 🔸 default() modifier 우선 적용
    for modifier in &field.modifiers {
        if let FieldModifier::Default(default_str) = modifier {
            if raw_value.is_none() || raw_value.as_deref() == Some("") {
                raw_value = Some(unescape_string(default_str));
            }
        }
    }

    // 🔸 값이 없는 경우 빈 문자열 반환 (에러는 아님)
    let Some(mut value) = raw_value else {
        return Ok(String::new());
    };

    // 🔸 빈 문자열이면 prefix/suffix 적용하지 않음
    if value.is_empty() {
        return Ok(String::new());
    }

    // 🔸 prefix → suffix 순서로 modifier 적용
    for modifier in &field.modifiers {
        match modifier {
            FieldModifier::Prefix(pre) => {
                value = format!("{}{}", unescape_string(pre), value);
            }
            FieldModifier::Suffix(suf) => {
                value = format!("{}{}", value, unescape_string(suf));
            }
            FieldModifier::Default(_) => {
                // default는 앞에서 이미 처리됨
            }
        }
    }

    Ok(value)
}
