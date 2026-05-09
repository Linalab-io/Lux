# Lux ↔ LDP Mapping Table

> Lux Unity Toolkit의 각 작업(operation)이 LDP Decision Protocol의 어느 Step/Category에 매핑되는지 정의합니다.
> AI agent가 Lux 명령어를 실행하기 전 참조하여 적절한 LDP 검증 레벨을 선택합니다.

## Quick Reference Matrix

| Lux Operation | LDP Step | Category | Level | Domain | Mode |
|---------------|----------|----------|-------|--------|------|
| `lux compile` | 1+2+4 | A,C,D | Quick | Game | Direct |
| `lux run-tests` | 1+2+3+4 | A,B,C,D | Standard | Game | Direct |
| `lux dynamic-code` | 0+1+2+3+4+5 | All | Full | Game | Mixed |
| `lux screenshot` | 1+5 | E | Quick | Generic | Direct |
| `lux unity context` | 1 | — | Quick | Generic | Direct |
| `lux get-logs` | — | — | — | — | Skip* |
| `lux clear-console` | — | — | — | — | Skip* |
| `lux find-game-objects` | 1+3 | B | Quick | Game | Direct |
| `lux get-hierarchy` | 1 | — | Quick | Generic | Direct |
| `lux play-mode` (start) | 1+2+3+4 | A,B,C,D | Standard | Game | Direct |
| `lux mouse / lux keyboard` | 1+3+4 | B,C,D | Standard | Game | Direct |
| `lux record` (start) | 1+2+3+4+5 | All | Full | Game | Mixed |
| `lux replay` | 1+2+3+4 | A,B,C,D | Standard | Game | Direct |
| `lux launch` | 1+2+4 | A,C,D | Standard | Game | Direct |
| AI Bridge remote op | 0+1+2+3+4+5 | All | Full | Game | Mixed |
| CodexImage generate | 0+1+2+3+4+5 | All | Full | Game | Mixed |
| `lux skill install` | 1+5 | E | Quick | Generic | Direct |
| Git integration | 1+4 | C,D | Quick | Generic | Direct |

*\*Skip = 읽기 전용 또는 파괴적이지 않은 작업으로 LDP 검증 생략 가능*

---

## Detailed Mappings

### 1. Build & Compile Operations

#### `lux compile`

| LDP Attribute | Value |
|---------------|-------|
| Primary Step | Step2 (Numbers/A) + Step4 (Termination/C,D) |
| Key Question | SS4: 컴파일 메트릭이 기록되었는가? SS9: 실패 시 rollback 계획은? |
| Ethics Check (B) | 생성된 코드에 보안 취약점이 있는가? |
| Input for LDP | compile output JSON (error count, warning count, duration, files changed) |
| Pass Criteria | 0 error, warnings < 10, duration < threshold |
| Auto-Reject Condition | 컴파일 에러 > 50 or 보안 warning 존재 |

#### `lux run-tests`

| LDP Attribute | Value |
|---------------|-------|
| Primary Step | Step2 (A) + Step3 (B) + Step4 (C,D) |
| Key Question | SS6: 테스트가 플레이어 데이터에 접근하는가? SS5: 테스트 커버리지 기준은? |
| Ethics Check (B) | PlayMode 테스트에서 네트워크/파일 시스템 접근 권한 |
| Input for LDP | test results JSON (pass/fail, duration, platform, test names) |
| Pass Criteria | pass rate ≥ 95%, no data access violations |
| Auto-Reject Condition | player data leak detected or core test failure |

### 2. Runtime Operations

#### `lux dynamic-code` (C# snippet execution)

| LDP Attribute | Value |
|---------------|-------|
| Primary Step | **ALL steps** (highest risk operation) |
| Key Question | SS3: 실행 결과 되돌릴 수 있는가? SS7: 민감한 API를 호출하는가? |
| Ethics Check (B) | File.IO, Network, PlayerPrefs, UnityEngine.API 점검 |
| Input for LDP | snippet content, target assembly, execution context |
| Pass Criteria | sandboxed execution, no destructive APIs, audit log |
| Auto-Reject Condition | File deletion, network call, PlayerPrefs write detected |

#### PlayMode Input (`lux mouse`, `lux keyboard`)

| LDP Attribute | Value |
|---------------|-------|
| Primary Step | Step3 (B) + Step4 (C,D) |
| Key Question | SS6: UI 자동화가 플레이어 경험을 왜곡하는가? |
| Ethics Check (B) | 입력 자동화가 치팅/봇 탐지에 걸리는가? |
| Input for LDP | input sequence, target UI elements |
| Pass Criteria | editor-only automation, no runtime injection |

