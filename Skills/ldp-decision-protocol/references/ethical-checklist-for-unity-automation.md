# Ethical Checklist for Unity Automation
# Unity 자동화 윤리 체크리스트

> Lux Unity Toolkit으로 AI 에이전트가 Unity 편집기를 자동화할 때 반드시 확인해야 할
> 윤리적 체크리스트입니다. 각 항목은 LDP Category B (Ethics) 및 Category C (Risk)에 매핑됩니다.

---

## Section A: Automated Testing Ethics (자동화 테스트 윤리)

### A1. Data Privacy in Tests (테스트 데이터 프라이버시)

- [ ] **A1.1** PlayMode 테스트가 실제 플레이어 저장 데이터(PlayerPrefs, 파일)에 접근하지 않는가?
  - *LDP*: B2 (Data Privacy) — Hard Reject candidate
  - *검증*: `PlayerPrefs.DeleteAll()` mock 또는 isolated sandbox 사용
  - *Lux 명령어*: `lux run-tests` 전 `lux dynamic-code`로 sandbox 설정 확인

- [ ] **A1.2** 테스트 fixture에 민감한 정보(API key, token, 개인정보)가 하드코딩되지 않았는가?
  - *LDP*: B2 + C4 (Data Loss Risk)
  - *검증*: `.gitignore`에 fixture 파일 포함, environment variable 사용

- [ ] **A1.3** 테스트 로그에 민감한 stack trace나 메모리 덤프가 포함되지 않는가?
  - *LDP*: B3 (Transparency) — 로그 redaction 정책 필요
  - *검증*: `lux get-logs` 출력에 PII filter 적용

### A2. Test Integrity (테스트 무결성)

- [ ] **A2.1** AI가 생성한 테스트가 false positive/negative를 유발하지 않는가?
  - *LDP*: B4 (Fairness) — 테스트가 실제 버그를 잡는가?
  - *검증*: mutation testing 또는 known-bug regression set

- [ ] **A2.2** PlayMode 테스트가 게임 밸런스/확률을 조작하지 않는가?
  - *LDP*: B1 (User Harm) — gacha/loot box 확률 변조 방지
  - *검증*: RNG seed 고정, 확률 분포 assertion

- [ ] **A2.3** 자동화 테스트가 anti-cheat 시스템을 트리거하지 않는가?
  - *LDP*: B3 (Transparency) — 자동화 탐지 우회 금지
  - *검증*: Editor-only 실행, runtime injection 없음 확인

### A3. Resource Usage (리소스 사용)

- [ ] **A3.1** 자동화 테스트 스위트가 과도한 CI/CD 리소스를 소비하지 않는가?
  - *LDP*: A4 (Thresholds) — 실행 시간/메모리 상한
  - *검증*: `lux run-tests --json`에서 duration/memory 모니터링

- [ ] **A3.2** 병렬 테스트 실행 시 race condition이 발생하지 않는가?
  - *LDP*: C2 (Stop Condition) — 동시성 문제 감지
  - *검증*: isolation container 또는 sequential fallback

---

## Section B: AI-Generated Code Review (AI 생성 코드 리뷰)

### B1. Code Quality (코드 품질)

- [ ] **B1.1** AI가 생성한 C# 코드가 Unity coding convention을 따르는가?
  - *LDP*: B5 (Accountability) — 코드 품질에 대한 책임
  - *검증*: namespace `UnityEditor`, class prefix `Lux`, partial class 규칙

- [ ] **B1.2** 생성된 코드에 보안 취약점(SQL injection, XSS equivalent, unsafe deserialize)이 없는가?
  - *LDP*: B1 (User Harm) + C3 (Recovery Procedure)
  - *검증*: static analysis rule set, forbidden API blocklist

- [ ] **B1.3** AI 코드가 기존 코드베이스와 호환되는가(API surface, serialization format)?
  - *LDP*: D2 (Change Blast Radius) — 호환성 파급 범위
  - *검증*: `lux compile` error/warning zero, existing tests pass

### B2. Attribution & Licensing (저작권 & 라이선스)

- [ ] **B2.1** AI가 생성한 코드에 학습 데이터 출처가 필요한 경우 proper attribution이 있는가?
  - *LDP*: B2 (Attribution) — IP rights
  - *검증*: 코드 헤더 주석, LICENSE 파일 참조

- [ ] **B2.2** 생성된 코드가 오픈소스 라이선스 의무(compatibility, attribution)를 준수하는가?
  - *LDP*: B1 (Legal Compliance)
  - *검증*: dependency license scanner 통과

### B3. Maintainability (유지보수성)

- [ ] **B3.1** AI 생성 코드에 충분한 주석과 문서가 있는가?
  - *LDP*: B3 (Transparency) — 코드 이해 가능성
  - *검증*: XML doc comment覆盖率, SKILL.md 업데이트

- [ ] **B3.2** 생성 코드가 디버깅 가능한가(충분한 logging, error handling)?
  - *LDP*: C1 (Rollback Plan) — 문제 발생 시 추적 가능
  - *검증*: structured logging, anyhow-style error propagation

- [ ] **B3.3** AI 코드가 과도한 abstraction이나 unnecessary complexity를 도입하지 않는가?
  - *LDP*: D3 (Deps Impact) — 유지보수 부 debt 방지
  - *검증*: cyclomatic complexity threshold, code review checklist

---

## Section C: Asset Generation Ethics (에셋 생성 윤리)

### C1. Intellectual Property (지식재산권)

