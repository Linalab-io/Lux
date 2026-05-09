---
name: ldp-decision-protocol
description: "Lazy-load when ethical verification, AI-driven Unity automation guardrails, or LDP (Lina Decision Protocol) checks are needed before/after Lux operations. Bridges LDP's 5-step decision framework with Lux's compile/test/build pipeline."
---

# LDP Decision Protocol (LDP 통합 스킬)

## [한국어] LDP 결정 프로토콜 — Lux Unity 자동화 윤리 검증

### LDP란 무엇인가?

**Lina Decision Protocol (LDP)**은 AI 주도 의사결정에 대한 구조화된 **윤리 검증 프레임워크**입니다. 5단계 프로토콜(Acknowledge → Numbers → Ethics → Termination → Approval)을 통해 모든 자동화된 의사결정이 투명하고, 책임 있고, 재현 가능하도록 보장합니다.

Lux가 AI 에이전트를 통해 Unity 편집기를 자동화할 때, LDP는 다음을 제공합니다:

- **Pre-build 가드레일**: 컴파일/빌드/테스트 실행 전 윤리적 적합성 검증
- **Post-operation 검증**: 자동화 작업 완료 후 결과물의 안전성과 품질 심사
- **결정 감사 추적**: 모든 AI 의사결정에 대한 기록과 계보(lineage) 관리

### Lux가 LDP가 필요한 이유

| 문제 | LDP 해결책 |
|------|-----------|
| AI가 생성한 C# 코드가 프로젝트를 망칠 수 있음 | Step3 Ethics: 코드 변경 영향 범위 검증 |
| 자동화 빌드가 예상치 못한 asset 변경을 일으킴 | Step4 Termination: 롤백 계획 및 중단 조건 명시 |
| PlayMode 테스트에서 플레이어 데이터 유출 위험 | Step2 Numbers: 데이터 흐름 정량화 |
| AI 결정에 대한 책임 소재 불명확 | Step5 Approval: 서명(stakeholder sign-off) 기록 |
| 반복적인 자동화 작업에서 누적되는 드리프트 | Step1 Acknowledge: 목표-결과 간 편차 측정 |

### LDP 5단계 → Lux 워크플로우 매핑

```
┌─────────────────────────────────────────────────────────────────────┐
│                    LDP 5-Step Protocol                              │
│                                                                     │
│  SS1-SS3  ACKNOWLEDGE   ──→  lux unity context (작업 전 상태 파악)    │
│  SS4-SS5  NUMBERS       ──→  lux compile --json (메트릭 수집)        │
│  SS6-SS8  ETHICS        ──→  LDP ethically-check (윤리 검증)         │
│  SS9-SS10 TERMINATION   ──→  lux run-tests (중단 조건 확인)          │
│  SS11-SS12 APPROVAL     ──→  기록 저장 + lux screenshot (승인 증빙)   │
└─────────────────────────────────────────────────────────────────────┘
```

### 통합 포인트

#### 1. RustGateway Hooks (Axum middleware)

```rust
// RustGateway~/src/ldp_middleware.rs (신규 파일 제안)
pub async fn ldp_pre_build_guard(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, AppError> {
    // 1. 요청 파싱: 어떤 Unity 작업인지 식별
    // 2. LDP 엔진 호출: domain=Game, level=Quick
    // 3. Verdict==Pass 인 경우만 next().await
    // 4. Verdict==Reject 시 403 + homework 반환
}
```

#### 2. MCP Tool Wrappers

```typescript
// McpHelper~/src/ldp-tools.ts (제안)
const ldpTools = {
  ldp_verify_operation: {
    description: "Run LDP ethics check before Lux operation",
    parameters: {
      operation: "compile|test|build|screenshot|dynamic-code",
      context: "string — current Unity project state summary"
    }
  },
  ldp_record_decision: {
    description: "Record post-operation result into LDP audit trail",
    parameters: {
      operation_id: "string",
      result: "success|failure|partial",
      artifacts: "string[] — generated file paths"
    }
  }
};
```

