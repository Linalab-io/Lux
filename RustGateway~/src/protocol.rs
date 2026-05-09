use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const PROTOCOL_VERSION: u32 = 1;

/// Stable identifier for the origin of an event.
/// Every LuxEvent carries exactly one source.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventSource {
    /// User action inside the Unity Editor (Inspector, menu, console, etc.)
    Editor,
    /// AI agent work step (codex, opencode, MCP tool invocation, skill execution)
    Ai,
    /// Gameplay runtime event (playmode callbacks, player input, scene lifecycle)
    Runtime,
}

/// High-level classification of an event within the LUX pipeline.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventCategory {
    Playmode,
    Scene,
    Log,
    AiActionLog,
    Tool,
    Input,
    Screenshot,
    Hierarchy,
}

/// Records which fields were redacted, why, and when — without leaking original values.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RedactionMetadata {
    pub redacted_fields: Vec<String>,
    pub redaction_classes: Vec<String>,
    /// ISO 8601 timestamp of when redaction was applied.
    pub timestamp: Option<String>,
}

/// Describes how long an event or log entry should be retained.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionMetadata {
    pub max_age_days: Option<u32>,
    pub max_lines: Option<usize>,
    pub policy: Option<String>,
    /// ISO 8601 timestamp when the retention window was created.
    pub created_at: Option<String>,
    /// ISO 8601 timestamp when the retention window expires.
    pub expires_at: Option<String>,
}

/// Unified event schema representing Editor user actions, AI work steps, and gameplay runtime events.
///
/// This is the canonical wire format for every event that flows through the LUX pipeline.
/// It carries stable identifiers, timestamps, project context, classification, a human-readable
/// summary, structured payload, and optional redaction/retention metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventEnvelope {
    pub schema_version: u32,
    /// Stable UUID-style event identifier.
    pub event_id: String,
    pub category: EventCategory,
    /// Origin of the event (editor user action, AI work step, or runtime).
    pub source: EventSource,
    /// Session that produced this event.
    pub session_id: String,
    /// ISO 8601 UTC timestamp of when the event was captured.
    pub captured_at_utc: String,
    /// File-system path to the Unity project (redacted in transit).
    pub project_path: Option<String>,
    /// Human-readable one-line summary of what happened.
    pub summary: Option<String>,
    pub redaction_metadata: Option<RedactionMetadata>,
    pub retention_metadata: Option<RetentionMetadata>,
    /// Arbitrary structured payload specific to the event category.
    pub payload: Value,
}

/// Type alias so the AC1 spec name "LuxEvent" is available in public API.
pub type LuxEvent = EventEnvelope;

impl EventEnvelope {
    pub fn schema_example() -> Self {
        Self {
            schema_version: PROTOCOL_VERSION,
            event_id: "example-event".to_string(),
            category: EventCategory::Tool,
            source: EventSource::Editor,
            session_id: "example-session".to_string(),
            captured_at_utc: "2026-04-30T00:00:00.0000000Z".to_string(),
            project_path: Some("/Users/example/UnityProjects/NeonGlitch".to_string()),
            summary: Some("Lux gateway event envelope prototype".to_string()),
            redaction_metadata: Some(RedactionMetadata {
                redacted_fields: vec!["summary".to_string(), "payload.token".to_string()],
                redaction_classes: vec!["secret".to_string(), "project_path".to_string()],
                timestamp: Some("2026-04-30T00:01:00Z".to_string()),
            }),
            retention_metadata: Some(RetentionMetadata {
                max_age_days: Some(30),
                max_lines: Some(10_000),
                policy: Some("default".to_string()),
                created_at: Some("2026-04-30T00:00:00Z".to_string()),
                expires_at: Some("2026-05-30T00:00:00Z".to_string()),
            }),
            payload: json!({
                "kind": "example",
                "message": "Lux gateway event envelope prototype"
            }),
        }
    }

    pub fn normalize(mut self) -> Self {
        if self.schema_version == 0 {
            self.schema_version = PROTOCOL_VERSION;
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_example_has_phase_one_categories() {
        let json = serde_json::to_value(EventEnvelope::schema_example()).unwrap();
        assert_eq!(json["schema_version"], PROTOCOL_VERSION);
        assert_eq!(json["category"], "tool");
    }

    #[test]
    fn all_categories_serialize_as_protocol_names() {
        let names = [
            EventCategory::Playmode,
            EventCategory::Scene,
            EventCategory::Log,
            EventCategory::AiActionLog,
            EventCategory::Tool,
            EventCategory::Input,
            EventCategory::Screenshot,
            EventCategory::Hierarchy,
        ]
        .map(|category| serde_json::to_value(category).unwrap());

        assert_eq!(
            names,
            [
                json!("playmode"),
                json!("scene"),
                json!("log"),
                json!("ai-action-log"),
                json!("tool"),
                json!("input"),
                json!("screenshot"),
                json!("hierarchy"),
            ]
        );
    }

    #[test]
    fn ai_action_log_roundtrips_through_serde() {
        let serialized = serde_json::to_value(EventCategory::AiActionLog).unwrap();
        assert_eq!(serialized, json!("ai-action-log"));

        let deserialized: EventCategory = serde_json::from_value(json!("ai-action-log")).unwrap();
        assert_eq!(deserialized, EventCategory::AiActionLog);
    }

    #[test]
    fn enriched_event_schema_roundtrips_through_serde() {
        let event = EventEnvelope {
            schema_version: PROTOCOL_VERSION,
            event_id: "event-1".to_string(),
            category: EventCategory::AiActionLog,
            source: EventSource::Ai,
            session_id: "session-1".to_string(),
            captured_at_utc: "2026-05-05T00:00:00Z".to_string(),
            project_path: Some("/project".to_string()),
            summary: Some("AI work step completed".to_string()),
            redaction_metadata: Some(RedactionMetadata {
                redacted_fields: vec!["payload.email".to_string()],
                redaction_classes: vec!["gameplay_sensitive".to_string()],
                timestamp: Some("2026-05-05T00:00:01Z".to_string()),
            }),
            retention_metadata: Some(RetentionMetadata {
                max_age_days: Some(7),
                max_lines: Some(100),
                policy: Some("aggressive".to_string()),
                created_at: Some("2026-05-05T00:00:00Z".to_string()),
                expires_at: Some("2026-05-12T00:00:00Z".to_string()),
            }),
            payload: json!({ "kind": "work-step" }),
        };

        let serialized = serde_json::to_string(&event).unwrap();
        assert!(serialized.contains("project_path"));
        assert!(serialized.contains("redaction_metadata"));
        assert!(serialized.contains("retention_metadata"));
        assert!(serialized.contains("summary"));
        assert!(serialized.contains("\"source\":\"ai\""));

        let deserialized: EventEnvelope = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, event);
        assert_eq!(deserialized.summary.as_deref(), Some("AI work step completed"));
        assert_eq!(deserialized.source, EventSource::Ai);
        assert_eq!(
            deserialized
                .redaction_metadata
                .as_ref()
                .and_then(|r| r.timestamp.as_deref()),
            Some("2026-05-05T00:00:01Z")
        );
        assert_eq!(
            deserialized
                .retention_metadata
                .as_ref()
                .and_then(|r| r.created_at.as_deref()),
            Some("2026-05-05T00:00:00Z")
        );
        assert_eq!(
            deserialized
                .retention_metadata
                .as_ref()
                .and_then(|r| r.expires_at.as_deref()),
            Some("2026-05-12T00:00:00Z")
        );
    }
}
