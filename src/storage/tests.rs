// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use super::*;
use crate::models::*;

use assertables::*;
use tempfile::TempDir;
use tracing_test::traced_test;

// Create and returns temp dir.
// TempDir destructor will delete the dir.
fn make_temp_dir() -> TempDir {
    TempDir::with_prefix("rocd_test").unwrap()
}

// Creates temp dir and opens storage.
async fn make_temp_storage() -> (TempDir, Storage) {
    let temp_dir = make_temp_dir();

    let storage = Storage::open(
        &StorageConfigBuilder::default()
            .db_path(temp_dir.path().join("test.db").to_str().unwrap())
            .build()
            .unwrap(),
    )
    .await
    .unwrap();

    // Note: tuples elements are destroyed from right to left, so
    // when the tuple is dropped, storage will be destroyed first,
    // then temp dir will be removed.
    (temp_dir, storage)
}

fn make_device<S: Into<String>>(uid: S, name: S) -> Arc<Device> {
    Arc::new(Device {
        uid: uid.into(),
        system_name: name.into(),
        display_name: "test".into(),
        dev_type: DeviceType::Sink,
        driver: DeviceDriver::Pipewire,
        is_hardware: true,
        is_stream: false,
        status: DeviceStatus::Enabled,
        is_muted: false,
    })
}

#[tokio::test]
async fn test_open_bad_path() {
    let result = Storage::open(
        &StorageConfigBuilder::default()
            .db_path("/very/bad/non/existent/path/test.db")
            .build()
            .unwrap(),
    )
    .await;
    // can't open: path doesn't exist
    assert_matches!(&result, Err(StorageError::DatabaseError(_)));
}

#[tokio::test]
async fn test_open_create() {
    let temp_dir = make_temp_dir();

    let result = Storage::open(
        &StorageConfigBuilder::default()
            .db_path(temp_dir.path().join("test.db").to_str().unwrap())
            .build()
            .unwrap(),
    )
    .await;
    assert_ok!(&result);
}

#[tokio::test]
async fn test_open_existing() {
    let temp_dir = make_temp_dir();

    {
        let result = Storage::open(
            &StorageConfigBuilder::default()
                .db_path(temp_dir.path().join("test.db").to_str().unwrap())
                .build()
                .unwrap(),
        )
        .await;
        assert_ok!(&result);
    }

    {
        let result = Storage::open(
            &StorageConfigBuilder::default()
                .db_path(temp_dir.path().join("test.db").to_str().unwrap())
                .build()
                .unwrap(),
        )
        .await;

        assert_ok!(&result);
    }
}

#[tokio::test]
async fn test_open_already_opened() {
    let temp_dir = make_temp_dir();

    let result = Storage::open(
        &StorageConfigBuilder::default()
            .db_path(temp_dir.path().join("test.db").to_str().unwrap())
            .build()
            .unwrap(),
    )
    .await;
    assert_ok!(&result);

    {
        let result = Storage::open(
            &StorageConfigBuilder::default()
                .db_path(temp_dir.path().join("test.db").to_str().unwrap())
                .build()
                .unwrap(),
        )
        .await;
        // can't open: db is locked
        assert_matches!(&result, Err(StorageError::DatabaseError(_)));
    }
}

#[tokio::test]
async fn test_read_write() {
    let (_temp_dir, storage) = make_temp_storage().await;

    assert_matches!(storage.read_device("uid_1").await, Err(StorageError::UidNotFound(_)));
    assert_matches!(storage.read_device("uid_2").await, Err(StorageError::UidNotFound(_)));

    let device_1 = make_device("uid_1", "name_1");
    let device_2 = make_device("uid_2", "name_2");
    assert!(*device_1 != *device_2);

    assert_ok!(storage.write_device(&device_1).await);
    assert_eq!(*storage.read_device("uid_1").await.unwrap(), *device_1);
    assert_matches!(storage.read_device("uid_2").await, Err(StorageError::UidNotFound(_)));

    assert_ok!(storage.write_device(&device_2).await);
    assert_eq!(*storage.read_device("uid_1").await.unwrap(), *device_1);
    assert_eq!(*storage.read_device("uid_2").await.unwrap(), *device_2);
}

