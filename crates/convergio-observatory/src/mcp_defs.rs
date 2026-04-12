//! MCP tool definitions for the observatory extension.

use convergio_types::extension::McpToolDef;
use serde_json::json;

pub fn observatory_tools() -> Vec<McpToolDef> {
    vec![
        McpToolDef {
            name: "cvg_observatory_timeline".into(),
            description: "Get observatory event timeline. Returns last 20 events by default."
                .into(),
            method: "GET".into(),
            path: "/api/observatory/timeline".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "limit": {"type": "integer", "description": "Max events to return (default 20)"},
                    "org_id": {"type": "string", "description": "Filter by organization"},
                    "event_type": {"type": "string", "description": "Filter by event type"},
                    "since": {"type": "string", "description": "ISO timestamp lower bound"}
                }
            }),
            min_ring: "community".into(),
            path_params: vec![],
        },
        McpToolDef {
            name: "cvg_observatory_search".into(),
            description: "Search observatory events.".into(),
            method: "GET".into(),
            path: "/api/observatory/search".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Search query"}
                }
            }),
            min_ring: "community".into(),
            path_params: vec![],
        },
        McpToolDef {
            name: "cvg_observatory_dashboard".into(),
            description: "Get observatory dashboard data.".into(),
            method: "GET".into(),
            path: "/api/observatory/dashboard".into(),
            input_schema: json!({"type": "object", "properties": {}}),
            min_ring: "community".into(),
            path_params: vec![],
        },
        McpToolDef {
            name: "cvg_observatory_metrics".into(),
            description: "Get observatory metrics.".into(),
            method: "GET".into(),
            path: "/api/observatory/metrics".into(),
            input_schema: json!({"type": "object", "properties": {}}),
            min_ring: "community".into(),
            path_params: vec![],
        },
        McpToolDef {
            name: "cvg_list_anomalies".into(),
            description: "List detected anomalies.".into(),
            method: "GET".into(),
            path: "/api/observatory/anomalies".into(),
            input_schema: json!({"type": "object", "properties": {}}),
            min_ring: "community".into(),
            path_params: vec![],
        },
        McpToolDef {
            name: "cvg_resolve_anomaly".into(),
            description: "Resolve a detected anomaly.".into(),
            method: "POST".into(),
            path: "/api/observatory/anomalies/:id/resolve".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "resolution": {"type": "string"}
                },
                "required": ["id"]
            }),
            min_ring: "trusted".into(),
            path_params: vec!["id".into()],
        },
        McpToolDef {
            name: "cvg_list_webhooks".into(),
            description: "List observatory webhooks.".into(),
            method: "GET".into(),
            path: "/api/observatory/webhooks".into(),
            input_schema: json!({"type": "object", "properties": {}}),
            min_ring: "community".into(),
            path_params: vec![],
        },
        McpToolDef {
            name: "cvg_create_webhook".into(),
            description: "Create an observatory webhook.".into(),
            method: "POST".into(),
            path: "/api/observatory/webhooks".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "url": {"type": "string"},
                    "events": {"type": "array", "items": {"type": "string"}}
                },
                "required": ["url"]
            }),
            min_ring: "trusted".into(),
            path_params: vec![],
        },
        McpToolDef {
            name: "cvg_delete_webhook".into(),
            description: "Delete an observatory webhook.".into(),
            method: "DELETE".into(),
            path: "/api/observatory/webhooks/:id".into(),
            input_schema: json!({
                "type": "object",
                "properties": {"id": {"type": "string"}},
                "required": ["id"]
            }),
            min_ring: "trusted".into(),
            path_params: vec!["id".into()],
        },
    ]
}
