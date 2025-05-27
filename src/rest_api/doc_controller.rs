// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::get;
use std::sync::Arc;
use utoipa::openapi::OpenApi;
use utoipa_redoc::{Redoc, Servable};

pub struct DocController {
    spec: OpenApi,
}

impl DocController {
    pub fn new(spec: OpenApi) -> Self {
        DocController { spec }
    }

    pub fn router(self: &Arc<Self>) -> Router {
        Router::new()
            .route("/openapi/openapi.json", get(openapi_json))
            .route("/openapi/openapi.yaml", get(openapi_yaml))
            .merge(Redoc::with_url("/openapi", self.spec.clone()))
            .with_state(Arc::clone(self))
    }
}

async fn openapi_json(State(controller): State<Arc<DocController>>) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(controller.spec.to_json().unwrap()))
        .unwrap()
}

async fn openapi_yaml(State(controller): State<Arc<DocController>>) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/yaml")
        .body(Body::from(controller.spec.to_yaml().unwrap()))
        .unwrap()
}
