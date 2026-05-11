import type { LuxEvalResult, LuxPluginConfig } from "./types"

interface SessionState {
  continuationCount: number
  lastAction: string
  lastTimestamp: number
}

const DEFAULT_MAX_CONTINUATIONS = 10
const DEFAULT_TARGET_AMBIGUITY = 0.02

const sessionStates = new Map<string, SessionState>()

function resolveMaxContinuations(config: LuxPluginConfig): number {
  return config.maxContinuations || DEFAULT_MAX_CONTINUATIONS
}

function resolveTargetAmbiguity(config: LuxPluginConfig): number {
  return config.targetAmbiguity || DEFAULT_TARGET_AMBIGUITY
}

function getOrCreateState(projectPath: string): SessionState {
  let state = sessionStates.get(projectPath)
  if (!state) {
    state = { continuationCount: 0, lastAction: "", lastTimestamp: Date.now() }
    sessionStates.set(projectPath, state)
  }
  return state
}

export function resetSession(projectPath: string): void {
  sessionStates.delete(projectPath)
}

export function getContinuationCount(projectPath: string): number {
  return getOrCreateState(projectPath).continuationCount
}

export function canContinue(projectPath: string, config: LuxPluginConfig): boolean {
  const state = getOrCreateState(projectPath)
  return state.continuationCount < resolveMaxContinuations(config)
}

export function formatNextAction(evalResult: LuxEvalResult): string {
  if (!evalResult.should_continue) {
    return ""
  }

  const parts: string[] = []

  if (evalResult.ambiguity_score > 0.7) {
    parts.push("[Lux] Spec is highly ambiguous. Addressing critical gaps:")
  } else if (evalResult.ambiguity_score > 0.4) {
    parts.push("[Lux] Spec needs refinement. Next priority:")
  } else {
    parts.push("[Lux] Spec is nearly complete. Remaining item:")
  }

  if (evalResult.next_action) {
    parts.push(evalResult.next_action)
  }

  return parts.join(" ")
}

export function formatMaxReachedMessage(_projectPath: string, config: LuxPluginConfig): string {
  const maxContinuations = resolveMaxContinuations(config)
  return `[Lux] Maximum continuations reached (${maxContinuations}). Current spec ambiguity: review and update manually, or start a new session to continue.`
}

export function decideContinuation(
  projectPath: string,
  evalResult: LuxEvalResult,
  config: LuxPluginConfig,
): {
  shouldInject: boolean
  message: string
  continuationCount: number
} {
  const state = getOrCreateState(projectPath)

  if (evalResult.ambiguity_score <= resolveTargetAmbiguity(config)) {
    return {
      shouldInject: false,
      message: "",
      continuationCount: state.continuationCount,
    }
  }

  if (!evalResult.should_continue) {
    return {
      shouldInject: false,
      message: "",
      continuationCount: state.continuationCount,
    }
  }

  if (state.continuationCount >= resolveMaxContinuations(config)) {
    return {
      shouldInject: false,
      message: formatMaxReachedMessage(projectPath, config),
      continuationCount: state.continuationCount,
    }
  }

  state.continuationCount += 1
  state.lastAction = evalResult.next_action
  state.lastTimestamp = Date.now()

  return {
    shouldInject: true,
    message: formatNextAction(evalResult),
    continuationCount: state.continuationCount,
  }
}

export function getSessionSummary(projectPath: string): {
  continuationCount: number
  lastAction: string
  elapsedMs: number
} {
  const state = getOrCreateState(projectPath)
  return {
    continuationCount: state.continuationCount,
    lastAction: state.lastAction,
    elapsedMs: Date.now() - state.lastTimestamp,
  }
}