- [ ] **C1.1** AI가 생성한 texture/model/audio에 저작권 침해 위험이 없는가?
  - *LDP*: B1 (IP Rights) — Hard Reject candidate
  - *검증*: 생성 모델의 training data 라이선스 확인, reverse image search

- [ ] **C1.2** 생성된 asset이 상표권(trademark) 또는 public right of personality를 침해하지 않는가?
  - *LDP*: B1 (Legal Risk)
  - *검증*: legal review workflow for commercial assets

- [ ] **C1.3** AI asset generation prompt에 다른 작품의 스타일 모방 지시가 없는가?
  - *LDP*: B4 (Fairness/Deception) — 스타일 mimicry 투명성
  - *검증*: prompt audit log, style reference documentation

### C2. Quality & Transparency (품질 & 투명성)

- [ ] **C2.1** AI 생성 asset의 품질이 hand-crafted asset과 차별되지 않는가?
  - *LDP*: B4 (Quality Deception) — 플레이어에게 AI 생성 사실 숨기지 않기
  - *검증*: quality benchmark, artifact detection pass

- [ ] **C2.2** 생성 asset의 메타데이터에 AI 생성 fact가 기록되는가?
  - *LDP*: B3 (Transparency) — provenance tracking
  - *검증*: Unity asset metadata, .meta file custom fields

- [ ] **C2.3** 생성 asset이 platform guideline(App Store Review, Steam policy)을 준수하는가?
  - *LDP*: B1 (Policy Compliance)
  - *검증*: platform-specific AI disclosure requirement check

### C3. Resource & Performance (리소스 & 성능)

- [ ] **C3.1** AI 생성 asset의 파일 크기/폴리곤 수가 프로젝트 예산 내인가?
  - *LDP*: A2 (Scope Metrics)
  - *검증*: Unity Profiler memory budget, draw call limit

- [ ] **C3.2** 생성 asset이 runtime 성능(로드 시간, 렌더링)에 부정적 영향을 주지 않는가?
  - *LDP*: C4 (Irreversible Impact)
  - *검증*: profiling baseline comparison

---

## Section D: Automation Governance (자동화 거버넌스)

### D1. Human-in-the-Loop (사람 개입)

- [ ] **D1.1** 파괴적 작업(delete file, modify prefab, change scene) 전에 사람 승인이 필요한가?
  - *LDP*: E1 (Stakeholder Signoff)
  - *검증*: `lux ldp approve` 필수 단계, confirmation prompt

- [ ] **D1.2** AI 자동화의 결정을 사람이 override할 수 있는가?
  - *LDP*: C1 (Rollback) + E2 (Risk Comprehension)
  - *검증*: manual revert command, emergency stop mechanism

- [ ] **D1.3** 자동화 작업의 진행 상태가 실시간으로 모니터링 가능한가?
  - *LDP*: B3 (Transparency)
  - *검증*: progress callback, status endpoint, log streaming

### D2. Audit & Accountability (감사 & 책임)

- [ ] **D2.1** 모든 자동화 작업이 불변의 decision record로 기록되는가?
  - *LDP*: E3 (Documentation Complete)
  - *검증*: SQLite record, JSON export, tamper-evident log

- [ ] **D2.2** 각 결정에 대해 "누가(AI agent), 무엇을, 언제, 왜" 기록되는가?
  - *LDP*: B5 (Accountability) + E1-E3
  - *검증*: DecisionRecord struct fields populated (id, created_at, input, steps, verdict)

- [ ] **D2.3** Decision lineage(부모-자식 결정 관계)가 추적 가능한가?
  - *LDP*: LineageEntry (parent_id, generation)
  - *검증*: `lux ldp history --lineage`로 계보 조회

### D3. Boundary Setting (경계 설정)

- [ ] **D3.1** 자동화가 접근할 수 있는 Unity API 범위가 제한되어 있는가?
  - *LDP*: D1 (Feature Boundary) + C2 (Sandbox Boundary)
  - *검증*: allowlist/denylist for Editor API classes

- [ ] **D3.2** 파일 시스템 접근 경로가 제한되어 있는가?
  - *LDP*: C2 (Isolation Failure prevention)
  - *검증*: project root sandbox, path traversal protection

- [ ] **D3.3** 네트워크 호출이 명시적으로 허용된 경우에만 가능한가?
  - *LDP*: B2 (Data Privacy) + C3 (Security)
  - *검증*: network permission allowlist, outbound request log

---

## Checklist Usage Guide

### Before Running Any Lux Operation

```
1. Operation 식별 → Mapping Table에서 LDP Step/Category 확인
2. 해당 Section의 체크리스트 항목 전체 확인
3. 모든 [ ] 항목이 Pass인 경우 → lux ldp check --operation <name>
4. 하나라도 Fail인 경우 → 작업 중단 또는 완화 조치 후 재확인
5. LDP Verdict == PASS → Lux operation 실행
6. 작업 완료 → Post-operation checklist + lux ldp record
```

### Scoring

| Section | Pass 기준 | Weight |
|---------|----------|--------|
| A. Testing Ethics | 6/6 항목 | 25% |
| B. Code Review | 9/9 항목 | 35% |
| C. Asset Generation | 8/8 항목 | 25% |
| D. Governance | 9/9 항목 | 15% |
| **Total** | **32/32 = 100%** | **100%** |

**Hard Fail**: A1.1, A1.2, B1.2, C1.1, C1.2, D3.2 중任何一个 No → **즉시 REJECT**
