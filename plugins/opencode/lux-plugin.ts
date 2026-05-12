type LuxPluginContext = {
  project?: string
  client?: {
    tui?: {
      showToast?: (input: { body: { message: string; variant: "success" | "error" | "info" } }) => Promise<unknown> | unknown
      appendPrompt?: (input: { body: { text: string } }) => Promise<unknown> | unknown
    }
    app?: {
      log?: (input: {
        body: {
          service: string
          level: "debug" | "info" | "warn" | "error"
          message: string
          extra?: Record<string, unknown>
        }
      }) => Promise<unknown> | unknown
    }
  }
  directory?: string
  worktree?: string
}

type OpenCodeEvent = {
  type?: string
  properties?: Record<string, unknown>
  [key: string]: unknown
}

type ToolInput = {
  tool?: string
  [key: string]: unknown
}

type CompactingOutput = {
  context?: string[]
  [key: string]: unknown
}

const SERVICE = "lux-plugin"

const eventProperties = (event: OpenCodeEvent) => event.properties ?? event

const valueToString = (value: unknown) => {
  if (typeof value === "string") return value
  if (value === undefined || value === null) return ""
  return String(value)
}

const resultStatus = (output: unknown) => {
  if (!output || typeof output !== "object") return "completed"

  const record = output as Record<string, unknown>
  if (typeof record.status === "string") return record.status
  if (typeof record.error === "string" || record.error) return "error"
  if (typeof record.ok === "boolean") return record.ok ? "success" : "error"

  return "completed"
}

export const LuxPlugin = async ({ project, client, directory, worktree }: LuxPluginContext) => {
  const log = async (level: "debug" | "info" | "warn" | "error", message: string, extra?: Record<string, unknown>) => {
    await client?.app?.log?.({
      body: {
        service: SERVICE,
        level,
        message,
        extra,
      },
    })
  }

  const toast = async (message: string, variant: "success" | "error" | "info" = "info") => {
    await client?.tui?.showToast?.({
      body: { message, variant },
    })
  }

  await log("info", "Lux plugin loaded", { project, directory, worktree })

  return {
    event: async ({ event }: { event: OpenCodeEvent }) => {
      if (event.type === "session.idle") {
        await log("info", "Lux observed session completion", { event: eventProperties(event) })
        await toast("Lux: Session completed", "success")
        return
      }

      if (event.type === "session.status") {
        const properties = eventProperties(event)
        const status = valueToString(properties.status)
        const message = valueToString(properties.message || properties.error)

        await log("info", "Lux observed session status", { status, message, event: properties })

        if (status.toLowerCase() === "error" || message.toLowerCase().includes("error")) {
          await toast(`Lux: ${message || "Session status error"}`, "error")
        }
      }
    },

    "tool.execute.after": async (input: ToolInput, output: unknown) => {
      const toolName = valueToString(input.tool)
      if (!toolName.startsWith("lux_")) return

      const status = resultStatus(output)
      const variant = status.toLowerCase() === "error" ? "error" : "info"

      await log("info", "Lux tool completed", { tool: toolName, status })
      await toast(`Lux: ${toolName} ${status}`, variant)
    },

    "experimental.session.compacting": async (_input: unknown, output: CompactingOutput) => {
      const context = [
        "Lux context:",
        "- Lux MCP tools available when configured: lux_status, lux_goals, lux_project_info.",
        "- Prefer lux_status for current process state and lux_goals for active project goals before changing Lux-managed state.",
        "- Preserve .lux as the single source of truth for Lux runtime state.",
      ].join("\n")

      if (!Array.isArray(output.context)) output.context = []
      output.context.push(context)

      await log("debug", "Lux context injected during session compacting")
    },
  }
}
