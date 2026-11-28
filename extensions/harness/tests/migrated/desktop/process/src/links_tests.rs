//! Links and starts handed to the running app: queued until the window reads
//! them, delivered once and in order.

use systemprompt_harness_views::app::deeplink::{LaunchRequest, parse_args};

use crate::TestResult;

fn strings(list: &[&str]) -> Vec<String> {
    list.iter().map(|item| (*item).to_owned()).collect()
}

#[test]
fn links_given_at_launch_reach_the_app_once_and_in_order() -> TestResult {
    let scheme = systemprompt_harness_brand::identity().scheme();
    let first = format!("{scheme}://settings?page=about");
    let second = format!("{scheme}://new?prompt=hello");
    let request = parse_args(scheme, strings(&[&first, "/work", &second]))?;
    assert_eq!(request.links, vec![first, second]);
    assert_eq!(request.directory, Some("/work".into()));
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_start_handed_over_before_the_window_reads_it_waits_and_arrives_once() -> TestResult {
    use std::time::Duration;

    use systemprompt_harness_shell::app::single_instance::{Endpoint, Instance, acquire, serve};
    use systemprompt_harness_views::app::paths::AppPaths;
    use systemprompt_traits::BackgroundTasks;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let dir = tempfile::tempdir()?;
    let paths = AppPaths::under(dir.path().to_path_buf(), dir.path().to_path_buf());
    std::fs::create_dir_all(&paths.state)?;
    let identity = systemprompt_harness_brand::identity();
    let endpoint = Endpoint::for_app(identity, &paths);
    let scheme = identity.scheme();
    runtime.block_on(async {
        let Instance::Primary(listener) = acquire(&endpoint, &LaunchRequest::default()).await
        else {
            return Err("the first start must own the endpoint".into());
        };
        let tasks = BackgroundTasks::new();
        let (sender, mut window) = tokio::sync::mpsc::unbounded_channel();
        serve(listener, endpoint.clone(), &tasks, sender);
        let early = LaunchRequest {
            directory: None,
            links: vec![format!("{scheme}://settings")],
        };
        let later = LaunchRequest {
            directory: Some("/work".into()),
            links: vec![format!("{scheme}://new")],
        };
        for request in [&early, &later] {
            assert!(matches!(
                acquire(&endpoint, request).await,
                Instance::Forwarded
            ));
        }
        assert_eq!(window.recv().await, Some(early));
        assert_eq!(window.recv().await, Some(later));
        assert!(
            window.try_recv().is_err(),
            "a taken start is not delivered again"
        );
        assert!(
            tasks
                .shutdown_or_abort(Duration::from_secs(1))
                .await
                .is_drained()
        );
        Ok(())
    })
}
