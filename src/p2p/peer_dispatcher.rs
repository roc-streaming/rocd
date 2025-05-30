// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::*;

pub struct PeerDispatcher {
    // TODO
}

impl PeerDispatcher {
    pub fn new() -> Self {
        PeerDispatcher {}
    }

    pub async fn self_uid(&self) -> Uid {
        Uid::parse("777777-888888-999999").unwrap()
    }

    pub async fn get_all(&self) -> Vec<PeerSpec> {
        vec![self.get_peer(&Uid::parse("777777-888888-999999").unwrap()).await]
    }

    pub async fn get_peer(&self, peer_uid: &Uid) -> PeerSpec {
        PeerSpec { peer_uri: Uri::from_peer(peer_uid), peer_uid: *peer_uid }
    }
}
