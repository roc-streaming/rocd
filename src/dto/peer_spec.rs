// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::uid::*;
use crate::dto::uri::*;
use crate::dto::validate::*;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
pub struct PeerSpec {
    #[schema(value_type = String)]
    pub peer_uri: Uri,

    #[schema(value_type = String)]
    pub peer_uid: Uid,
}

impl Validate for PeerSpec {
    fn validate(&self) -> ValidationResult {
        if self.peer_uri.kind() != UriKind::Peer {
            return Err(ValidationError::LayoutError("unexpected peer_uri format".into()));
        }

        if self.peer_uri.peer_uid().unwrap() != self.peer_uid {
            return Err(ValidationError::LayoutError(
                "UID mismatch in peer_uri and peer_uid fields".into(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertables::*;

    #[test]
    fn test_validate() {
        let peer_uid = Uid::generate_random();

        let good_spec = PeerSpec { peer_uri: Uri::from_peer(&peer_uid), peer_uid: peer_uid };

        assert_ok!(good_spec.validate());

        let bad_specs = vec![
            // invalid peer_uri type
            {
                let mut spec = good_spec.clone();
                spec.peer_uri = Uri::from_stream(&peer_uid);
                spec
            },
            // UID mismatch in peer_uri and peer_uid
            {
                let mut spec = good_spec.clone();
                spec.peer_uid = Uid::generate_random();
                spec
            },
        ];

        for spec in &bad_specs {
            assert_matches!(spec.validate(), Err(ValidationError::LayoutError(_)));
        }
    }
}
