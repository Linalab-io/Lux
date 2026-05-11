export interface UnitySpec {
  required_version: string | null
  detected_version: string | null
  render_pipeline: "urp" | "hdrp" | "built-in" | null
  scripting_backend: "il2cpp" | "mono" | null
}

export interface TargetsSpec {
  platforms: string[]
  min_sdk: Record<string, string>
  test_platform: string | null
}

export interface PackageEntry {
  name: string
  reason: string | null
  version: string | null
}

export interface PackagesSpec {
  required: PackageEntry[]
  forbidden: PackageEntry[]
  detected: PackageEntry[]
}

export interface TestingSpec {
  framework: string | null
  strategy: string | null
  coverage: boolean
}

export interface GlossarySpec {
  path: string
  last_updated: string | null
  term_count: number
}

export interface DomainSpec {
  name: string
  content_path: string
  fields: Record<string, unknown>
  ambiguity_score: number
  last_evaluated: string | null
  defined: boolean
}

export type PillarStatus = "Strong" | "NeedsWork" | "Missing"

export interface PillarRating {
  status: PillarStatus
  description: string | null
  score: number
}

export interface PhaseResult {
  name: string
  status: PillarStatus
  summary: string | null
  score: number
  questions: string[]
}

export interface TetradResult {
  mechanics: PillarRating
  story: PillarRating
  aesthetics: PillarRating
  technology: PillarRating
  harmony_score: number
}

export interface AssessmentResult {
  status: PillarStatus
  viability_score: number
  strengths: string[]
  risks: string[]
  recommendations: string[]
  summary: string | null
}

export interface SchellEvaluation {
  phase1_experience: PhaseResult
  phase2_tetrad: TetradResult
  phase3_core_loop: PhaseResult
  phase4_motivation: PhaseResult
  phase5_assessment: AssessmentResult
}

export interface LuxSpecProject {
  version: string
  project_id: string
  project_name: string
  created_at: string
  updated_at: string
  source: string
  status: "Draft" | "Active" | "Deprecated"
  domains: {
    design: DomainSpec | null
    architecture: DomainSpec | null
    art_style: DomainSpec | null
    audio: DomainSpec | null
    narrative: DomainSpec | null
    levels: DomainSpec | null
    ui_ux: DomainSpec | null
    custom: Record<string, DomainSpec>
  }
  schell_evaluation: SchellEvaluation
  overall_ambiguity: number
  unity: UnitySpec | null
  targets: TargetsSpec | null
  packages: PackagesSpec | null
  testing: TestingSpec | null
  glossary: GlossarySpec | null
}

export interface LuxEvalResult {
  should_continue: boolean
  next_action: string
  ambiguity_score: number
  continuation_count: number
}

export interface LuxPluginConfig {
  maxContinuations: number
  specPath: string
  glossaryPath: string
  targetAmbiguity: number
}

export interface LuxGlossaryEntry {
  term: string
  definition: string
  context: string
  first_seen: string
}
