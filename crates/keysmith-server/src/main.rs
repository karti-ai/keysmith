use std::{env, net::SocketAddr, path::PathBuf, sync::Arc};

use anyhow::Result;
use axum::{
    Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get,
    routing::post,
};
use keysmith_core::{
    ConfigurationSnapshot, Inspection, KeysmithProbe, MutationPlan, PlanInspection,
    inspect_connected, inspect_plan as verify_plan, probe_connected,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

#[derive(Clone, Default)]
struct AppState {
    inspection_lock: Arc<Mutex<()>>,
}

#[derive(Serialize)]
struct ApiError {
    error: String,
}

#[derive(Deserialize, Serialize)]
struct PlanPreviewRequest {
    baseline: ConfigurationSnapshot,
    target: ConfigurationSnapshot,
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState::default();
    let web_dir = env::var_os("KEYSMITH_WEB_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("apps/web/dist"));
    let web_dir_display = web_dir.display().to_string();
    let index = web_dir.join("index.html");
    let app = build_app(state, web_dir, index);

    let address = SocketAddr::from(([127, 0, 0, 1], 3762));
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!(
        "Keysmith listening on http://{address} (web root: {})",
        web_dir_display
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

fn build_app(state: AppState, web_dir: PathBuf, index: PathBuf) -> Router {
    let web = ServeDir::new(&web_dir).not_found_service(ServeFile::new(index));
    Router::new()
        .route("/api/health", get(health))
        .route("/api/inspect", get(inspect))
        .route("/api/config/snapshot", get(config_snapshot))
        .route("/api/firmware/probe", get(firmware_probe))
        .route("/api/plans/preview", post(plan_preview))
        .route("/api/plans/inspect", post(plan_inspect))
        .fallback_service(web)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn firmware_probe(
    State(state): State<AppState>,
) -> Result<Json<KeysmithProbe>, impl IntoResponse> {
    let _guard = state.inspection_lock.lock().await;
    probe_connected().map(Json).map_err(|error| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiError {
                error: error.to_string(),
            }),
        )
    })
}

async fn health() -> &'static str {
    "ok"
}

async fn inspect(State(state): State<AppState>) -> Result<Json<Inspection>, impl IntoResponse> {
    let _guard = state.inspection_lock.lock().await;
    inspect_connected().map(Json).map_err(|error| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiError {
                error: error.to_string(),
            }),
        )
    })
}

async fn config_snapshot(
    State(state): State<AppState>,
) -> Result<Json<ConfigurationSnapshot>, impl IntoResponse> {
    let _guard = state.inspection_lock.lock().await;
    inspect_connected()
        .map(|inspection| Json(ConfigurationSnapshot::from_inspection(&inspection)))
        .map_err(|error| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiError {
                    error: error.to_string(),
                }),
            )
        })
}

async fn plan_preview(
    Json(request): Json<PlanPreviewRequest>,
) -> Result<Json<MutationPlan>, (StatusCode, Json<ApiError>)> {
    MutationPlan::create(request.baseline, request.target)
        .map(Json)
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: error.to_string(),
                }),
            )
        })
}

async fn plan_inspect(Json(plan): Json<MutationPlan>) -> Json<PlanInspection> {
    Json(verify_plan(&plan))
}

async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use keysmith_core::{
        CONFIG_SNAPSHOT_SCHEMA_VERSION, KEYCHRON_VENDOR_ID, KeyboardConfiguration,
        Q3_MAX_ANSI_PRODUCT_ID, SnapshotDebounceInfo, SnapshotDevice, SnapshotEncoderBinding,
        SnapshotKeymapLayer, SnapshotMacroInfo, SnapshotRgbInfo, SnapshotSnapClickInfo,
        SnapshotWirelessPower,
    };
    use tower::ServiceExt;

    use super::*;

    fn snapshot() -> ConfigurationSnapshot {
        ConfigurationSnapshot {
            schema_version: CONFIG_SNAPSHOT_SCHEMA_VERSION,
            device: SnapshotDevice {
                name: "Keychron Q3 Max".to_owned(),
                layout: "ANSI encoder".to_owned(),
                vendor_id: KEYCHRON_VENDOR_ID,
                product_id: Q3_MAX_ANSI_PRODUCT_ID,
                firmware: "v1.1.1 test".to_owned(),
                via_protocol: 12,
                keychron_protocol: 2,
                qmk_command_set: 2,
            },
            configuration: KeyboardConfiguration {
                active_default_layer: 0,
                layers: (0..4)
                    .map(|index| SnapshotKeymapLayer {
                        index,
                        name: format!("Layer {index}"),
                        matrix: vec![vec![0; 17]; 6],
                    })
                    .collect(),
                macros: SnapshotMacroInfo {
                    slots: 16,
                    buffer_bytes: 1698,
                    used_bytes: 0,
                },
                snap_click: SnapshotSnapClickInfo {
                    pair_capacity: 20,
                    configured_pairs: 0,
                },
                wireless_power: SnapshotWirelessPower {
                    backlight_timeout_seconds: 600,
                    sleep_timeout_seconds: 7200,
                },
                debounce: SnapshotDebounceInfo {
                    algorithm_id: 4,
                    algorithm: "symmetric eager, per key".to_owned(),
                    time_ms: 50,
                },
                rgb: SnapshotRgbInfo {
                    brightness: 255,
                    effect: 5,
                    speed: 127,
                    hue: 0,
                    saturation: 255,
                },
                encoders: (0..4)
                    .map(|layer| SnapshotEncoderBinding {
                        layer,
                        counter_clockwise: 0x80,
                        clockwise: 0x81,
                    })
                    .collect(),
            },
        }
    }

    fn test_app() -> Router {
        let missing = PathBuf::from("/tmp/keysmith-server-test-web-does-not-exist");
        build_app(
            AppState::default(),
            missing.clone(),
            missing.join("index.html"),
        )
    }

    #[tokio::test]
    async fn preview_endpoint_returns_a_non_executable_deterministic_plan() {
        let baseline = snapshot();
        let mut target = baseline.clone();
        target.configuration.rgb.brightness = 42;
        let body = serde_json::to_vec(&PlanPreviewRequest { baseline, target }).unwrap();
        let request = Request::builder()
            .method("POST")
            .uri("/api/plans/preview")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = test_app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("content-type")
                .unwrap()
                .to_str()
                .unwrap(),
            "application/json"
        );
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let plan: MutationPlan = serde_json::from_slice(&body).unwrap();
        assert!(!plan.executable());
        assert_eq!(plan.diff().changes.len(), 1);
        assert!(plan.plan_id().starts_with("ksplan_v1_"));
        assert!(verify_plan(&plan).valid);
    }

    #[tokio::test]
    async fn no_apply_endpoint_exists() {
        let request = Request::builder()
            .method("POST")
            .uri("/api/plans/apply")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap();
        let response = test_app().oneshot(request).await.unwrap();
        assert!(!response.status().is_success());
    }

    #[tokio::test]
    async fn api_does_not_grant_cross_origin_read_access() {
        let request = Request::builder()
            .method("GET")
            .uri("/api/health")
            .header("origin", "https://untrusted.example")
            .body(Body::empty())
            .unwrap();
        let response = test_app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response
                .headers()
                .get("access-control-allow-origin")
                .is_none()
        );
    }
}