### 3. Observation Operations

#### `lux screenshot`

| LDP Attribute | Value |
|---------------|-------|
| Primary Step | Step1 (Acknowledge) + Step5 (Approval/E) |
| Key Question | SS11: 스크린샷 저장 위치와 보안은? |
| Ethics Check (B) | 스크린샷에 민감 정보(API key, 개발 데이터) 포함 여부 |
| Input for LDP | screenshot path, resolution, redaction rules |
| Pass Criteria | redaction applied if needed, secure storage |

#### `lux get-logs`, `lux clear-console`, `lux unity context`, `lux get-hierarchy`

| LDP Attribute | Value |
|---------------|-------|
| Primary Step | **Skip** (read-only, non-destructive) |
| Note | Production 환경에서는 log redaction 권장 |

### 4. Asset & Code Generation

#### CodexImage / AI Asset Generation

| LDP Attribute | Value |
|---------------|-------|
| Primary Step | **ALL steps with Socratic (Step0)** |
| Key Question | SS6: 생성된 asset의 저작권/라이선스는? SS8: 생성 과정이 투명한가? |
| Ethics Check (B) | Training data attribution, style mimicry, license compliance |
| Input for LDP | prompt, model used, output path, intended use |
| Pass Criteria | Proper licensing, no copyright infringement, documented provenance |
| Auto-Reject Condition | Unlicensed training data or trademark infringement |

### 5. Record & Replay

#### `lux record`, `lux replay`

| LDP Attribute | Value |
|---------------|-------|
| Primary Step | Step2 (A) + Step3 (B) + Step4 (C,D) + Step5 (E) |
| Key Question | SS9: 녹화 데이터에 민감 정보가 포함되는가? SS10: 재생 범위는 제한되는가? |
| Ethics Check (B) | Input recording privacy, replay determinism safety |
| Input for LDP | recording duration, input types captured, data size |
| Pass Criteria | Redacted recording, deterministic replay, size limits |

---

## Category-Specific Question Selection

### Category A (Numbers) — Applied to metrics-producing operations

| Lux Operation | Which A-Questions Apply |
|--------------|------------------------|
| compile | A1(cost/time), A2(scope), A3(baseline), A4(thresholds), A5(risk-metric) |
| run-tests | A1(duration), A2(coverage), A3(baseline-pass-rate), A4(flaky-threshold), A5(regression-metric) |
| launch | A1(resource-cost), A2(memory), A3(startup-time-baseline), A4(timeout), A5(crash-rate) |

### Category B (Ethics) — Applied to user/player-affecting operations

| Lux Operation | Which B-Questions Apply |
|--------------|------------------------|
| dynamic-code | B1(user-harm), B2(data-privacy), B3(transparency), B4(fairness), B5(accountability) |
| play-mode-input | B1(gameplay-integrity), B2(player-data), B3(detection-evasion), B4(automation-disclosure), B5(responsibility) |
| ai-generate-asset | B1(ip-rights), B2(attribution), B3(consent), B4(quality-deception), B5(provenance) |
| find-game-objects | B2(data-exposure), B3(access-control) |

### Category C (Risk/Termination) — Applied to all state-changing operations

| Lux Operation | Which C-Questions Apply |
|--------------|------------------------|
| compile | C1(rollback-plan), C2(stop-condition), C3(recovery-procedure), C4(data-loss-risk) |
| dynamic-code | C1(undo-capability), C2(sandbox-boundary), C3(isolation-failure), C4(irreversible-change) |
| play-mode-start | C1(crash-recovery), C2(hang-timeout), C3(state-corruption), C4(save-data-risk) |

### Category D (Scope) — Applied to complex multi-step operations

| Lux Operation | Which D-Questions Apply |
|--------------|------------------------|
| record/replay | D1(feature-boundary), D2(change-blast-radius), D3(deps-impact), D4(schedule-vs-quality) |
| ai-bridge-remote | D1(api-surface), D2(cascade-failure), D3(editor-stability), D4(session-lifetime) |

### Category E (Approval) — Applied to all recorded operations

| Lux Operation | Which E-Questions Apply |
|--------------|------------------------|
| All operations | E1(stakeholder-signoff), E2(risk-comprehension), E3(documentation-complete) |
| screenshot-specific | E1(visual-verification), E2(audit-trail) |
| post-operation | E3(result-archived), E1(signoff-recorded) |
