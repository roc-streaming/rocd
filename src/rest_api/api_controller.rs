// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::*;
use crate::io_endpoints::EndpointDispatcher;
use crate::io_streams::StreamDispatcher;
use crate::p2p::PeerDispatcher;
use crate::rest_api::error::*;

use axum::Router;
use axum::extract::{Extension, Json, Path};
use std::result;
use std::sync::Arc;
use utoipa::OpenApi as _;
use utoipa::openapi::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub struct ApiController {
    peer_dispatcher: Arc<PeerDispatcher>,
    endpoint_dispatcher: Arc<EndpointDispatcher>,
    stream_dispatcher: Arc<StreamDispatcher>,
}

impl ApiController {
    pub fn new(
        peer_dispatcher: Arc<PeerDispatcher>, endpoint_dispatcher: Arc<EndpointDispatcher>,
        stream_dispatcher: Arc<StreamDispatcher>,
    ) -> Self {
        ApiController { peer_dispatcher, endpoint_dispatcher, stream_dispatcher }
    }

    pub fn spec() -> OpenApi {
        ApiController::build().into_openapi()
    }

    pub fn router_with_spec(self: &Arc<Self>) -> (Router, OpenApi) {
        let ext = Extension(Arc::clone(self));

        ApiController::build().layer(ext).split_for_parts()
    }

    fn build() -> OpenApiRouter {
        OpenApiRouter::with_openapi(ApiDoc::openapi())
            // peers
            .routes(routes!(list_peers))
            .routes(routes!(read_peer))
            .routes(routes!(update_peer))
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

#[derive(utoipa::OpenApi)]
#[openapi(info(title = "rocd REST API",))]
struct ApiDoc;

type Result<T> = result::Result<T, HandlerError>;

// peers

#[utoipa::path(
    get,
    path = "/peers",
    responses(
        (status = 200, description = "Success", body = [PeerSpec]),
    )
)]
async fn list_peers(
    Extension(controller): Extension<Arc<ApiController>>,
) -> Result<Json<Vec<PeerSpec>>> {
    Ok(Json(controller.peer_dispatcher.get_all().await))
}

#[utoipa::path(
    get,
    path = "/peers/{peer_uid}",
    responses(
        (status = 200, description = "Success", body = PeerSpec),
    )
)]
async fn read_peer(
    Extension(controller): Extension<Arc<ApiController>>, Path(peer_uid): Path<String>,
) -> Result<Json<PeerSpec>> {
    let peer_uid = if peer_uid == "self" {
        controller.peer_dispatcher.self_uid().await
    } else {
        Uid::parse(&peer_uid)?
    };

    Ok(Json(controller.peer_dispatcher.get_peer(&peer_uid).await))
}

#[utoipa::path(
    put,
    path = "/peers/{peer_uid}",
    responses(
        (status = 200, description = "Success", body = PeerSpec),
    )
)]
async fn update_peer(
    Extension(controller): Extension<Arc<ApiController>>, Path(peer_uid): Path<String>,
) -> Result<Json<PeerSpec>> {
    let peer_uid = if peer_uid == "self" {
        controller.peer_dispatcher.self_uid().await
    } else {
        Uid::parse(&peer_uid)?
    };

    Ok(Json(controller.peer_dispatcher.get_peer(&peer_uid).await))
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
) -> Result<Json<Vec<EndpointSpec>>> {
    let peer_uid = if peer_uid == "self" {
        controller.peer_dispatcher.self_uid().await
    } else {
        Uid::parse(&peer_uid)?
    };

    Ok(Json(controller.endpoint_dispatcher.get_all(&peer_uid).await))
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
) -> Result<Json<EndpointSpec>> {
    let peer_uid = if peer_uid == "self" {
        controller.peer_dispatcher.self_uid().await
    } else {
        Uid::parse(&peer_uid)?
    };

    let endpoint_uid = Uid::parse(&endpoint_uid)?;

    Ok(Json(controller.endpoint_dispatcher.get_endpoint(&peer_uid, &endpoint_uid).await))
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
) -> Result<Json<EndpointSpec>> {
    let peer_uid = if peer_uid == "self" {
        controller.peer_dispatcher.self_uid().await
    } else {
        Uid::parse(&peer_uid)?
    };

    let endpoint_uid = Uid::parse(&endpoint_uid)?;

    Ok(Json(controller.endpoint_dispatcher.get_endpoint(&peer_uid, &endpoint_uid).await))
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
) -> Result<Json<Vec<StreamSpec>>> {
    Ok(Json(controller.stream_dispatcher.get_all().await))
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
) -> Result<Json<StreamSpec>> {
    let stream_uid = Uid::parse(&stream_uid)?;

    Ok(Json(controller.stream_dispatcher.get_stream(&stream_uid).await))
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
) -> Result<Json<StreamSpec>> {
    let stream_uid = Uid::parse(&stream_uid)?;

    Ok(Json(controller.stream_dispatcher.get_stream(&stream_uid).await))
}
