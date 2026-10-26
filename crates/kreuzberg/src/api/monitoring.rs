//! Runtime monitoring endpoints for the HTTP API.

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    Extension, Json,
    body::Body,
    extract::{MatchedPath, State},
    http::{HeaderMap, HeaderValue, Request, StatusCode, header},
    middleware::Next,
    response::{Html, Response},
};
use chrono::Utc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use super::types::PluginStatus;

pub(super) struct ApiMonitor {
    started_at: Instant,
    endpoints: Mutex<HashMap<String, EndpointMetrics>>,
}

impl ApiMonitor {
    pub(super) fn new() -> Self {
        Self {
            started_at: Instant::now(),
            endpoints: Mutex::default(),
        }
    }

    fn request_started(&self, path: &str) {
        let mut endpoints = self.endpoints.lock();
        let endpoint = endpoints.entry(path.to_string()).or_default();
        endpoint.in_flight = endpoint.in_flight.saturating_add(1);
    }

    fn request_finished(&self, path: &str, status_code: u16, duration: Duration) {
        let mut endpoints = self.endpoints.lock();
        let endpoint = endpoints.entry(path.to_string()).or_default();
        endpoint.in_flight = endpoint.in_flight.saturating_sub(1);
        endpoint.requests_total = endpoint.requests_total.saturating_add(1);
        if status_code >= StatusCode::INTERNAL_SERVER_ERROR.as_u16() {
            endpoint.server_errors_total = endpoint.server_errors_total.saturating_add(1);
        }
        endpoint.last_status_code = Some(status_code);
        endpoint.last_request_at = Some(Utc::now().to_rfc3339());

        let elapsed_ms = duration.as_secs_f64() * 1_000.0;
        endpoint.total_duration_ms += elapsed_ms;
        endpoint.last_response_time_ms = Some(elapsed_ms);
    }

    fn snapshot(&self) -> ApiStatusResponse {
        let endpoints = self.endpoints.lock();
        let mut endpoint_statuses = Vec::with_capacity(endpoints.len());
        let mut requests_total = 0_u64;
        let mut requests_in_flight = 0_usize;
        let mut server_errors_total = 0_u64;

        for (path, metrics) in endpoints.iter() {
            requests_total = requests_total.saturating_add(metrics.requests_total);
            requests_in_flight = requests_in_flight.saturating_add(metrics.in_flight);
            server_errors_total = server_errors_total.saturating_add(metrics.server_errors_total);
            endpoint_statuses.push(EndpointStatus {
                path: path.clone(),
                health: match metrics.last_status_code {
                    Some(code) if code >= StatusCode::INTERNAL_SERVER_ERROR.as_u16() => "unhealthy",
                    Some(_) => "healthy",
                    None => "unknown",
                }
                .to_string(),
                requests_total: metrics.requests_total,
                requests_in_flight: metrics.in_flight,
                server_errors_total: metrics.server_errors_total,
                last_status_code: metrics.last_status_code,
                last_request_at: metrics.last_request_at.clone(),
                last_response_time_ms: metrics.last_response_time_ms,
                average_response_time_ms: (metrics.requests_total > 0)
                    .then(|| metrics.total_duration_ms / metrics.requests_total as f64),
            });
        }

        endpoint_statuses.sort_by(|left, right| left.path.cmp(&right.path));
        let status = if endpoint_statuses.iter().any(|endpoint| endpoint.health == "unhealthy") {
            "degraded"
        } else {
            "healthy"
        };

        ApiStatusResponse {
            status: status.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.started_at.elapsed().as_secs(),
            requests_total,
            requests_in_flight,
            server_errors_total,
            endpoints: endpoint_statuses,
        }
    }
}

#[derive(Default)]
struct EndpointMetrics {
    requests_total: u64,
    requests_in_flight: usize,
    server_errors_total: u64,
    last_status_code: Option<u16>,
    last_request_at: Option<String>,
    last_response_time_ms: Option<f64>,
    total_duration_ms: f64,
}

/// Real-time API request and endpoint health information.
#[cfg_attr(alef, alef(skip))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct ApiStatusResponse {
    /// `degraded` if any route's most recent response was 5xx; otherwise `healthy`.
    pub status: String,
    /// Kreuzberg version.
    pub version: String,
    /// Seconds since this API router was created.
    pub uptime_seconds: u64,
    /// Number of completed requests observed by this router.
    pub requests_total: u64,
    /// Number of requests currently being handled.
    pub requests_in_flight: usize,
    /// Number of completed requests that returned a 5xx response.
    pub server_errors_total: u64,
    /// Request and last-response status for each matched route.
    pub endpoints: Vec<EndpointStatus>,
}

