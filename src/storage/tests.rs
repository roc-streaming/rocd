// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use super::*;
use crate::models::*;

use assertables::*;
use ctor::ctor;
use std::path::Path;
use tempfile::TempDir;
use tracing_test::traced_test;

#[ctor]
fn setup() {
    procspawn::init();
}

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

// Parent directories are not automatically created on open.
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

// If directories exist, DB file is automatically created on open.
#[tokio::test]
async fn test_open_create() {
    let temp_dir = make_temp_dir();

    assert!(!temp_dir.path().join("test.db").is_file());

    let result = Storage::open(
        &StorageConfigBuilder::default()
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
        let result = Storage::open(
            &StorageConfigBuilder::default()
                .db_path(temp_dir.path().join("test.db").to_str().unwrap())
                .build()
                .unwrap(),
        )
        .await;
        assert_ok!(&result);
    }

    assert!(temp_dir.path().join("test.db").is_file());

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

    assert!(temp_dir.path().join("test.db").is_file());
}

// Can't open same DB concurrently.
#[tokio::test]
async fn test_open_locked() {
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
        assert!(temp_dir.path().join("test.db").is_file());

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

// Can't open same DB concurrently, even from separate processes.
#[tokio::test]
async fn test_open_locked_two_processes() {
    let temp_dir = make_temp_dir();
    let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();

    let result = Storage::open(
        &StorageConfigBuilder::default().db_path(db_path.clone()).build().unwrap(),
    )
    .await;
    assert_ok!(&result);

    // run closure in a new process
    procspawn::spawn(db_path, |db_path| -> () {
        assert!(Path::new(&db_path).is_file());

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let result = Storage::open(
                &StorageConfigBuilder::default().db_path(db_path).build().unwrap(),
            )
            .await;
            // can't open: db is locked (from another process)
            assert_matches!(&result, Err(StorageError::DatabaseError(_)));
        });
    })
    .join()
    .unwrap();
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
async fn test_list() {
    let (_temp_dir, storage) = make_temp_storage().await;

    let devices = storage.list_devices().await.unwrap();
    assert_eq!(devices.len(), 0);

    let device_1 = make_device("uid_1", "name_1");
    let device_2 = make_device("uid_2", "name_2");
    assert_ok!(storage.write_device(&device_1).await);
    assert_ok!(storage.write_device(&device_2).await);
    let devices = storage.list_devices().await.unwrap();
    assert_eq!(*devices, HashSet::from(["uid_1".to_string(), "uid_2".to_string()]));

    assert_ok!(storage.remove_device("uid_2").await);
    let devices = storage.list_devices().await.unwrap();
    assert_eq!(*devices, HashSet::from(["uid_1".to_string()]));

    assert_ok!(storage.remove_device("uid_1").await);
    let devices = storage.list_devices().await.unwrap();
    assert_eq!(devices.len(), 0);
}

#[tokio::test]
async fn test_reopen() {
    let temp_dir = make_temp_dir();

    let device_1 = make_device("uid_1", "name_1");
    let device_2_a = make_device("uid_2", "name_2_a");
    let device_2_b = make_device("uid_2", "name_2_b"); // same uid

    // Write and close DB.
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

    // Open same DB, check existing content, modify and close.
    {
        let storage = Storage::open(
            &StorageConfigBuilder::default()
                .db_path(temp_dir.path().join("test.db").to_str().unwrap())
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

        let devices = storage.list_devices().await.unwrap();
        assert_eq!(*devices, HashSet::from(["uid_1".to_string(), "uid_2".to_string()]));

        assert_eq!(*storage.read_device("uid_1").await.unwrap(), *device_1);
        assert_eq!(*storage.read_device("uid_2").await.unwrap(), *device_2_a);

        assert_ok!(storage.remove_device("uid_1").await);
        assert_ok!(storage.write_device(&device_2_b).await);
    }

    // Open same DB again, check content.
    {
        let storage = Storage::open(
            &StorageConfigBuilder::default()
                .db_path(temp_dir.path().join("test.db").to_str().unwrap())
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

        let devices = storage.list_devices().await.unwrap();
        assert_eq!(*devices, HashSet::from(["uid_2".to_string()]));

        assert_matches!(storage.read_device("uid_1").await, Err(StorageError::UidNotFound(_)));
        assert_eq!(*storage.read_device("uid_2").await.unwrap(), *device_2_b);
    }
}

// Check reference counting and copy-on-write.
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

        // Since storage also keeps a reference to the device, make_mut() should
        // clone device and reset device_ptr to a new object.
        let device = Arc::make_mut(&mut device_ptr);

        device.system_name = "name_2_b".to_string();

        assert_matches!(storage.write_device(&device_ptr).await, Ok(()));
    }

    // Device 1 is same as before, because we haven't modified it.
    assert!(Arc::ptr_eq(&storage.read_device("uid_1").await.unwrap(), &device_1));
    // Device 2 is a new pointer, because we've modified it, and entries are immutable.
    assert!(!Arc::ptr_eq(&storage.read_device("uid_1").await.unwrap(), &device_2_a));
    assert!(!Arc::ptr_eq(&storage.read_device("uid_1").await.unwrap(), &device_2_b));
    // Device 2 points to a struct equal to device_2_b.
    assert_eq!(*storage.read_device("uid_2").await.unwrap(), *device_2_b);
}

