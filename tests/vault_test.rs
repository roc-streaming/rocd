// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use rocd::dto::*;
use rocd::vault::*;

use assertables::*;
use ctor::ctor;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use tracing_test::traced_test;

#[ctor]
fn setup_tests() {
    // for procspawn::spawn() used in some tests
    procspawn::init();
}

// Create and returns temp dir.
// TempDir destructor will delete the dir.
fn make_temp_dir() -> TempDir {
    TempDir::with_prefix("rocd_test").unwrap()
}

// Creates temp dir and opens vault.
async fn make_temp_vault() -> (TempDir, Vault) {
    let temp_dir = make_temp_dir();

    let vault = Vault::open(
        &VaultConfigBuilder::default()
            .db_path(temp_dir.path().join("test.db").to_str().unwrap())
            .build()
            .unwrap(),
    )
    .await
    .unwrap();

    // Note: tuples elements are destroyed from right to left, so
    // when the tuple is dropped, vault will be destroyed first,
    // then temp dir will be removed.
    (temp_dir, vault)
}

fn make_uid<S: ToString>(name: S) -> Uid {
    Uid::generate_reproducible(UidTag::Unspecified, &name.to_string())
}

fn make_peer_spec(peer_uid: &Uid) -> Arc<PeerSpec> {
    Arc::new(PeerSpec { peer_uri: Uri::from_peer(peer_uid), peer_uid: *peer_uid })
}

fn make_endpoint_spec<S: ToString>(endpoint_uid: &Uid, endpoint_name: S) -> Arc<EndpointSpec> {
    let peer_uid = make_uid("test_peer");
    let endpoint_name = endpoint_name.to_string();

    Arc::new(EndpointSpec {
        endpoint_uri: Uri::from_endpoint(&peer_uid, &endpoint_uid),
        endpoint_uid: *endpoint_uid,
        endpoint_type: EndpointType::SystemDevice,
        stream_direction: EndpointDir::Output,
        driver: DriverId::Pipewire,
        display_name: "Test Name".into(),
        system_name: endpoint_name,
    })
}

fn make_stream_spec(stream_uid: &Uid) -> Arc<StreamSpec> {
    let peer_uid = make_uid("test_peer");
    let endpoint_uid = make_uid("test_endpoint");

    Arc::new(StreamSpec {
        stream_uri: Uri::from_stream(stream_uid),
        stream_uid: *stream_uid,
        source: ConnectionSpec::Endpoint {
            connection_type: ConnectionType::Endpoint,
            endpoint_uri: Uri::from_endpoint(&peer_uid, &endpoint_uid),
        },
        destination: ConnectionSpec::External {
            connection_type: ConnectionType::External,
            media_uri: Uri::parse("rtp+rs8m://192.168.0.101:10000").unwrap(),
            repair_uri: Uri::parse("rs8m://192.168.0.101:10001").unwrap(),
            control_uri: Uri::parse("rtcp://192.168.0.101:10002").unwrap(),
        },
    })
}

// Parent directories are not automatically created on open.
#[tokio::test]
async fn test_open_bad_path() {
    let result = Vault::open(
        &VaultConfigBuilder::default()
            .db_path("/very/bad/non/existent/path/test.db")
            .build()
            .unwrap(),
    )
    .await;
    // can't open: path doesn't exist
    assert_matches!(&result, Err(VaultError::DatabaseError(_)));
}

// If directories exist, DB file is automatically created on open.
#[tokio::test]
async fn test_open_create() {
    let temp_dir = make_temp_dir();

    assert!(!temp_dir.path().join("test.db").is_file());

    let result = Vault::open(
        &VaultConfigBuilder::default()
            .db_path(temp_dir.path().join("test.db").to_str().unwrap())
            .build()
            .unwrap(),
    )
    .await;
    assert_ok!(&result);

    assert!(temp_dir.path().join("test.db").is_file());
}