/// Recent request health for one matched API route.
#[cfg_attr(alef, alef(skip))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct EndpointStatus {
    /// Axum route pattern, or `unmatched` for requests handled by the fallback.
    pub path: String,
    /// `unknown` before the first response, `healthy` for the last non-5xx response,
    /// or `unhealthy` when the last response was 5xx.
    pub health: String,
    /// Number of completed requests observed for this route.
    pub requests_total: u64,
    /// Number of requests currently being handled for this route.
    pub requests_in_flight: usize,
    /// Number of 5xx responses observed for this route.
    pub server_errors_total: u64,
    /// HTTP status code from the most recently completed request.
    pub last_status_code: Option<u16>,
    /// RFC 3339 timestamp of the most recently completed request.
    pub last_request_at: Option<String>,
    /// Duration of the most recently completed request, in milliseconds.
    pub last_response_time_ms: Option<f64>,
    /// Mean response duration for completed requests, in milliseconds.
    pub average_response_time_ms: Option<f64>,
}

/// Read-only JSON monitoring endpoint.
#[utoipa::path(
    get,
    path = "/status",
    tag = "health",
    responses(
        (status = 200, description = "Runtime and endpoint status", body = ApiStatusResponse),
    )
)]
pub(super) async fn status_handler(
    Extension(monitor): Extension<Arc<ApiMonitor>>,
) -> (HeaderMap, Json<ApiStatusResponse>) {
    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    (headers, Json(monitor.snapshot()))
}

/// Liveness probe that confirms the HTTP process is responsive.
#[utoipa::path(
    get,
    path = "/health/live",
    tag = "health",
    responses(
        (
            status = 200,
            description = "The server process is alive",
            body = super::types::ProbeResponse
        ),
    )
)]
pub(super) async fn liveness_handler() -> Json<super::types::ProbeResponse> {
    Json(super::types::ProbeResponse {
        status: "alive".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Readiness probe that verifies the document extraction registry is available.
#[utoipa::path(
    get,
    path = "/health/ready",
    tag = "health",
    responses(
        (
            status = 200,
            description = "The server can accept extraction requests",
            body = super::types::ReadinessResponse
        ),
        (
            status = 503,
            description = "No document extractors are registered",
            body = super::types::ReadinessResponse
        ),
    )
)]
pub(super) async fn readiness_handler() -> (StatusCode, Json<super::types::ReadinessResponse>) {
    let initialization_error = crate::extractors::ensure_initialized().err();
    if let Some(error) = &initialization_error {
        tracing::error!("API readiness check could not initialize document extractors: {error}");
    }

    let plugin_status = crate::plugins::startup_validation::PluginHealthStatus::check();
    let ready = initialization_error.is_none() && plugin_status.extractors_count > 0;
    let status = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    let reason = if initialization_error.is_some() {
        Some("extractor_initialization_failed".to_string())
    } else if plugin_status.extractors_count == 0 {
        Some("no_extractors_registered".to_string())
    } else {
        None
    };

    (
        status,
        Json(super::types::ReadinessResponse {
            status: if ready { "ready" } else { "not_ready" }.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            reason,
            plugins: PluginStatus {
                ocr_backends_count: plugin_status.ocr_backends_count,
                ocr_backends: plugin_status.ocr_backends,
                extractors_count: plugin_status.extractors_count,
                post_processors_count: plugin_status.post_processors_count,
            },
        }),
    )
}

/// Embedded dashboard that polls the runtime status endpoint.
#[utoipa::path(
    get,
    path = "/dashboard",
    tag = "health",
    responses(
        (status = 200, description = "Read-only browser dashboard"),
    )
)]
pub(super) async fn dashboard_handler() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

pub(super) async fn track_requests(
    State(monitor): State<Arc<ApiMonitor>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let path = request
        .extensions()
        .get::<MatchedPath>()
        .map(|matched| matched.as_str())
        .unwrap_or("unmatched")
        .to_string();

    monitor.request_started(&path);
    let started_at = Instant::now();
    let response = next.run(request).await;
    monitor.request_finished(&path, response.status().as_u16(), started_at.elapsed());
    response
}