#### 3. CLI Commands

```bash
# Pre-build ethics check
lux ldp check --operation compile --level quick

# Full review with socratic mode
lux ldp review --operation ai-generate-code --mode mixed --domain game

# View decision history
lux ldp history --json

# Record manual approval
lux ldp approve <decision-id> --note "Reviewed by lead dev"
```

---

## [English] LDP Decision Protocol — Ethical Guardrails for Lux

### What is LDP?

The **Lina Decision Protocol (LDP)** is a structured ethical verification framework for AI-driven decisions. Its 5-step protocol ensures every automated decision is transparent, accountable, and reproducible.

When Lux automates the Unity Editor via AI agents, LDP provides:

- **Pre-build guardrails**: Ethical fitness verification before compile/build/test runs
- **Post-operation verification**: Safety and quality audit after automation tasks
- **Decision audit trail**: Records and lineage for all AI decisions

### Why Lux Needs LDP

| Problem | LDP Solution |
|---------|-------------|
| AI-generated C# code can break projects | Step3 Ethics: Code change impact validation |
| Automated builds cause unexpected asset changes | Step4 Termination: Rollback plans & stop conditions |
| PlayMode tests risk player data leakage | Step2 Numbers: Data flow quantification |
| Unclear accountability for AI decisions | Step5 Approval: Stakeholder sign-off records |
| Cumulative drift in repetitive automation | Step1 Acknowledge: Goal-result drift measurement |

### Integration Points

| Layer | Component | Role |
|-------|-----------|------|
| **RustGateway** | Axum middleware hook | Intercepts build/test commands, runs LDP pre-check |
| **MCP Helper** | Node.js tool wrappers | Exposes `ldp_verify` / `ldp_record` to AI agents |
| **CLI** | `lux ldp *` subcommands | Human-facing check/approve/history workflow |
| **C# Editor** | `LuxLdpGate.cs` | In-editor decision prompt before destructive operations |
| **Storage** | SQLite via ldp-storage | Persistent decision records with lineage |

---

## [日本語] LDP 決定プロトコル — Lux Unity自動化の倫理検証

### LDPとは？

**Lina Decision Protocol (LDP)**は、AI主導の意思決定のための構造化された**倫理検証フレームワーク**です。5ステッププロトコルを通じて、すべての自動化された意思決定が透明で、責任があり、再現可能であることを保証します。

### LuxがLDPを必要とする理由

| 問題 | LDPの解決策 |
|------|-----------|
| AIが生成したC#コードがプロジェクトを破壊する可能性 | Step3 Ethics: コード変更影響範囲の検証 |
| 自動化ビルドが予期せぬアセット変更を引き起こす | Step4 Termination: ロールバック計画と停止条件 |
| PlayModeテストでプレイヤーデータ漏洩のリスク | Step2 Numbers: データフローの定量化 |
| AI決定に対する責任所在が不明確 | Step5 Approval: ステークホルダー署名記録 |

---

## 언제 사용할까 (When to Use)

- ✅ **AI가 C# 코드를 생성/수정하기 전** — Step3 Ethics 검증
- ✅ **자동화 빌드 또는 PlayMode 테스트 실행 전** — Step2+Step4 검증
- ✅ **동적 코드 실행 (`lux dynamic-code`) 전** — 전체 5단계 Quick 모드
- ✅ **스크린샷/녹화 후 분석 리포트 생성 시** — Step5 Approval 기록
- ✅ **AI Bridge를 통한 원격 조작 전** — Full 모드 Socratic 검증
- ❌ **단순 읽기 전용 조회** (`lux unity context`, `lux get-logs`) — 불필요
- ❌ **수동으로 검토한 안전한 작업** — Skip 가능

## 사전 요건 (Prerequisites)

1. **LDP 엔진**: `lina-decision-protocol` 크레이트 빌드 완료 (`cargo build -p ldp-core`)
2. **Lux Core**: `lux-unity` 스킬 v1.0.0 이상 설치됨
3. **Unity 프로젝트**: `lux unity context`로 접근 가능한 상태
4. **SQLite**: ldp-storage 의존성 (decision record 지속성)