#[tokio::test]
async fn test_open_existing() {
    let temp_dir = make_temp_dir();

    assert!(!temp_dir.path().join("test.db").is_file());

    {
        let result = Vault::open(
            &VaultConfigBuilder::default()
                .db_path(temp_dir.path().join("test.db").to_str().unwrap())
                .build()
                .unwrap(),
        )
        .await;
        assert_ok!(&result);
    }

    assert!(temp_dir.path().join("test.db").is_file());

    {
        let result = Vault::open(
            &VaultConfigBuilder::default()
                .db_path(temp_dir.path().join("test.db").to_str().unwrap())
                .build()
                .unwrap(),
        )
        .await;

        assert_ok!(&result);
    }

    assert!(temp_dir.path().join("test.db").is_file());
}

// Can't open same DB concurrently.
#[tokio::test]
async fn test_open_locked() {
    let temp_dir = make_temp_dir();

    let result = Vault::open(
        &VaultConfigBuilder::default()
            .db_path(temp_dir.path().join("test.db").to_str().unwrap())
            .build()
            .unwrap(),
    )
    .await;
    assert_ok!(&result);

    {
        assert!(temp_dir.path().join("test.db").is_file());

        let result = Vault::open(
            &VaultConfigBuilder::default()
                .db_path(temp_dir.path().join("test.db").to_str().unwrap())
                .build()
                .unwrap(),
        )
        .await;
        // can't open: db is locked
        assert_matches!(&result, Err(VaultError::DatabaseError(_)));
    }
}

// Can't open same DB concurrently, even from separate processes.
#[tokio::test]
async fn test_open_locked_two_processes() {
    let temp_dir = make_temp_dir();
    let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();

    let result =
        Vault::open(&VaultConfigBuilder::default().db_path(db_path.clone()).build().unwrap())
            .await;
    assert_ok!(&result);

    // run closure in a new process
    procspawn::spawn(db_path, |db_path| -> () {
        assert!(Path::new(&db_path).is_file());

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let result =
                Vault::open(&VaultConfigBuilder::default().db_path(db_path).build().unwrap())
                    .await;
            // can't open: db is locked (from another process)
            assert_matches!(&result, Err(VaultError::DatabaseError(_)));
        });
    })
    .join()
    .unwrap();
}

#[tokio::test]
async fn test_read_write() {
    let (_temp_dir, vault) = make_temp_vault().await;

    let uid_1 = make_uid("uid_1");
    let uid_2 = make_uid("uid_2");

    assert_matches!(vault.read_endpoint(&uid_1).await, Err(VaultError::UidNotFound(_)));
    assert_matches!(vault.read_endpoint(&uid_2).await, Err(VaultError::UidNotFound(_)));

    let endpoint_1 = make_endpoint_spec(&uid_1, "name_1");
    let endpoint_2 = make_endpoint_spec(&uid_2, "name_2");
    assert!(*endpoint_1 != *endpoint_2);

    assert_ok!(vault.write_endpoint(&endpoint_1).await);
    assert_eq!(*vault.read_endpoint(&uid_1).await.unwrap(), *endpoint_1);
    assert_matches!(vault.read_endpoint(&uid_2).await, Err(VaultError::UidNotFound(_)));

    assert_ok!(vault.write_endpoint(&endpoint_2).await);
    assert_eq!(*vault.read_endpoint(&uid_1).await.unwrap(), *endpoint_1);
    assert_eq!(*vault.read_endpoint(&uid_2).await.unwrap(), *endpoint_2);
}

