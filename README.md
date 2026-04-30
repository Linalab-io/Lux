# LUX

**LUX** stands for **Linalab Unity X**.

LUX is a unified Unity Editor AI adapter and automation toolkit. It is designed
for AI coding tools, external terminals, Git workflows, integrated Codex Image
generation, and Unity control protocols through one Editor-focused package.

The `lux` CLI is part of that workflow, but LUX is not just a Unity CLI wrapper:
it is the Unity-side integration layer for AI-assisted development, local
automation, validation, and project operations.

## Phase 1

- macOS-first Unity Editor adapter.
- External terminal/client connection through AI Bridge and MCP surfaces.
- Existing Git and AI Bridge modules preserved as internal assemblies.
- Lux-owned Unity bridge status is integrated through `LuxEditor/` and the
  packaged Rust `lux` CLI without embedding separate Unity automation projects.
- Codex Image is integrated as an internal Lux image-generation and
  data-pipeline capability for Codex-driven 2D asset workflows.
- Broad AI automation guardrails: command blacklist, audit log, and approval state.

## Phase 2 Foundations

- Remote gateway and WebRTC protocol design are documented as package scope.
- iOS app/PWA implementation is intentionally out of this package for now.

## Rust Gateway Prototype

- `RustGateway~/` contains the Phase 1 Rust WebSocket/HTTP gateway prototype.
- The gateway exposes `/health`, `/schema`, and `/events` WebSocket endpoints
  with local shared-token authentication.
- Unity can publish prototype event envelopes through
  `Tools > Linalab > Lux > Rust Gateway Prototype` after the Rust gateway is
  running.
- Install or update the global `lux` Rust CLI through `Window > Linalab >
  Lux Workbench` or `Tools > Linalab > Lux > Rust CLI > Install or Update
  Global Tool`. The terminal equivalent is `cargo install --path
  Packages/com.linalab.lux/RustGateway~ --force --locked`.
- Write Lux-owned Unity bridge status through `Tools > Linalab > Lux > Unity
  Bridge > Write Lux Bridge Settings`, then inspect it with `lux unity status`.
- Phase 1 event categories are `playmode`, `scene`, `log`, `tool`, `input`,
  `screenshot`, and `hierarchy`; high-fidelity streaming and remote-control
  sessions remain deferred.

## Entry Points

- `Window > Linalab > Lux Workbench`
- `Window > Linalab > Lux > Unity Git`
- `Tools > Linalab > Lux > AI Bridge`

## Structure

```text
com.linalab.lux/
├── LuxEditor/       # Adapter workbench, automation gateway, execution policy
├── UnityGitEditor/  # Unity Git integration
├── AiBridgeEditor/  # AI Bridge TCP server and protocol handler
├── CodexImage/      # Integrated Codex Image generation and pipeline tooling
├── RustGateway~/    # Rust WebSocket gateway prototype (external tooling)
├── McpHelper~/      # Node MCP helper
├── Skills/          # Lux Skill Manager definitions and references
├── seeds/           # Seed specifications for skill provisioning
└── *Tests/          # EditMode tests
```

## Acknowledgments

LUX was heavily inspired by and references significant portions of
[**unity-cli-loop**](https://github.com/hatayama/unity-cli-loop) by
[hatayama](https://github.com/hatayama) (formerly uLoopMCP).

The AI Bridge module (`AiBridgeEditor/`), including the TCP server, protocol
handler, dynamic code execution, input recording/replay, and the associated
skill/reference structure, was derived from unity-cli-loop. We are grateful for
the foundational work that made this project possible.

## License

This project is licensed under the [MIT License](LICENSE).
