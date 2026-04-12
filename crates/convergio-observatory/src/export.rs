//! Export — Prometheus/Grafana metrics and webhook event delivery.

use rusqlite::{params, Connection};

use crate::types::{PrometheusMetric, WebhookPayload};

/// Generate Prometheus-compatible text exposition from observatory data.
pub fn prometheus_exposition(conn: &Connection) -> Result<String, rusqlite::Error> {
    let metrics = collect_prometheus_metrics(conn)?;
    let mut out = String::new();
    for m in &metrics {
        out.push_str(&format!(
            "# HELP {} {}\n",
            sanitize_metric_name(&m.name),
            sanitize_label_value(&m.help),
        ));
        out.push_str(&format!(
            "# TYPE {} {}\n",
            sanitize_metric_name(&m.name),
            m.metric_type,
        ));
        if m.labels.is_empty() {
            out.push_str(&format!("{} {}\n", sanitize_metric_name(&m.name), m.value,));
        } else {
            let labels: Vec<String> = m
                .labels
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}=\"{}\"",
                        sanitize_metric_name(k),
                        sanitize_label_value(v),
                    )
                })
                .collect();
            out.push_str(&format!(
                "{}{{{}}} {}\n",
                sanitize_metric_name(&m.name),
                labels.join(","),
                m.value,
            ));
        }
    }
    Ok(out)
}