#[tokio::test]
async fn test_overwrite() {
    let (_temp_dir, vault) = make_temp_vault().await;

    let uid_1 = make_uid("uid_1");
    let uid_2 = make_uid("uid_2");

    assert_matches!(vault.read_endpoint(&uid_1).await, Err(VaultError::UidNotFound(_)));
    assert_matches!(vault.read_endpoint(&uid_2).await, Err(VaultError::UidNotFound(_)));

    let endpoint_1_a = make_endpoint_spec(&uid_1, "name_1_a");
    let endpoint_1_b = make_endpoint_spec(&uid_1, "name_1_b"); // same uid
    assert!(*endpoint_1_a != *endpoint_1_b);

    assert_ok!(vault.write_endpoint(&endpoint_1_a).await);
    assert_eq!(*vault.read_endpoint(&uid_1).await.unwrap(), *endpoint_1_a);

    assert_ok!(vault.write_endpoint(&endpoint_1_b).await);
    assert_eq!(*vault.read_endpoint(&uid_1).await.unwrap(), *endpoint_1_b);
}

#[tokio::test]
async fn test_remove() {
    let (_temp_dir, vault) = make_temp_vault().await;

    let uid_1 = make_uid("uid_1");
    let uid_2 = make_uid("uid_2");

    assert_matches!(vault.read_endpoint(&uid_1).await, Err(VaultError::UidNotFound(_)));
    assert_matches!(vault.read_endpoint(&uid_2).await, Err(VaultError::UidNotFound(_)));

    let endpoint_1 = make_endpoint_spec(&uid_1, "name_1");
    let endpoint_2 = make_endpoint_spec(&uid_2, "name_2");

    assert_ok!(vault.write_endpoint(&endpoint_1).await);
    assert_ok!(vault.write_endpoint(&endpoint_2).await);

    assert_eq!(*vault.read_endpoint(&uid_1).await.unwrap(), *endpoint_1);
    assert_eq!(*vault.read_endpoint(&uid_2).await.unwrap(), *endpoint_2);

    assert_ok!(vault.remove_endpoint(&uid_1).await);
    assert_matches!(vault.read_endpoint(&uid_1).await, Err(VaultError::UidNotFound(_)));
    assert_eq!(*vault.read_endpoint(&uid_2).await.unwrap(), *endpoint_2);

    assert_ok!(vault.remove_endpoint(&uid_2).await);
    assert_matches!(vault.read_endpoint(&uid_1).await, Err(VaultError::UidNotFound(_)));
    assert_matches!(vault.read_endpoint(&uid_2).await, Err(VaultError::UidNotFound(_)));
}

#[tokio::test]
async fn test_list() {
    let (_temp_dir, vault) = make_temp_vault().await;

    let uid_1 = make_uid("uid_1");
    let uid_2 = make_uid("uid_2");

    let endpoints = vault.list_endpoints().await.unwrap();
    assert_eq!(endpoints.len(), 0);

    let endpoint_1 = make_endpoint_spec(&uid_1, "name_1");
    let endpoint_2 = make_endpoint_spec(&uid_2, "name_2");
    assert_ok!(vault.write_endpoint(&endpoint_1).await);
    assert_ok!(vault.write_endpoint(&endpoint_2).await);
    let endpoints = vault.list_endpoints().await.unwrap();
    assert_eq!(*endpoints, HashSet::from([uid_1, uid_2]));

    assert_ok!(vault.remove_endpoint(&uid_2).await);
    let endpoints = vault.list_endpoints().await.unwrap();
    assert_eq!(*endpoints, HashSet::from([uid_1]));

    assert_ok!(vault.remove_endpoint(&uid_1).await);
    let endpoints = vault.list_endpoints().await.unwrap();
    assert_eq!(endpoints.len(), 0);
}

