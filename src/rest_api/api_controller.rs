// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::*;
use crate::io_endpoint::EndpointDispatcher;
use crate::io_stream::StreamDispatcher;

use axum::Router;
use axum::extract::{Extension, Json, Path};
use axum::http::StatusCode;
use std::sync::Arc;
use utoipa::openapi::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub struct ApiController {
    endpoint_dispatcher: Arc<EndpointDispatcher>,
    stream_dispatcher: Arc<StreamDispatcher>,
}

impl ApiController {
    pub fn new(
        endpoint_dispatcher: Arc<EndpointDispatcher>, stream_dispatcher: Arc<StreamDispatcher>,
    ) -> Self {
        ApiController { endpoint_dispatcher, stream_dispatcher }
    }

    pub fn spec() -> OpenApi {
        ApiController::build().into_openapi()
    }

    pub fn router_with_spec(self: &Arc<Self>) -> (Router, OpenApi) {
        let ext = Extension(Arc::clone(self));

        ApiController::build().layer(ext).split_for_parts()
    }

    fn build() -> OpenApiRouter {
        OpenApiRouter::new()
            // endpoints
            .routes(routes!(list_endpoints))
            .routes(routes!(read_endpoint))
            .routes(routes!(update_endpoint))
            // streams
            .routes(routes!(list_streams))
            .routes(routes!(read_stream))
            .routes(routes!(update_stream))
    }
}

// endpoints

#[utoipa::path(
    get,
    path = "/peers/{peer_uid}/endpoints",
    responses(
        (status = 200, description = "Success", body = [EndpointSpec]),
    )
)]
async fn list_endpoints(
    Extension(controller): Extension<Arc<ApiController>>, Path(peer_uid): Path<String>,
) -> (StatusCode, Json<Vec<EndpointSpec>>) {
    (StatusCode::OK, Json(controller.endpoint_dispatcher.get_all(&peer_uid).await))
}

#[utoipa::path(
    get,
    path = "/peers/{peer_uid}/endpoints/{endpoint_uid}",
    responses(
        (status = 200, description = "Success", body = EndpointSpec),
    )
)]
async fn read_endpoint(
    Extension(controller): Extension<Arc<ApiController>>,
    Path((peer_uid, endpoint_uid)): Path<(String, String)>,
) -> (StatusCode, Json<EndpointSpec>) {
    (
        StatusCode::OK,
        Json(controller.endpoint_dispatcher.get_endpoint(&peer_uid, &endpoint_uid).await),
    )
}

#[utoipa::path(
    put,
    path = "/peers/{peer_uid}/endpoints/{endpoint_uid}",
    responses(
        (status = 200, description = "Success", body = EndpointSpec),
    )
)]
async fn update_endpoint(
    Extension(controller): Extension<Arc<ApiController>>,
    Path((peer_uid, endpoint_uid)): Path<(String, String)>,
) -> (StatusCode, Json<EndpointSpec>) {
    (
        StatusCode::OK,
        Json(controller.endpoint_dispatcher.get_endpoint(&peer_uid, &endpoint_uid).await),
    )
}

// streams

#[utoipa::path(
    get,
    path = "/streams",
    responses(
        (status = 200, description = "Success", body = [StreamSpec]),
    )
)]
async fn list_streams(
    Extension(controller): Extension<Arc<ApiController>>,
) -> (StatusCode, Json<Vec<StreamSpec>>) {
    (StatusCode::OK, Json(controller.stream_dispatcher.get_all().await))
}

#[utoipa::path(
    get,
    path = "/streams/{stream_uid}",
    responses(
        (status = 200, description = "Success", body = StreamSpec),
    )
)]
async fn read_stream(
    Extension(controller): Extension<Arc<ApiController>>, Path(stream_uid): Path<String>,
) -> (StatusCode, Json<StreamSpec>) {
    (StatusCode::OK, Json(controller.stream_dispatcher.get_stream(&stream_uid).await))
}

#[utoipa::path(
    put,
    path = "/streams/{stream_uid}",
    responses(
        (status = 200, description = "Success", body = StreamSpec),
    )
)]
async fn update_stream(
    Extension(controller): Extension<Arc<ApiController>>, Path(stream_uid): Path<String>,
) -> (StatusCode, Json<StreamSpec>) {
    (StatusCode::OK, Json(controller.stream_dispatcher.get_stream(&stream_uid).await))
}
