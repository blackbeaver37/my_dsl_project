//! ✅ utils.rs
//!
//! DSL에서 사용하는 유틸리티 함수 모음
//! 현재는 문자열 이스케이프 처리만 포함되어 있음

/// 🔧 문자열에서 이스케이프 시퀀스를 실제 문자로 변환
///
/// # 예시
/// - `"Hello\\nWorld"` → `"Hello\nWorld"`
/// - `"Tab:\\tIndent"` → `"Tab:	Indent"`
pub fn unescape_string(s: &str) -> String {
    s.replace("\\\\", "\\")   // 먼저 역슬래시 자체 처리
     .replace("\\n", "\n")    // 개행
     .replace("\\r", "\r")    // 캐리지 리턴
     .replace("\\t", "\t")    // 탭
     .replace("\\\"", "\"")   // 따옴표
}