// Check reference counting and copy-on-write for entry list.
#[tokio::test]
async fn test_arc_list() {
    let (_temp_dir, storage) = make_temp_storage().await;

    let device_1 = make_device("uid_1", "name_1");
    let device_2 = make_device("uid_2", "name_2");
    let device_3 = make_device("uid_3", "name_3");

    assert_ok!(storage.write_device(&device_1).await);

    let devices_ptr_1: *const HashSet<String>;
    {
        let devices = storage.list_devices().await.unwrap();
        assert_eq!(*devices, HashSet::from(["uid_1".to_string()]));
        devices_ptr_1 = Arc::as_ptr(&devices);
    }

    // Modify device list without holding an Arc to the current list.
    assert_ok!(storage.write_device(&device_2).await);

    let devices_ptr_2: *const HashSet<String>;
    {
        let devices = storage.list_devices().await.unwrap();
        assert_eq!(*devices, HashSet::from(["uid_1".to_string(), "uid_2".to_string()]));
        devices_ptr_2 = Arc::as_ptr(&devices);
    }

    // Since we weren't holding an Arc, hashset was updated in-place.
    assert!(std::ptr::eq(devices_ptr_1, devices_ptr_2));

    // Modify device list while holding an Arc to the current list.
    let old_devices = storage.list_devices().await.unwrap();
    assert_ok!(storage.write_device(&device_3).await);

    let devices_ptr_3: *const HashSet<String>;
    {
        let devices = storage.list_devices().await.unwrap();
        assert_eq!(
            *devices,
            HashSet::from(["uid_1".to_string(), "uid_2".to_string(), "uid_3".to_string()])
        );
        devices_ptr_3 = Arc::as_ptr(&devices);
    }

    // Since we were holding an Arc, a new hashset was allocated.
    assert!(!std::ptr::eq(devices_ptr_1, devices_ptr_3));
    assert_eq!(*old_devices, HashSet::from(["uid_1".to_string(), "uid_2".to_string()]));
}

// How LRU cache works when cache is smaller than DB size.
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

    // Write TOTAL_SIZE devices.
    for n in 0..TOTAL_SIZE {
        let device = make_device(format!("uid_{n}"), format!("name_{n}"));
        assert_matches!(storage.write_device(&device).await, Ok(()));
    }

    let metrics = storage.metrics().await;
    assert_eq!(metrics.cache_size, CACHE_SIZE);
    assert!(CACHE_SIZE < TOTAL_SIZE);
    assert_eq!(metrics.db_reads, 0);
    assert_eq!(metrics.db_writes, TOTAL_SIZE as u64);

    // Read TOTAL_SIZE devices.
    for n in 0..TOTAL_SIZE {
        let expected_device = make_device(format!("uid_{n}"), format!("name_{n}"));
        let actual_device = storage.read_device(format!("uid_{n}").as_str()).await.unwrap();
        assert_eq!(*expected_device, *actual_device);
    }

    let metrics = storage.metrics().await;
    assert_eq!(metrics.cache_size, CACHE_SIZE);
    assert!(CACHE_SIZE < TOTAL_SIZE);
    // A bit relaxed requirement for # of read operations, because we
    // don't want to rely on exact detail of quick-cache.
    assert_ge!(metrics.db_reads, (TOTAL_SIZE - CACHE_SIZE - 1) as u64);
    assert_le!(metrics.db_reads, (TOTAL_SIZE - CACHE_SIZE + 1) as u64);
    assert_eq!(metrics.db_writes, TOTAL_SIZE as u64);
}

// How LRU cache works when cache is larger than DB size.
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

    // Write TOTAL_SIZE devices.
    for n in 0..TOTAL_SIZE {
        let device = make_device(format!("uid_{n}"), format!("name_{n}"));
        assert_matches!(storage.write_device(&device).await, Ok(()));
    }

    let metrics = storage.metrics().await;
    assert_eq!(metrics.cache_size, TOTAL_SIZE);
    assert!(TOTAL_SIZE < CACHE_SIZE);
    assert_eq!(metrics.db_reads, 0);
    assert_eq!(metrics.db_writes, TOTAL_SIZE as u64);

    // Read TOTAL_SIZE devices.
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