#[tokio::test]
async fn test_overwrite() {
    let (_temp_dir, storage) = make_temp_storage().await;

    assert_matches!(storage.read_device("uid_1").await, Err(StorageError::UidNotFound(_)));
    assert_matches!(storage.read_device("uid_2").await, Err(StorageError::UidNotFound(_)));

    let device_1_a = make_device("uid_1", "name_1_a");
    let device_1_b = make_device("uid_1", "name_1_b"); // same uid
    assert!(*device_1_a != *device_1_b);

    assert_ok!(storage.write_device(&device_1_a).await);
    assert_eq!(*storage.read_device("uid_1").await.unwrap(), *device_1_a);

    assert_ok!(storage.write_device(&device_1_b).await);
    assert_eq!(*storage.read_device("uid_1").await.unwrap(), *device_1_b);
}

#[tokio::test]
async fn test_remove() {
    let (_temp_dir, storage) = make_temp_storage().await;

    assert_matches!(storage.read_device("uid_1").await, Err(StorageError::UidNotFound(_)));
    assert_matches!(storage.read_device("uid_2").await, Err(StorageError::UidNotFound(_)));

    let device_1 = make_device("uid_1", "name_1");
    let device_2 = make_device("uid_2", "name_2");

    assert_ok!(storage.write_device(&device_1).await);
    assert_ok!(storage.write_device(&device_2).await);

    assert_eq!(*storage.read_device("uid_1").await.unwrap(), *device_1);
    assert_eq!(*storage.read_device("uid_2").await.unwrap(), *device_2);

    assert_ok!(storage.remove_device("uid_1").await);
    assert_matches!(storage.read_device("uid_1").await, Err(StorageError::UidNotFound(_)));
    assert_eq!(*storage.read_device("uid_2").await.unwrap(), *device_2);

    assert_ok!(storage.remove_device("uid_2").await);
    assert_matches!(storage.read_device("uid_1").await, Err(StorageError::UidNotFound(_)));
    assert_matches!(storage.read_device("uid_2").await, Err(StorageError::UidNotFound(_)));
}

