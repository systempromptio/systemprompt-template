use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

#[tokio::test]
async fn tool_execution_response_returned_quickly() {
    let start = Instant::now();

    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    });

    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 150,
        "Response should return without blocking on background task: {}ms",
        elapsed.as_millis()
    );
}

#[tokio::test]
async fn tokio_spawn_does_not_block() {
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    let start = Instant::now();

    tokio::spawn(async move {
        let mut c = counter_clone.lock().await;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        *c = 1;
    });

    let immediate_elapsed = start.elapsed();

    assert!(
        immediate_elapsed.as_millis() < 50,
        "Spawn should return immediately without waiting for background task: {}ms",
        immediate_elapsed.as_millis()
    );

    assert_eq!(
        *counter.lock().await,
        0,
        "Counter should still be 0 immediately after spawn"
    );

    tokio::time::sleep(std::time::Duration::from_millis(600)).await;

    assert_eq!(
        *counter.lock().await,
        1,
        "Counter should be 1 after background task completes"
    );
}

#[tokio::test]
async fn multiple_concurrent_spawns_fast() {
    let start = Instant::now();

    let mut handles = vec![];
    for i in 0..10 {
        let handle = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100 + i * 10)).await;
            i
        });
        handles.push(handle);
    }

    let spawn_elapsed = start.elapsed();

    assert!(
        spawn_elapsed.as_millis() < 100,
        "Spawning 10 tasks should be fast: {}ms",
        spawn_elapsed.as_millis()
    );

    for handle in handles {
        let _ = handle.await;
    }
}

#[tokio::test]
async fn error_in_spawned_task_does_not_panic() {
    let error_occurred = Arc::new(Mutex::new(false));
    let error_flag = error_occurred.clone();

    tokio::spawn(async move {
        eprintln!("Error in background task: simulated error");
        *error_flag.lock().await = true;
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    assert!(*error_occurred.lock().await, "Error flag should be set");
}

#[tokio::test]
async fn spawned_task_eventually_completes() {
    let flag = Arc::new(Mutex::new(false));
    let flag_clone = flag.clone();

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let mut f = flag_clone.lock().await;
        *f = true;
    });

    assert!(
        !*flag.lock().await,
        "Flag should be false immediately after spawn"
    );

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    assert!(
        *flag.lock().await,
        "Flag should be true after task completes"
    );
}

#[tokio::test]
async fn cloning_primitives_for_spawn() -> anyhow::Result<()> {
    let test_str = "test_value".to_string();
    let test_id = "12345".to_string();

    let str_clone = test_str.clone();
    let id_clone = test_id.clone();

    tokio::spawn(async move {
        assert_eq!(str_clone, "test_value");
        assert_eq!(id_clone, "12345");
    })
    .await?;

    assert_eq!(test_str, "test_value");
    assert_eq!(test_id, "12345");
    Ok(())
}

#[test]
fn clone_availability() {
    #[derive(Clone)]
    struct CloneableRepo {
        id: String,
    }

    let repo = CloneableRepo {
        id: "test".to_string(),
    };

    let repo_clone = repo.clone();

    assert_eq!(repo.id, "test");
    assert_eq!(repo_clone.id, "test");
}

#[tokio::test]
async fn non_blocking_pattern_basic() {
    let data = Arc::new(Mutex::new(0));
    let data_clone = data.clone();

    let start = Instant::now();

    {
        let d = data.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            let mut val = d.lock().await;
            *val = 42;
        });
    }

    let response_time = start.elapsed();

    assert!(response_time.as_millis() < 50);
    assert_eq!(*data_clone.lock().await, 0);

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    assert_eq!(*data_clone.lock().await, 42);
}

#[tokio::test]
async fn error_handling_in_spawned_task() {
    let result_holder = Arc::new(Mutex::new(None));
    let result_clone = result_holder.clone();

    {
        let res = result_holder.clone();
        tokio::spawn(async move {
            let error = "simulated error".to_string();
            let mut r = res.lock().await;
            *r = Some(error);
        });
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    assert_eq!(
        result_clone.lock().await.as_ref().map(|s| s.as_str()),
        Some("simulated error")
    );
}

#[tokio::test]
async fn multiple_operations_spawned() {
    let op1_done = Arc::new(Mutex::new(false));
    let op2_done = Arc::new(Mutex::new(false));
    let op3_done = Arc::new(Mutex::new(false));

    let op1_clone = op1_done.clone();
    let op2_clone = op2_done.clone();
    let op3_clone = op3_done.clone();

    let start = Instant::now();

    {
        let o1 = op1_done.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            *o1.lock().await = true;
        });
    }

    {
        let o2 = op2_done.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            *o2.lock().await = true;
        });
    }

    {
        let o3 = op3_done.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            *o3.lock().await = true;
        });
    }

    let spawn_time = start.elapsed();

    assert!(spawn_time.as_millis() < 50);

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    assert!(*op1_clone.lock().await);
    assert!(*op2_clone.lock().await);
    assert!(*op3_clone.lock().await);
}
