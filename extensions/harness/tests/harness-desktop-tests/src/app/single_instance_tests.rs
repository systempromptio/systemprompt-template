use systemprompt_harness_shell::app::single_instance::pipe_name;

use crate::TestResult;

#[test]
fn pipe_names_are_per_build_and_per_root() {
    let a = pipe_name("io.example.app", r"C:\Users\a\AppData\Local\Vendor\App");
    assert_eq!(
        a,
        pipe_name("io.example.app", r"c:\users\A\appdata\local\vendor\app")
    );
    assert_ne!(
        a,
        pipe_name("io.example.app", r"C:\Users\b\AppData\Local\Vendor\App")
    );
    assert_ne!(
        a,
        pipe_name("io.example.app.dev", r"C:\Users\a\AppData\Local\Vendor\App")
    );
    assert!(a.starts_with(r"\\.\pipe\io.example.app-"));
}

#[cfg(unix)]
#[test]
fn a_second_start_hands_its_request_to_the_first() -> TestResult {
    use std::time::Duration;

    use systemprompt_harness_shell::app::single_instance::{Endpoint, Instance, acquire, serve};
    use systemprompt_harness_views::app::deeplink::LaunchRequest;
    use systemprompt_harness_views::app::paths::AppPaths;
    use systemprompt_traits::BackgroundTasks;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let dir = tempfile::tempdir()?;
    let paths = AppPaths::under(dir.path().to_path_buf(), dir.path().to_path_buf());
    std::fs::create_dir_all(&paths.state)?;
    let endpoint = Endpoint::for_app(systemprompt_harness_brand::identity(), &paths);
    runtime.block_on(async {
        let Instance::Primary(listener) = acquire(&endpoint, &LaunchRequest::default()).await
        else {
            return Err("the first start must own the endpoint".into());
        };
        let tasks = BackgroundTasks::new();
        let (sender, mut received) = tokio::sync::mpsc::unbounded_channel();
        serve(listener, endpoint.clone(), &tasks, sender);
        let request = LaunchRequest {
            directory: Some("/work".into()),
            links: vec!["sp-harness://settings".into()],
        };
        assert!(matches!(
            acquire(&endpoint, &request).await,
            Instance::Forwarded
        ));
        assert_eq!(received.recv().await, Some(request));
        assert!(
            tasks
                .shutdown_or_abort(Duration::from_secs(1))
                .await
                .is_drained()
        );
        Ok(())
    })
}