/// Strip characters illegal in Prometheus metric/label names.
fn sanitize_metric_name(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == ':' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Escape backslash, double-quote, and newline in label values per the
/// Prometheus exposition format spec.
fn sanitize_label_value(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

/// Collect raw Prometheus metrics from observatory tables.
pub fn collect_prometheus_metrics(
    conn: &Connection,
) -> Result<Vec<PrometheusMetric>, rusqlite::Error> {
    let mut metrics = Vec::new();

    // Total timeline events
    if let Ok(n) = conn.query_row("SELECT COUNT(*) FROM obs_timeline", [], |r| {
        r.get::<_, f64>(0)
    }) {
        metrics.push(PrometheusMetric {
            name: "convergio_timeline_events_total".into(),
            help: "Total timeline events recorded".into(),
            metric_type: "counter".into(),
            value: n,
            labels: vec![],
        });
    }

    // Unresolved anomalies
    if let Ok(n) = conn.query_row(
        "SELECT COUNT(*) FROM obs_anomalies WHERE resolved = 0",
        [],
        |r| r.get::<_, f64>(0),
    ) {
        metrics.push(PrometheusMetric {
            name: "convergio_anomalies_unresolved".into(),
            help: "Number of unresolved anomalies".into(),
            metric_type: "gauge".into(),
            value: n,
            labels: vec![],
        });
    }

    // Events by source (last 24h)
    let mut stmt = conn.prepare(
        "SELECT source, COUNT(*) FROM obs_timeline \
         WHERE created_at >= datetime('now', '-24 hours') \
         GROUP BY source",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
    })?;
    for r in rows {
        let (source, count) = r?;
        metrics.push(PrometheusMetric {
            name: "convergio_timeline_events_24h".into(),
            help: "Timeline events in last 24h by source".into(),
            metric_type: "gauge".into(),
            value: count,
            labels: vec![("source".into(), source)],
        });
    }

    Ok(metrics)
}

/// Register a webhook endpoint.
///
/// The URL must use the `https` scheme (or `http` for localhost in
/// development).  Private/reserved IP ranges are rejected to prevent
/// SSRF attacks.
pub fn register_webhook(conn: &Connection, url: &str, event_filter: &str) -> Result<i64, String> {
    validate_webhook_url(url)?;
    conn.execute(
        "INSERT INTO obs_webhooks (url, event_filter) VALUES (?1, ?2)",
        params![url, event_filter],
    )
    .map_err(|e| format!("db: {e}"))?;
    Ok(conn.last_insert_rowid())
}

/// Validate that a webhook URL is safe to store and later invoke.
fn validate_webhook_url(url: &str) -> Result<(), String> {
    // Length guard
    if url.len() > 2048 {
        return Err("URL too long (max 2048 chars)".into());
    }

    // Must start with https:// (allow http://localhost for dev)
    let lower = url.to_ascii_lowercase();
    let is_https = lower.starts_with("https://");
    let is_localhost_http = lower.starts_with("http://localhost")
        || lower.starts_with("http://127.0.0.1")
        || lower.starts_with("http://[::1]");

    if !is_https && !is_localhost_http {
        return Err("webhook URL must use HTTPS".into());
    }

    // Reject URLs that look like they target private networks
    let host_part = url
        .split("://")
        .nth(1)
        .unwrap_or("")
        .split('/')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("");

    let blocked_prefixes = [
        "10.",
        "192.168.",
        "172.16.",
        "172.17.",
        "172.18.",
        "172.19.",
        "172.20.",
        "172.21.",
        "172.22.",
        "172.23.",
        "172.24.",
        "172.25.",
        "172.26.",
        "172.27.",
        "172.28.",
        "172.29.",
        "172.30.",
        "172.31.",
        "169.254.",
        "0.",
        "metadata.google",
        "metadata.aws",
    ];
    for prefix in &blocked_prefixes {
        if host_part.starts_with(prefix) {
            return Err("webhook URL must not target private networks".into());
        }
    }

    Ok(())
}

/// List active webhooks.
pub fn list_webhooks(conn: &Connection) -> Result<Vec<(i64, String, String)>, rusqlite::Error> {
    let mut stmt =
        conn.prepare("SELECT id, url, event_filter FROM obs_webhooks WHERE active = 1")?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
    rows.collect()
}

/// Remove a webhook.
pub fn remove_webhook(conn: &Connection, webhook_id: i64) -> Result<bool, rusqlite::Error> {
    let n = conn.execute(
        "UPDATE obs_webhooks SET active = 0 WHERE id = ?1",
        params![webhook_id],
    )?;
    Ok(n > 0)
}

/// Build a webhook payload for an event.
pub fn build_payload(event_type: &str, data: serde_json::Value) -> WebhookPayload {
    WebhookPayload {
        event_type: event_type.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> convergio_db::pool::PooledConn {
        let pool = convergio_db::pool::create_memory_pool().unwrap();
        let conn = pool.get().unwrap();
        convergio_db::migration::ensure_registry(&conn).unwrap();
        convergio_db::migration::apply_migrations(
            &conn,
            "observatory",
            &crate::schema::migrations(),
        )
        .unwrap();
        conn
    }

    #[test]
    fn prometheus_exposition_format() {
        let conn = setup_db();
        // Insert some timeline events
        conn.execute(
            "INSERT INTO obs_timeline (source, event_type, actor, summary) \
             VALUES ('system', 'boot', 'daemon', 'Started')",
            [],
        )
        .unwrap();
        let text = prometheus_exposition(&conn).unwrap();
        assert!(text.contains("convergio_timeline_events_total"));
        assert!(text.contains("# HELP"));
        assert!(text.contains("# TYPE"));
    }

    #[test]
    fn webhook_crud() {
        let conn = setup_db();
        let id = register_webhook(&conn, "https://example.com/webhook", "anomaly.*").unwrap();
        assert!(id > 0);

        let hooks = list_webhooks(&conn).unwrap();
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].1, "https://example.com/webhook");

        assert!(remove_webhook(&conn, id).unwrap());
        let hooks = list_webhooks(&conn).unwrap();
        assert_eq!(hooks.len(), 0);
    }

    #[test]
    fn register_webhook_rejects_http() {
        let conn = setup_db();
        let result = register_webhook(&conn, "http://evil.com/steal", "*");
        assert!(result.is_err());
    }

    #[test]
    fn register_webhook_rejects_private_ip() {
        let conn = setup_db();
        let result = register_webhook(&conn, "https://192.168.1.1/hook", "*");
        assert!(result.is_err());
        let result = register_webhook(&conn, "https://10.0.0.1/hook", "*");
        assert!(result.is_err());
    }

    #[test]
    fn register_webhook_allows_localhost_http() {
        let conn = setup_db();
        let result = register_webhook(&conn, "http://localhost:8080/hook", "*");
        assert!(result.is_ok());
    }

    #[test]
    fn build_payload_populates_fields() {
        let p = build_payload(
            "anomaly.cost_spike",
            serde_json::json!({"org": "legal-corp"}),
        );
        assert_eq!(p.event_type, "anomaly.cost_spike");
        assert!(!p.timestamp.is_empty());
    }

    #[test]
    fn prometheus_label_injection_escaped() {
        let conn = setup_db();
        // Insert an event with a source containing injection chars
        conn.execute(
            "INSERT INTO obs_timeline (source, event_type, actor, summary, created_at) \
             VALUES ('evil\"source\ninjected', 'boot', 'daemon', 'test', datetime('now'))",
            [],
        )
        .unwrap();
        let text = prometheus_exposition(&conn).unwrap();
        // The injected newline and quote must be escaped
        assert!(!text.contains("evil\"source"));
        assert!(text.contains("evil\\\"source\\ninjected"));
    }
}
