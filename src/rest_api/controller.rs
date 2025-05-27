// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::*;
use crate::io_endpoint::EndpointDispatcher;
use crate::io_stream::StreamDispatcher;

use axum::extract::{Extension, Json, Path};
use axum::http::StatusCode;
use std::sync::Arc;
use utoipa::openapi::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub struct RestController {
    endpoint_dispatcher: Arc<EndpointDispatcher>,
    stream_dispatcher: Arc<StreamDispatcher>,
}

impl RestController {
    /// Create controller.
    pub fn new(
        endpoint_dispatcher: Arc<EndpointDispatcher>, stream_dispatcher: Arc<StreamDispatcher>,
    ) -> Self {
        RestController { endpoint_dispatcher, stream_dispatcher }
    }

    /// Get api spec.
    pub fn openapi() -> OpenApi {
        RestController::build().into_openapi()
    }

    /// Construct http router for this controller.
    pub fn router(self: &Arc<Self>) -> OpenApiRouter {
        // we use Extension extractor instead of State to be able to construct
        // dummy (state-less) router in openapi() method
        RestController::build().layer(Extension(Arc::clone(self)))
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
    path = "/peers/{peer_uuid}/endpoints",
    responses(
        (status = 200, description = "Success", body = [EndpointSpec]),
    )
)]
async fn list_endpoints(
    Extension(controller): Extension<Arc<RestController>>, Path(peer_uuid): Path<String>,
) -> (StatusCode, Json<Vec<EndpointSpec>>) {
    (StatusCode::OK, Json(controller.endpoint_dispatcher.get_all(&peer_uuid).await))
}

#[utoipa::path(
    get,
    path = "/peers/{peer_uuid}/endpoints/{endpoint_uuid}",
    responses(
        (status = 200, description = "Success", body = EndpointSpec),
    )
)]
async fn read_endpoint(
    Extension(controller): Extension<Arc<RestController>>,
    Path((peer_uuid, endpoint_uuid)): Path<(String, String)>,
) -> (StatusCode, Json<EndpointSpec>) {
    (
        StatusCode::OK,
        Json(controller.endpoint_dispatcher.get_endpoint(&peer_uuid, &endpoint_uuid).await),
    )
}

#[utoipa::path(
    put,
    path = "/peers/{peer_uuid}/endpoints/{endpoint_uuid}",
    responses(
        (status = 200, description = "Success", body = EndpointSpec),
    )
)]
async fn update_endpoint(
    Extension(controller): Extension<Arc<RestController>>,
    Path((peer_uuid, endpoint_uuid)): Path<(String, String)>,
) -> (StatusCode, Json<EndpointSpec>) {
    (
        StatusCode::OK,
        Json(controller.endpoint_dispatcher.get_endpoint(&peer_uuid, &endpoint_uuid).await),
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
    Extension(controller): Extension<Arc<RestController>>,
) -> (StatusCode, Json<Vec<StreamSpec>>) {
    (StatusCode::OK, Json(controller.stream_dispatcher.get_all().await))
}

#[utoipa::path(
    get,
    path = "/streams/{stream_uuid}",
    responses(
        (status = 200, description = "Success", body = StreamSpec),
    )
)]
async fn read_stream(
    Extension(controller): Extension<Arc<RestController>>, Path(stream_uuid): Path<String>,
) -> (StatusCode, Json<StreamSpec>) {
    (StatusCode::OK, Json(controller.stream_dispatcher.get_stream(&stream_uuid).await))
}

#[utoipa::path(
    put,
    path = "/streams/{stream_uuid}",
    responses(
        (status = 200, description = "Success", body = StreamSpec),
    )
)]
async fn update_stream(
    Extension(controller): Extension<Arc<RestController>>, Path(stream_uuid): Path<String>,
) -> (StatusCode, Json<StreamSpec>) {
    (StatusCode::OK, Json(controller.stream_dispatcher.get_stream(&stream_uuid).await))
}
