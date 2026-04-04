use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use axum::{Json, Router, extract::State, routing::get};
use parking_lot::RwLock;
use serde::Serialize;

use crate::context::AppContext;

#[derive(Debug, Clone, Default)]
pub struct Metrics {
    http_requests_total: Arc<AtomicU64>,
    socket_events_total: Arc<AtomicU64>,
    http_requests_by_route: Arc<RwLock<BTreeMap<(String, String, u16), u64>>>,
    socket_events_by_type: Arc<RwLock<BTreeMap<String, u64>>>,
    websocket_connections: Arc<RwLock<BTreeMap<String, i64>>>,
}

impl Metrics {
    pub fn incr_http_request(&self, method: &str, route: &str, status: u16) {
        self.http_requests_total.fetch_add(1, Ordering::Relaxed);
        *self
            .http_requests_by_route
            .write()
            .entry((method.to_string(), route.to_string(), status))
            .or_default() += 1;
    }

    pub fn incr_socket_event(&self, event_type: &str) {
        self.socket_events_total.fetch_add(1, Ordering::Relaxed);
        *self
            .socket_events_by_type
            .write()
            .entry(event_type.to_string())
            .or_default() += 1;
    }

    pub fn connection_opened(&self, connection_type: &str) {
        *self
            .websocket_connections
            .write()
            .entry(connection_type.to_string())
            .or_default() += 1;
    }

    pub fn connection_closed(&self, connection_type: &str) {
        let mut guard = self.websocket_connections.write();
        let entry = guard.entry(connection_type.to_string()).or_default();
        *entry = (*entry - 1).max(0);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            http_requests_total: self.http_requests_total.load(Ordering::Relaxed),
            socket_events_total: self.socket_events_total.load(Ordering::Relaxed),
        }
    }

    pub fn render_prometheus(&self) -> String {
        let mut output = String::new();
        output.push_str("# TYPE http_requests_total counter\n");
        output.push_str(&format!(
            "http_requests_total {}\n",
            self.http_requests_total.load(Ordering::Relaxed)
        ));
        output.push_str("# TYPE socket_events_total counter\n");
        output.push_str(&format!(
            "socket_events_total {}\n",
            self.socket_events_total.load(Ordering::Relaxed)
        ));
        output.push_str("# TYPE vibe_http_requests_by_route_total counter\n");
        for ((method, route, status), count) in self.http_requests_by_route.read().iter() {
            output.push_str(&format!(
                "vibe_http_requests_by_route_total{{method=\"{}\",route=\"{}\",status=\"{}\"}} {}\n",
                escape_label(method),
                escape_label(route),
                status,
                count
            ));
        }
        output.push_str("# TYPE websocket_events_by_type_total counter\n");
        for (event_type, count) in self.socket_events_by_type.read().iter() {
            output.push_str(&format!(
                "websocket_events_by_type_total{{event_type=\"{}\"}} {}\n",
                escape_label(event_type),
                count
            ));
        }
        output.push_str("# TYPE websocket_connections_total gauge\n");
        for (connection_type, count) in self.websocket_connections.read().iter() {
            output.push_str(&format!(
                "websocket_connections_total{{type=\"{}\"}} {}\n",
                escape_label(connection_type),
                count
            ));
        }
        output
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct MetricsSnapshot {
    pub http_requests_total: u64,
    pub socket_events_total: u64,
}

pub fn routes() -> Router<AppContext> {
    Router::new()
        .route("/metrics", get(metrics))
        .route("/health", get(health))
}

async fn metrics(State(ctx): State<AppContext>) -> String {
    ctx.metrics().render_prometheus()
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

#[derive(Debug, Clone, Serialize)]
struct HealthResponse {
    status: &'static str,
    timestamp: String,
}

fn escape_label(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::Metrics;

    #[test]
    fn metrics_snapshot_tracks_counters() {
        let metrics = Metrics::default();
        metrics.incr_http_request("GET", "/v1/feed", 200);
        metrics.incr_socket_event("message");
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.http_requests_total, 1);
        assert_eq!(snapshot.socket_events_total, 1);
        let text = metrics.render_prometheus();
        assert!(text.contains("http_requests_total 1"));
        assert!(text.contains("websocket_events_by_type_total{event_type=\"message\"} 1"));
    }
}