## 절차 (Procedure)

### 1. Pre-Build Check (빌드/컴파일 전)

```bash
# 1단계: 현재 Unity 컨텍스트 수집
lux unity context --json > .lux/ctx.json

# 2단계: LDP 윤리 검증 실행
lux ldp check \
  --operation compile \
  --input .lux/ctx.json \
  --level quick \
  --domain game \
  --json > .lux/ldp-verdict.json

# 3단계: Verdict 확인
# PASS → 진행
# REVIEW → homework 항목 수정 후 재검증
# REJECT → 작업 중단, 로그 확인
```

### 2. Post-Operation Verification (작업 완료 후)

```bash
# 1단계: 작업 결과 수집
lux run-tests --test-platform EditMode --json > .lux/test-results.json
lux screenshot --path .lux/post-op.png

# 2단계: LDP 레코드에 결과 기록
lux ldp record \
  --decision-id $(cat .lux/ldp-verdict.json | jq -r '.record_id') \
  --result success \
  --artifacts .lux/test-results.json,.lux/post-op.png \
  --json
```

### 3. AI Agent 통합 (MCP 경유)

```
Agent: "lux compile 실행해줘"
  → MCP: ldp_verify_operation(operation="compile", context=...)
  → LDP Engine: { verdict: "PASS", score: 92, record_id: "ldr_..." }
  → MCP: lux compile (실행)
  → MCP: ldp_record_decision(operation_id="ldr_...", result="success")
  → Agent: "컴파일 성공. LDP score: 92/100 (PASS)"
```

## 주의사항 (Pitfalls)

| Pitfall | 해결책 |
|---------|--------|
| **LDP 검증이 빌드 시간을 지연시킴** | Quick 모드 사용 (질문 8개만, ~2초) |
| **REJECT verdict에 대한 fallback 부재** | REJECT 시 자동 rollback 훅 연결 |
| **Domain mismatch** (Generic vs Game) | Lux 작업에는 항상 `--domain game` 사용 |
| **SQLite lock contention** | 병렬 작업 시 WAL 모드 활성화 |
| **Circular dependency** (LDP→Lux→LDP) | Pre-build 체크만 LDP 통과, post는 비동기 기록 |
| **한국어 질문-영어 agent 간 번역 비용** | LDP 엔진 내부적으로 locale 지원 확장 필요 |
| **Decision record storage bloat** | 주간 압축 + 90일 보 retention policy |

## 검증 (Verification)

```bash
# 1. 스킬 디렉토리 구조 확인
ls Skills/ldp-decision-protocol/
# Expected: manifest.json, SKILL.md, references/

# 2. manifest.json 유효성
python3 -c "
import json
m = json.load(open('Skills/ldp-decision-protocol/manifest.json'))
assert m['name'] == 'ldp-decision-protocol'
assert m['type'] == 'integration'
assert 'lux-unity' in m.get('dependencies', {})
print('✓ manifest valid')
"

# 3. LDP 엔진 연결 테스트
cd RustGateway~ && cargo run -- ldp check --help
# Assert: help output shows --operation, --level, --domain flags

# 4. End-to-end smoke test
lux ldp check --operation compile --level quick --domain game
# Assert: JSON output with verdict field (PASS/REVIEW/REJECT)

# 5. 참고 문서 존재 확인
ls Skills/ldp-decision-protocol/references/
# Expected: ldp-protocol-overview.md, lux-ldp-mapping-table.md,
#           ethical-checklist-for-unity-automation.md
```

## 참고 문서 (References)

- [LDP 5-Step Protocol Overview](./references/ldp-protocol-overview.md) — LDP 프로토콜 요약
- [Lux ↔ LDP Mapping Table](./references/lux-ldp-mapping-table.md) — Lux 작업과 LDP 카테고리 매핑
- [Unity Automation Ethics Checklist](./references/ethical-checklist-for-unity-automation.md) — Unity 자동화 윤리 체크리스트
