import { decideContinuation } from "./continuation-injector"
import { evaluateSpec } from "./spec-evaluator"
import type { LuxPluginConfig } from "./types"

interface OpenCodePluginEvent {
  type: string
}

interface OpenCodePluginEventPayload {
  event: OpenCodePluginEvent
}

interface OpenCodePluginServerContext {
  directory: string
}

interface OpenCodePluginServerResult {
  tool: Record<string, never>
  event: (payload: OpenCodePluginEventPayload) => Promise<void>
}

interface OpenCodePlugin {
  id: string
  server: (ctx: OpenCodePluginServerContext) => Promise<OpenCodePluginServerResult>
}

// Default config
const DEFAULT_CONFIG: LuxPluginConfig = {
  maxContinuations: 10,
  specPath: ".lux/spec.json",
  glossaryPath: ".lux/glossary.md",
  targetAmbiguity: 0.02,
}

const plugin = {
  id: "lux-spec-orchestrator",
  server: async (ctx: OpenCodePluginServerContext) => {
    const projectPath = ctx.directory
    const config = DEFAULT_CONFIG

    return {
      tool: {},
      event: async ({ event }: OpenCodePluginEventPayload) => {
        if (event.type === "session.idle") {
          const evalResult = evaluateSpec(projectPath, config)
          const decision = decideContinuation(projectPath, evalResult, config)

          console.log("[Lux] Continuation decision", {
            projectPath,
            shouldInject: decision.shouldInject,
            continuationCount: decision.continuationCount,
            message: decision.message,
          })
        }
      },
    }
  },
} satisfies OpenCodePlugin

export default plugin
