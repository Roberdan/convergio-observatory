//! Per-project telemetry — aggregate observatory data by project.
//!
//! `GET /api/observatory/project/:id/summary` — test count, build time,
//! agent hours aggregated from existing observatory data.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::Json;
use axum::routing::get;
use axum::Router;
use serde::Serialize;
use serde_json::json;

use crate::routes::ObservatoryState;

/// Build the project telemetry routes.
pub fn project_telemetry_routes(state: Arc<ObservatoryState>) -> Router {
    Router::new()
        .route(
            "/api/observatory/project/:id/summary",
            get(handle_project_summary),
        )
        .with_state(state)
}

/// Per-project telemetry summary.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ProjectSummary {
    pub project_id: String,
    pub event_count: i64,
    pub agent_events: i64,
    pub error_events: i64,
    pub anomaly_count: i64,
    pub latest_event_at: Option<String>,
}

async fn handle_project_summary(
    State(state): State<Arc<ObservatoryState>>,
    Path(project_id): Path<String>,
) -> Json<serde_json::Value> {
    let conn = match state.pool.get() {
        Ok(c) => c,
        Err(e) => {
            return Json(json!({
                "error": {"code": "POOL_ERROR", "message": e.to_string()}
            }));
        }
    };

    let summary = build_project_summary(&conn, &project_id);

    Json(json!({
        "ok": true,
        "summary": summary,
    }))
}

/// Build a telemetry summary for a specific project from timeline data.
pub fn build_project_summary(conn: &rusqlite::Connection, project_id: &str) -> ProjectSummary {
    let mut summary = ProjectSummary {
        project_id: project_id.to_string(),
        ..Default::default()
    };

    // Total events referencing this project (by org_id or source)
    summary.event_count = conn
        .query_row(
            "SELECT COUNT(*) FROM obs_timeline \
             WHERE org_id = ?1 OR source = ?1",
            [project_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // Agent-related events
    summary.agent_events = conn
        .query_row(
            "SELECT COUNT(*) FROM obs_timeline \
             WHERE (org_id = ?1 OR source = ?1) \
             AND event_type LIKE '%agent%'",
            [project_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // Error events
    summary.error_events = conn
        .query_row(
            "SELECT COUNT(*) FROM obs_timeline \
             WHERE (org_id = ?1 OR source = ?1) \
             AND event_type LIKE '%error%'",
            [project_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // Unresolved anomalies for this project
    summary.anomaly_count = conn
        .query_row(
            "SELECT COUNT(*) FROM obs_anomalies \
             WHERE entity_id = ?1 AND resolved = 0",
            [project_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // Latest event timestamp
    summary.latest_event_at = conn
        .query_row(
            "SELECT MAX(created_at) FROM obs_timeline \
             WHERE org_id = ?1 OR source = ?1",
            [project_id],
            |r| r.get(0),
        )
        .ok();

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_pool() -> convergio_db::pool::ConnPool {
        let pool = convergio_db::pool::create_memory_pool().unwrap();
        let conn = pool.get().unwrap();
        for m in crate::schema::migrations() {
            conn.execute_batch(m.up).unwrap();
        }
        pool
    }

    #[test]
    fn project_telemetry_routes_build() {
        let pool = setup_pool();
        let state = Arc::new(ObservatoryState { pool });
        let _router = project_telemetry_routes(state);
    }

    #[test]
    fn build_summary_empty_db() {
        let pool = setup_pool();
        let conn = pool.get().unwrap();
        let summary = build_project_summary(&conn, "test-project");
        assert_eq!(summary.project_id, "test-project");
        assert_eq!(summary.event_count, 0);
        assert_eq!(summary.agent_events, 0);
    }

    #[test]
    fn build_summary_with_events() {
        let pool = setup_pool();
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO obs_timeline (source, event_type, org_id, summary) \
             VALUES ('test-proj', 'agent_spawn', 'test-proj', 'spawned')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO obs_timeline (source, event_type, org_id, summary) \
             VALUES ('test-proj', 'build_error', 'test-proj', 'failed')",
            [],
        )
        .unwrap();
        let summary = build_project_summary(&conn, "test-proj");
        assert!(summary.event_count >= 2);
        assert!(summary.agent_events >= 1);
    }

    #[test]
    fn summary_serializes() {
        let s = ProjectSummary::default();
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("event_count"));
    }
}
