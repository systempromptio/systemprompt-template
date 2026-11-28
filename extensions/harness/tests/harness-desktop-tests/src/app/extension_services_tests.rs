use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use systemprompt_harness::extension::{
    BackgroundService, EnabledExtensions, Enablement, ExtensionMetadata, SpExtension, SpExtensions,
};
use systemprompt_harness_views::app::extension_services::AdmittedService;
use systemprompt_identifiers::engine::ExtensionId;
use systemprompt_identifiers::extension_id;
use systemprompt_traits::{CancellationToken, OwnedTask};

use crate::TestResult;

#[derive(Default)]
struct Counting {
    starts: AtomicUsize,
    stops: AtomicUsize,
}

impl Counting {
    fn counts(&self) -> (usize, usize) {
        (
            self.starts.load(Ordering::SeqCst),
            self.stops.load(Ordering::SeqCst),
        )
    }
}

#[async_trait::async_trait]
impl BackgroundService for Counting {
    fn name(&self) -> &'static str {
        "counting"
    }

    async fn run(&self, cancel: CancellationToken) {
        self.starts.fetch_add(1, Ordering::SeqCst);
        cancel.cancelled().await;
        self.stops.fetch_add(1, Ordering::SeqCst);
    }
}

struct Plain(ExtensionMetadata);

impl SpExtension for Plain {
    fn metadata(&self) -> ExtensionMetadata {
        self.0.clone()
    }
}

fn extensions() -> TestResult<SpExtensions> {
    let linked: [Arc<dyn SpExtension>; 2] = [
        Arc::new(Plain(ExtensionMetadata::new(
            extension_id!("notifications"),
            "notifications",
        ))),
        Arc::new(Plain(
            ExtensionMetadata::new(extension_id!("slack"), "slack")
                .depends_on(extension_id!("notifications")),
        )),
    ];
    Ok(SpExtensions::from_extensions(linked)?)
}

fn enabled(ids: &[ExtensionId]) -> EnabledExtensions {
    ids.iter().cloned().collect()
}

async fn eventually(service: &Counting, wanted: (usize, usize)) {
    for _ in 0..400 {
        if service.counts() == wanted {
            return;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    assert_eq!(service.counts(), wanted);
}

fn start(
    extension: ExtensionId,
    service: &Arc<Counting>,
    enablement: &Enablement,
    cancel: &CancellationToken,
) -> TestResult<OwnedTask<()>> {
    let admitted = AdmittedService {
        extension,
        service: Arc::<Counting>::clone(service),
        extensions: extensions()?,
    };
    Ok(OwnedTask::spawn(
        "admitted-service",
        admitted.run(enablement.subscribe(), cancel.clone()),
    ))
}

#[tokio::test]
async fn a_service_runs_only_while_its_extension_is_admitted() -> TestResult {
    let service = Arc::new(Counting::default());
    let enablement = Enablement::new();
    let cancel = CancellationToken::new();
    let task = start(
        extension_id!("notifications"),
        &service,
        &enablement,
        &cancel,
    )?;

    tokio::time::sleep(Duration::from_millis(30)).await;
    assert_eq!(service.counts(), (0, 0));

    enablement.set(Some(enabled(&[extension_id!("notifications")])));
    eventually(&service, (1, 0)).await;

    enablement.set(Some(enabled(&[])));
    eventually(&service, (1, 1)).await;

    enablement.set(Some(enabled(&[extension_id!("notifications")])));
    eventually(&service, (2, 1)).await;

    cancel.cancel();
    task.join_named().await?;
    assert_eq!(service.counts(), (2, 2));
    Ok(())
}

#[tokio::test]
async fn a_service_whose_dependency_is_withheld_does_not_run() -> TestResult {
    let service = Arc::new(Counting::default());
    let enablement = Enablement::new();
    let cancel = CancellationToken::new();
    let task = start(extension_id!("slack"), &service, &enablement, &cancel)?;

    enablement.set(Some(enabled(&[extension_id!("slack")])));
    tokio::time::sleep(Duration::from_millis(30)).await;
    assert_eq!(service.counts(), (0, 0));

    enablement.set(Some(enabled(&[
        extension_id!("slack"),
        extension_id!("notifications"),
    ])));
    eventually(&service, (1, 0)).await;

    cancel.cancel();
    task.join_named().await?;
    assert_eq!(service.counts(), (1, 1));
    Ok(())
}