#[tokio::test]
async fn test_reopen() {
    let temp_dir = make_temp_dir();

    let device_1 = make_device("uid_1", "name_1");
    let device_2_a = make_device("uid_2", "name_2_a");
    let device_2_b = make_device("uid_2", "name_2_b"); // same uid

    {
        let storage = Storage::open(
            &StorageConfigBuilder::default()
                .db_path(temp_dir.path().join("test.db").to_str().unwrap())
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

        assert_ok!(storage.write_device(&device_1).await);
        assert_ok!(storage.write_device(&device_2_a).await);
    }

    {
        let storage = Storage::open(
            &StorageConfigBuilder::default()
                .db_path(temp_dir.path().join("test.db").to_str().unwrap())
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(*storage.read_device("uid_1").await.unwrap(), *device_1);
        assert_eq!(*storage.read_device("uid_2").await.unwrap(), *device_2_a);

        assert_ok!(storage.remove_device("uid_1").await);
        assert_ok!(storage.write_device(&device_2_b).await);
    }

    {
        let storage = Storage::open(
            &StorageConfigBuilder::default()
                .db_path(temp_dir.path().join("test.db").to_str().unwrap())
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

        assert_matches!(storage.read_device("uid_1").await, Err(StorageError::UidNotFound(_)));
        assert_eq!(*storage.read_device("uid_2").await.unwrap(), *device_2_b);
    }
}

#[tokio::test]
async fn test_arc() {
    let (_temp_dir, storage) = make_temp_storage().await;

    let device_1 = make_device("uid_1", "name_1");
    let device_2_a = make_device("uid_2", "name_2_a");
    let device_2_b = make_device("uid_2", "name_2_b");

    assert_ok!(storage.write_device(&device_1).await);
    assert_ok!(storage.write_device(&device_2_a).await);

    assert!(Arc::ptr_eq(&storage.read_device("uid_1").await.unwrap(), &device_1));
    assert!(Arc::ptr_eq(&storage.read_device("uid_2").await.unwrap(), &device_2_a));

    {
        let mut device_ptr: Arc<Device> = storage.read_device("uid_2").await.unwrap();

        // since storage also keeps a reference to the device, make_mut() should
        // clone device and reset device_ptr to new object
        let device = Arc::make_mut(&mut device_ptr);

        device.system_name = "name_2_b".to_string();

        assert_matches!(storage.write_device(&device_ptr).await, Ok(()));
    }

    // device 1 is same as before
    assert!(Arc::ptr_eq(&storage.read_device("uid_1").await.unwrap(), &device_1));
    // device 2 is new pointer
    assert!(!Arc::ptr_eq(&storage.read_device("uid_1").await.unwrap(), &device_2_a));
    assert!(!Arc::ptr_eq(&storage.read_device("uid_1").await.unwrap(), &device_2_b));
    // pointee is equal device_2_b
    assert_eq!(*storage.read_device("uid_2").await.unwrap(), *device_2_b);
}

#[traced_test]
#[tokio::test]
async fn test_small_cache() {
    const CACHE_SIZE: usize = 10;
    const TOTAL_SIZE: usize = 30;

    let temp_dir = make_temp_dir();

    let storage = Storage::open(
        &StorageConfigBuilder::default()
            .db_path(temp_dir.path().join("test.db").to_str().unwrap())
            .cache_size(CACHE_SIZE)
            .build()
            .unwrap(),
    )
    .await
    .unwrap();

    // write TOTAL_SIZE devices
    for n in 0..TOTAL_SIZE {
        let device = make_device(format!("uid_{n}"), format!("name_{n}"));
        assert_matches!(storage.write_device(&device).await, Ok(()));
    }

    let metrics = storage.metrics().await;
    assert_eq!(metrics.cache_size, CACHE_SIZE);
    assert!(CACHE_SIZE < TOTAL_SIZE);
    assert_eq!(metrics.db_reads, 0);
    assert_eq!(metrics.db_writes, TOTAL_SIZE as u64);

    // read TOTAL_SIZE devices
    for n in 0..TOTAL_SIZE {
        let expected_device = make_device(format!("uid_{n}"), format!("name_{n}"));
        let actual_device = storage.read_device(format!("uid_{n}").as_str()).await.unwrap();
        assert_eq!(*expected_device, *actual_device);
    }

    let metrics = storage.metrics().await;
    assert_eq!(metrics.cache_size, CACHE_SIZE);
    assert!(CACHE_SIZE < TOTAL_SIZE);
    // a bit relaxed requirement for # of read operations, because we
    // don't want to rely on exact detail of quick-cache
    assert_ge!(metrics.db_reads, (TOTAL_SIZE - CACHE_SIZE - 1) as u64);
    assert_le!(metrics.db_reads, (TOTAL_SIZE - CACHE_SIZE + 1) as u64);
    assert_eq!(metrics.db_writes, TOTAL_SIZE as u64);
}

#[traced_test]
#[tokio::test]
async fn test_big_cache() {
    const CACHE_SIZE: usize = 30;
    const TOTAL_SIZE: usize = 10;

    let temp_dir = make_temp_dir();

    let storage = Storage::open(
        &StorageConfigBuilder::default()
            .db_path(temp_dir.path().join("test.db").to_str().unwrap())
            .cache_size(CACHE_SIZE)
            .build()
            .unwrap(),
    )
    .await
    .unwrap();

    // write TOTAL_SIZE devices
    for n in 0..TOTAL_SIZE {
        let device = make_device(format!("uid_{n}"), format!("name_{n}"));
        assert_matches!(storage.write_device(&device).await, Ok(()));
    }

    let metrics = storage.metrics().await;
    assert_eq!(metrics.cache_size, TOTAL_SIZE);
    assert!(TOTAL_SIZE < CACHE_SIZE);
    assert_eq!(metrics.db_reads, 0);
    assert_eq!(metrics.db_writes, TOTAL_SIZE as u64);

    // read TOTAL_SIZE devices
    for n in 0..TOTAL_SIZE {
        let expected_device = make_device(format!("uid_{n}"), format!("name_{n}"));
        let actual_device = storage.read_device(format!("uid_{n}").as_str()).await.unwrap();
        assert_eq!(*expected_device, *actual_device);
    }

    let metrics = storage.metrics().await;
    assert_eq!(metrics.cache_size, TOTAL_SIZE);
    assert!(TOTAL_SIZE < CACHE_SIZE);
    assert_eq!(metrics.db_reads, 0); // every read was from cache
    assert_eq!(metrics.db_writes, TOTAL_SIZE as u64);
}