#[tokio::test]
async fn test_reopen() {
    let temp_dir = make_temp_dir();

    let uid_1 = make_uid("uid_1");
    let uid_2 = make_uid("uid_2");

    let endpoint_1 = make_endpoint_spec(&uid_1, "name_1");
    let endpoint_2_a = make_endpoint_spec(&uid_2, "name_2_a");
    let endpoint_2_b = make_endpoint_spec(&uid_2, "name_2_b"); // same uid

    // Write and close DB.
    {
        let vault = Vault::open(
            &VaultConfigBuilder::default()
                .db_path(temp_dir.path().join("test.db").to_str().unwrap())
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

        assert_ok!(vault.write_endpoint(&endpoint_1).await);
        assert_ok!(vault.write_endpoint(&endpoint_2_a).await);
    }

    // Open same DB, check existing content, modify and close.
    {
        let vault = Vault::open(
            &VaultConfigBuilder::default()
                .db_path(temp_dir.path().join("test.db").to_str().unwrap())
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

        let endpoints = vault.list_endpoints().await.unwrap();
        assert_eq!(*endpoints, HashSet::from([uid_1, uid_2]));

        assert_eq!(*vault.read_endpoint(&uid_1).await.unwrap(), *endpoint_1);
        assert_eq!(*vault.read_endpoint(&uid_2).await.unwrap(), *endpoint_2_a);

        assert_ok!(vault.remove_endpoint(&uid_1).await);
        assert_ok!(vault.write_endpoint(&endpoint_2_b).await);
    }

    // Open same DB again, check content.
    {
        let vault = Vault::open(
            &VaultConfigBuilder::default()
                .db_path(temp_dir.path().join("test.db").to_str().unwrap())
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

        let endpoints = vault.list_endpoints().await.unwrap();
        assert_eq!(*endpoints, HashSet::from([uid_2]));

        assert_matches!(vault.read_endpoint(&uid_1).await, Err(VaultError::UidNotFound(_)));
        assert_eq!(*vault.read_endpoint(&uid_2).await.unwrap(), *endpoint_2_b);
    }
}

// Check reference counting and copy-on-write.
#[tokio::test]
async fn test_arc() {
    let (_temp_dir, vault) = make_temp_vault().await;

    let uid_1 = make_uid("uid_1");
    let uid_2 = make_uid("uid_2");

    let endpoint_1 = make_endpoint_spec(&uid_1, "name_1");
    let endpoint_2_a = make_endpoint_spec(&uid_2, "name_2_a");
    let endpoint_2_b = make_endpoint_spec(&uid_2, "name_2_b");

    assert_ok!(vault.write_endpoint(&endpoint_1).await);
    assert_ok!(vault.write_endpoint(&endpoint_2_a).await);

    assert!(Arc::ptr_eq(&vault.read_endpoint(&uid_1).await.unwrap(), &endpoint_1));
    assert!(Arc::ptr_eq(&vault.read_endpoint(&uid_2).await.unwrap(), &endpoint_2_a));

    {
        let mut endpoint_ptr: Arc<EndpointSpec> = vault.read_endpoint(&uid_2).await.unwrap();

        // Since vault also keeps a reference to the endpoint, make_mut() should
        // clone endpoint and reset endpoint_ptr to a new object.
        let endpoint = Arc::make_mut(&mut endpoint_ptr);

        endpoint.system_name = "name_2_b".to_string();

        assert_matches!(vault.write_endpoint(&endpoint_ptr).await, Ok(()));
    }

    // Endpoint 1 is same as before, because we haven't modified it.
    assert!(Arc::ptr_eq(&vault.read_endpoint(&uid_1).await.unwrap(), &endpoint_1));
    // Endpoint 2 is a new pointer, because we've modified it, and entries are immutable.
    assert!(!Arc::ptr_eq(&vault.read_endpoint(&uid_1).await.unwrap(), &endpoint_2_a));
    assert!(!Arc::ptr_eq(&vault.read_endpoint(&uid_1).await.unwrap(), &endpoint_2_b));
    // Endpoint 2 points to a struct equal to endpoint_2_b.
    assert_eq!(*vault.read_endpoint(&uid_2).await.unwrap(), *endpoint_2_b);
}

// Check reference counting and copy-on-write for entry list.
#[tokio::test]
async fn test_arc_list() {
    let (_temp_dir, vault) = make_temp_vault().await;

    let uid_1 = make_uid("uid_1");
    let uid_2 = make_uid("uid_2");
    let uid_3 = make_uid("uid_3");

    let endpoint_1 = make_endpoint_spec(&uid_1, "name_1");
    let endpoint_2 = make_endpoint_spec(&uid_2, "name_2");
    let endpoint_3 = make_endpoint_spec(&uid_3, "name_3");

    assert_ok!(vault.write_endpoint(&endpoint_1).await);

    let endpoints_ptr_1: *const HashSet<Uid>;
    {
        let endpoints = vault.list_endpoints().await.unwrap();
        assert_eq!(*endpoints, HashSet::from([uid_1]));
        endpoints_ptr_1 = Arc::as_ptr(&endpoints);
    }

    // Modify endpoint list without holding an Arc to the current list.
    assert_ok!(vault.write_endpoint(&endpoint_2).await);

    let endpoints_ptr_2: *const HashSet<Uid>;
    {
        let endpoints = vault.list_endpoints().await.unwrap();
        assert_eq!(*endpoints, HashSet::from([uid_1, uid_2]));
        endpoints_ptr_2 = Arc::as_ptr(&endpoints);
    }

    // Since we weren't holding an Arc, hashset was updated in-place.
    assert!(std::ptr::eq(endpoints_ptr_1, endpoints_ptr_2));

    // Modify endpoint list while holding an Arc to the current list.
    let old_endpoints = vault.list_endpoints().await.unwrap();
    assert_ok!(vault.write_endpoint(&endpoint_3).await);

    let endpoints_ptr_3: *const HashSet<Uid>;
    {
        let endpoints = vault.list_endpoints().await.unwrap();
        assert_eq!(*endpoints, HashSet::from([uid_1, uid_2, uid_3]));
        endpoints_ptr_3 = Arc::as_ptr(&endpoints);
    }

    // Since we were holding an Arc, a new hashset was allocated.
    assert!(!std::ptr::eq(endpoints_ptr_1, endpoints_ptr_3));
    assert_eq!(*old_endpoints, HashSet::from([uid_1, uid_2]));
}

// How LRU cache works when cache is smaller than DB size.
#[traced_test]
#[tokio::test]
async fn test_small_cache() {
    const CACHE_SIZE: usize = 10;
    const TOTAL_SIZE: usize = 30;

    let temp_dir = make_temp_dir();

    let vault = Vault::open(
        &VaultConfigBuilder::default()
            .db_path(temp_dir.path().join("test.db").to_str().unwrap())
            .cache_size(CACHE_SIZE)
            .build()
            .unwrap(),
    )
    .await
    .unwrap();

    // Write TOTAL_SIZE endpoints.
    for n in 0..TOTAL_SIZE {
        let uid = make_uid(format!("uid_{n}"));
        let name = format!("name_{n}");
        let endpoint = make_endpoint_spec(&uid, &name);
        assert_matches!(vault.write_endpoint(&endpoint).await, Ok(()));
    }

    let metrics = vault.metrics().await;
    assert_eq!(metrics.cache_size, CACHE_SIZE);
    assert!(CACHE_SIZE < TOTAL_SIZE);
    assert_eq!(metrics.db_reads, 0);
    assert_eq!(metrics.db_writes, TOTAL_SIZE);

    // Read TOTAL_SIZE endpoints.
    for n in 0..TOTAL_SIZE {
        let uid = make_uid(format!("uid_{n}"));
        let name = format!("name_{n}");
        let expected_endpoint = make_endpoint_spec(&uid, &name);
        let actual_endpoint = vault.read_endpoint(&uid).await.unwrap();
        assert_eq!(*expected_endpoint, *actual_endpoint);
    }

    let metrics = vault.metrics().await;
    assert_eq!(metrics.cache_size, CACHE_SIZE);
    assert!(CACHE_SIZE < TOTAL_SIZE);
    // A bit relaxed requirement for # of read operations, because we
    // don't want to rely on exact detail of quick-cache.
    assert_ge!(metrics.db_reads, TOTAL_SIZE - CACHE_SIZE - 1);
    assert_le!(metrics.db_reads, TOTAL_SIZE - CACHE_SIZE + 1);
    assert_eq!(metrics.db_writes, TOTAL_SIZE);
}

// How LRU cache works when cache is larger than DB size.
#[traced_test]
#[tokio::test]
async fn test_big_cache() {
    const CACHE_SIZE: usize = 30;
    const TOTAL_SIZE: usize = 10;

    let temp_dir = make_temp_dir();

    let vault = Vault::open(
        &VaultConfigBuilder::default()
            .db_path(temp_dir.path().join("test.db").to_str().unwrap())
            .cache_size(CACHE_SIZE)
            .build()
            .unwrap(),
    )
    .await
    .unwrap();

    // Write TOTAL_SIZE endpoints.
    for n in 0..TOTAL_SIZE {
        let uid = make_uid(format!("uid_{n}"));
        let name = format!("name_{n}");
        let endpoint = make_endpoint_spec(&uid, &name);
        assert_matches!(vault.write_endpoint(&endpoint).await, Ok(()));
    }

    let metrics = vault.metrics().await;
    assert_eq!(metrics.cache_size, TOTAL_SIZE);
    assert!(TOTAL_SIZE < CACHE_SIZE);
    assert_eq!(metrics.db_reads, 0);
    assert_eq!(metrics.db_writes, TOTAL_SIZE);

    // Read TOTAL_SIZE endpoints.
    for n in 0..TOTAL_SIZE {
        let uid = make_uid(format!("uid_{n}"));
        let name = format!("name_{n}");
        let expected_endpoint = make_endpoint_spec(&uid, &name);
        let actual_endpoint = vault.read_endpoint(&uid).await.unwrap();
        assert_eq!(*expected_endpoint, *actual_endpoint);
    }

    let metrics = vault.metrics().await;
    assert_eq!(metrics.cache_size, TOTAL_SIZE);
    assert!(TOTAL_SIZE < CACHE_SIZE);
    assert_eq!(metrics.db_reads, 0); // every read was from cache
    assert_eq!(metrics.db_writes, TOTAL_SIZE);
}

// Ensure all DTOs are supported.
#[tokio::test]
async fn test_dto_types() {
    let (_temp_dir, vault) = make_temp_vault().await;

    let peer_uid = make_uid("peer_uid");
    let peer_spec = make_peer_spec(&peer_uid);

    let endpoint_uid = make_uid("endpoint_uid");
    let endpoint_spec = make_endpoint_spec(&endpoint_uid, "endpoint_name");

    let stream_uid = make_uid("stream_uid");
    let stream_spec = make_stream_spec(&stream_uid);

    // write
    assert_ok!(vault.write_peer(&peer_spec).await);
    assert_ok!(vault.write_endpoint(&endpoint_spec).await);
    assert_ok!(vault.write_stream(&stream_spec).await);

    // read
    assert_eq!(*vault.read_peer(&peer_uid).await.unwrap(), *peer_spec);
    assert_eq!(*vault.read_endpoint(&endpoint_uid).await.unwrap(), *endpoint_spec);
    assert_eq!(*vault.read_stream(&stream_uid).await.unwrap(), *stream_spec);

    // list
    assert_eq!(*vault.list_peers().await.unwrap(), HashSet::from([peer_uid]));
    assert_eq!(*vault.list_endpoints().await.unwrap(), HashSet::from([endpoint_uid]));
    assert_eq!(*vault.list_streams().await.unwrap(), HashSet::from([stream_uid]));

    // remove
    assert_ok!(vault.remove_peer(&peer_uid).await);
    assert_ok!(vault.remove_endpoint(&endpoint_uid).await);
    assert_ok!(vault.remove_stream(&stream_uid).await);

    // list
    assert_eq!(*vault.list_peers().await.unwrap(), HashSet::new());
    assert_eq!(*vault.list_endpoints().await.unwrap(), HashSet::new());
    assert_eq!(*vault.list_streams().await.unwrap(), HashSet::new());
}