const DASHBOARD_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="color-scheme" content="light dark">
  <title>Kreuzberg API status</title>
  <style>
    :root { color-scheme: light dark; font: 16px/1.5 system-ui, sans-serif; }
    body { max-width: 960px; margin: 2rem auto; padding: 0 1rem; }
    h1 { margin-bottom: .25rem; }
    #updated { color: GrayText; }
    .summary { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 1rem; margin: 1.5rem 0; }
    .card { border: 1px solid GrayText; border-radius: .5rem; padding: 1rem; }
    .value { font-size: 1.5rem; font-weight: 700; }
    .healthy { color: #16803c; }
    .unhealthy { color: #c62828; }
    .unknown { color: GrayText; }
    table { width: 100%; border-collapse: collapse; }
    th, td { text-align: left; padding: .65rem; border-bottom: 1px solid GrayText; }
    @media (max-width: 600px) { body { margin-top: 1rem; } th, td { padding: .4rem; } }
  </style>
</head>
<body>
  <h1>Kreuzberg API status</h1>
  <div id="updated" role="status" aria-live="polite">Connecting to status endpoint…</div>
  <section class="summary" aria-label="API summary">
    <div class="card"><div>Service</div><div class="value" id="service">—</div></div>
    <div class="card"><div>Uptime</div><div class="value" id="uptime">—</div></div>
    <div class="card"><div>Requests</div><div class="value" id="requests">—</div></div>
    <div class="card"><div>In flight</div><div class="value" id="in-flight">—</div></div>
    <div class="card"><div>5xx responses</div><div class="value" id="errors">—</div></div>
  </section>
  <h2>Endpoints</h2>
  <table>
    <thead><tr><th>Route</th><th>Health</th><th>Requests</th><th>In flight</th><th>Last status</th><th>Last response</th></tr></thead>
    <tbody id="endpoints"><tr><td colspan="6">Waiting for the first request…</td></tr></tbody>
  </table>
  <script>
    const text = (id, value) => { document.getElementById(id).textContent = value; };
    const duration = value => value == null ? "—" : `${value.toFixed(1)} ms`;
    async function refresh() {
      try {
        const response = await fetch("/status", { cache: "no-store" });
        if (!response.ok) throw new Error(`Status endpoint returned HTTP ${response.status}`);
        const data = await response.json();
        const service = document.getElementById("service");
        service.textContent = `${data.status} · v${data.version}`;
        service.className = `value ${data.status === "healthy" ? "healthy" : "unhealthy"}`;
        text("uptime", `${data.uptime_seconds}s`);
        text("requests", data.requests_total);
        text("in-flight", data.requests_in_flight);
        text("errors", data.server_errors_total);
        const rows = document.getElementById("endpoints");
        rows.replaceChildren();
        for (const endpoint of data.endpoints) {
          const row = document.createElement("tr");
          const values = [
            [endpoint.path, ""],
            [endpoint.health, endpoint.health],
            [endpoint.requests_total, ""],
            [endpoint.requests_in_flight, ""],
            [endpoint.last_status_code ?? "—", ""],
            [duration(endpoint.last_response_time_ms), ""]
          ];
          for (const [value, className] of values) {
            const cell = document.createElement("td");
            cell.textContent = value;
            if (className) cell.className = className;
            row.append(cell);
          }
          rows.append(row);
        }
        text("updated", `Updated ${new Date().toLocaleTimeString()} · refreshes every 2 seconds`);
      } catch (error) {
        text("service", "unavailable");
        text("updated", `Monitoring unavailable: ${error.message}`);
      }
    }
    refresh();
    window.setInterval(refresh, 2000);
  </script>
</body>
</html>"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_snapshot_aggregates_endpoint_metrics() {
        let monitor = ApiMonitor::new();
        monitor.request_started("/extract");
        monitor.request_finished("/extract", StatusCode::OK.as_u16(), Duration::from_millis(12));
        monitor.request_started("/extract");
        monitor.request_finished(
            "/extract",
            StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            Duration::from_millis(28),
        );

        let snapshot = monitor.snapshot();
        assert_eq!(snapshot.status, "degraded");
        assert_eq!(snapshot.requests_total, 2);
        assert_eq!(snapshot.requests_in_flight, 0);
        assert_eq!(snapshot.server_errors_total, 1);
        assert_eq!(snapshot.endpoints.len(), 1);
        assert_eq!(snapshot.endpoints[0].path, "/extract");
        assert_eq!(snapshot.endpoints[0].health, "unhealthy");
        assert_eq!(snapshot.endpoints[0].requests_total, 2);
        assert_eq!(snapshot.endpoints[0].server_errors_total, 1);
        assert!((snapshot.endpoints[0].average_response_time_ms.unwrap() - 20.0).abs() < 0.01);
    }

    #[test]
    fn status_snapshot_reports_in_flight_requests() {
        let monitor = ApiMonitor::new();
        monitor.request_started("/extract");

        let snapshot = monitor.snapshot();
        assert_eq!(snapshot.requests_total, 0);
        assert_eq!(snapshot.requests_in_flight, 1);
        assert_eq!(snapshot.endpoints[0].health, "unknown");
    }

    #[test]
    fn dashboard_uses_status_endpoint_and_safe_text_rendering() {
        assert!(DASHBOARD_HTML.contains("fetch(\"/status\""));
        assert!(DASHBOARD_HTML.contains("textContent"));
        assert!(DASHBOARD_HTML.contains("window.setInterval(refresh, 2000)"));
    }
}
