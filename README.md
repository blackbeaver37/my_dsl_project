# 🧩 MyDSL - A Simple JSONL Transformation DSL

**MyDSL**은 JSONL 파일을 손쉽게 처리하고 변환하기 위해 설계된 **간결한 DSL 언어**입니다.  
복잡한 파이썬 코드 없이도, 사람이 읽기 쉬운 문법으로 데이터를 가공할 수 있습니다.

---

## ✨ 주요 기능

- `input`, `output`, `transform` 구문을 통해 JSONL 파일 입출력 및 변환 가능
- `@필드명` 으로 JSON 필드 접근
- `.prefix("...")`, `.suffix("...")`, `.default("...")` 로 텍스트 가공
- `serial()` 함수로 고유 ID 생성
- `raw()` 함수로 전체 레코드 출력
- `+` 연산자를 통한 문자열 연결
- 중첩 필드 접근 지원 (`@meta.score` 등)

---

## 🔧 설치 방법

### 1. 실행 파일 다운로드

> GitHub Release 페이지에서 운영체제에 맞는 실행 파일을 다운로드하세요.

[▶ Windows용 실행 파일 (mydsl.exe)](https://github.com/blackbeaver37/my_dsl_project/releases/tag/v0.1.0)

### 2. PATH에 추가 (선택 사항)

압축을 푼 디렉토리를 시스템 PATH에 추가하면 전역에서 `mydsl` 명령어 사용이 가능합니다.

---

## 🚀 사용 예시

### DSL 파일 예시 (`script.jdl`)

```jdl
input "data/input.jsonl";
output "data/output.jsonl";

transform {
    id = serial();
    data_id = @번호;
    content = @문제.prefix("문제: ").default("없음") + @정답.prefix("\n정답: ");
    meta = raw();
}

print line 1;
```

### 실행

```bash
mydsl script.jdl
```

---

## 📁 프로젝트 구조

```bash
my_dsl_project/
├── src/
│ ├── lexer.rs # 렉서 - 토큰화
│ ├── parser.rs # 파서 - AST 생성
│ ├── evaluator.rs # 표현식 평가
│ ├── interpreter.rs # DSL 실행
│ ├── utils.rs # 유틸 함수
│ └── main.rs # CLI 엔트리포인트
├── test/
│ └── script.jdl # DSL 예제
├── Cargo.toml
```

---

## 📜 라이선스

MIT License  
Built with Rust
