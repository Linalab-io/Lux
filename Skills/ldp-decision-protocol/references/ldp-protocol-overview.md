# LDP 5-Step Protocol Overview — Lux Developer Reference

> Lina Decision Protocol (LDP)의 5단계 구조와 각 단계가 Lux Unity 자동화 워크플로우에 어떻게 적용되는지 요약합니다.

## Protocol Architecture

```
                    ┌──────────────────────────┐
                    │     Step0: Socratic      │ (선택적)
                    │  Clarify → Evidence →    │
                    │  Assumption → Risk →     │
                    │  Alternative → Decision  │
                    └──────────┬───────────────┘
                               │
                    ┌──────────▼───────────────┐
                    │  SS1-SS3: ACKNOWLEDGE     │ ← Category 없음
                    │  "이 결정을 이해하고       │
                    │   수행할 책임이 있습니까?"  │
                    └──────────┬───────────────┘
                               │
              ┌────────────────▼────────────────┐
              │  SS4-SS5: NUMBERS               │ ← Category A
              │  "정량적 근거가 충분합니까?"      │
              │  • 비용/수익 모델               │
              │  • 리소스 소모량                │
              │  • 영향 범위 메트릭             │
              └────────────────┬────────────────┘
                               │
              ┌────────────────▼────────────────┐
              │  SS6-SS8: ETHICS                │ ← Category B (Hard Reject)
              │  "윤리적으로 정당화됩니까?"       │
              │  • 사용자 영향                  │
              │  • 데이터 프라이버시            │
              │  • 공정성 / 편향               │
              │  • 투명성                      │
              └────────────────┬────────────────┘
                               │
              ┌────────────────▼────────────────┐
              │  SS9-SS10: TERMINATION          │ ← Category C + D
              │  "중단 조건과 롤백 계획이        │
              │   명확합니까?"                   │
              │  • Risk 경계 (Category C)       │
              │  • Scope 제한 (Category D)      │
              └────────────────┬────────────────┘
                               │
              ┌────────────────▼────────────────┐
              │  SS11-SS12: APPROVAL            │ ← Category E
              │  "승인 절차를 완료했습니까?"     │
              │  • Stakeholder 서명             │
              │  • 위험 이해 확인               │
              │  • 기록 보관                    │
              └─────────────────────────────────┘
```

## Step Details for Lux

### Step0: Socratic (선택적)

복잡한 AI 생성 작업(예:全新的 C# 시스템 생성) 전에 실행합니다.

| 질문 유형 | Lux 맥락 | 예시 |
|-----------|----------|------|
| **Clarify** | 무엇을 자동화하려는가? | "AI Bridge를 통해 어떤 Editor API를 호출할 것인가?" |
| **Evidence** | 근거는 무엇인가? | "기존 컴파일 에러 로그, 프로필링 데이터" |
| **Assumption** | 가정은 무엇인가? | "Unity 2022.3+ 호환, 해당 패키지 설치됨" |
| **Risk** | 위험은 무엇인가? | "생성된 코드가 빌드를 깨뜨림" |
| **Alternative** | 대안은 무엇인가? | "수동 편집, 부분 자동화만" |
| **Decision** | 최종 결정? | "전체 자동화 진행 / 부분 진행 / 중단" |

### Step1: Acknowledge (SS1-SS3)

**Lux 매핑**: `lux unity context` 실행 결과를 바탕으로 현재 상태 인정

- **SS1**: 이 작업의 목표를 명확히 설명할 수 있는가?
- **SS2**: 예상되는 결과와 부작용을 이해하는가?
- **SS3**: 결과에 대한 책임을 받아들일 수 있는가?

**Lux 통합**: `lux unity context --json` 출력에서 project name, unity version, open scenes, compile errors 수 등을 Acknowledge 입력으로 사용.

### Step2: Numbers (SS4-SS5) — Category A

**Lux 매핑**: `lux compile --json`, `lux run-tests --json` 메트릭

- **SS4**: 정량적 데이터(컴파일 시간, 테스트 개수, 파일 변경 수)가 충분한가?
- **SS5**: 성공/실패 기준이 숫자로 정의되어 있는가?

**Lux 메트릭 예시**:
```json
{
  "compile_time_ms": 12400,
  "files_changed": 3,
  "test_count": 47,
  "test_pass_rate": 0.97,
  "asset_impact_count": 0,
  "memory_delta_mb": -12
}
```

### Step3: Ethics (SS6-SS8) — Category B ⚠️ Hard Reject

**가장 중요한 단계**. `No` 응답은 즉시 **Reject** verdict를 강제합니다.

- **SS6**: 이 작업이 최종 사용자(플레이어)에게 부정적인 영향을 주는가?
- **SS7**: 개인정보 또는 민감한 데이터를 다루는가?
- **SS8**: 이 결정은 투명하게 문서화되고 설명 가능한가?

**Lux 윤리 체크포인트**:
- 🎮 AI가 생성한 코드가 gacha/확률형 메카닉을 조작하는가?
- 🔒 PlayMode 테스트에서 실제 사용자 데이터에 접근하는가?
- 📝 생성된 asset에 proper attribution이 있는가?
- 🤖 AI 결정 과정이 추적 가능한가?

### Step4: Termination (SS9-SS10) — Category C + D

**Lux 매핑**: 실패 시 rollback 계획

- **SS9 (Risk/C)**: 언제 이 작업을 중단해야 하는가? (경계 조건)
- **SS10 (Scope/D)**: 작업 범위가 명확하게 제한되어 있는가?

**Lux 통합**:
- Git stash/rollback 후크 연결
- Compile error threshold 초과 시 자동 abort
- PlayMode crash detector와 연동
- 파일 변경 목록 한도 설정

### Step5: Approval (SS11-SS12) — Category E

**Lux 매핑**: 최종 승인 및 기록

- **SS11**: 필요한 stakeholder 승인을 받았는가?
- **SS12**: 모든 homework 항목이 해결되었는가?

**Lux 승인 아티팩트**:
- `lux screenshot`으로 작업 전/후 상태 캡처
- Decision record JSON with lineage
- Test result report 첨부
- AI agent decision log 연결

## Scoring System

| Verdict | Score Range | Action |
|---------|------------|--------|
| **PASS** | ≥85 | 작업 진행 ✅ |
| **REVIEW** | 60–84 | homework 수정 후 재검증 ⚠️ |
| **REJECT** | <60 (or B/C hard reject) | 작업 중단 ❌ |

**Category Weights**: A=20, B=25, C=20, D=15, E=20 (합계 100)
**B(C) Hard Reject**: Category B 또는 C에서 `No` 응답 → 즉시 REJECT

## Execution Modes

| Mode | Steps Active | Use Case |
|------|-------------|----------|
| **Direct** | 1→2→3→4→5 | 일반적인 Lux 자동화 작업 |
| **Socratic** | 0→5 | 새로운 시스템 설계/생성 |
| **Mixed** | 0→1→2→3→4→5 | 복잡한 AI 주도 리팩토링 |
| **Quick** | 1→3→5 | 반복적인 일상 작업 (빌드, 테스트) |

## Review Levels

| Level | Questions | Time | Use Case |
|-------|-----------|------|----------|
| **Full** | 20 (all categories) | ~30s | 초기 통합, 위험 작업 |
| **Standard** | 12 (required only) | ~15s | 일반 자동화 |
| **Quick** | 8 (A+B critical) | ~2s | 반복 빌드/테스트 |
