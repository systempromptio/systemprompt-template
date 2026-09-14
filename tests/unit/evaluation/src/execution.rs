use std::collections::BTreeMap;
use systemprompt_evaluation::experiments::execution::{
    ArtifactFile, EvidenceArchive, ExecutionLimits,
};

#[test]
fn evidence_archive_rejects_portable_path_escapes_and_limits_content() {
    for path in ["../a", "/a", "C:/a", "a\\b", "a//b", "a/./b", "a\nb"] {
        assert!(
            EvidenceArchive {
                files: BTreeMap::from([(
                    path.into(),
                    ArtifactFile {
                        bytes: b"x".to_vec(),
                        executable: false
                    }
                )])
            }
            .validate()
            .is_err()
        );
    }
    let mut workspace = EvidenceArchive {
        files: BTreeMap::from([(
            "references/context.md".into(),
            ArtifactFile {
                bytes: b"source".to_vec(),
                executable: false,
            },
        )]),
    };
    let before = workspace.digest().unwrap();
    workspace.files.insert(
        "references/context.md".into(),
        ArtifactFile {
            bytes: b"changed source".to_vec(),
            executable: false,
        },
    );
    assert_ne!(before, workspace.digest().unwrap());
    workspace.files.insert(
        "large".into(),
        ArtifactFile {
            bytes: vec![b'x'; 16 * 1024 * 1024],
            executable: false,
        },
    );
    assert!(workspace.validate().is_err());
    assert!(
        ExecutionLimits {
            max_turns: 0,
            ..Default::default()
        }
        .validate()
        .is_err()
    );
}

#[cfg(unix)]
#[test]
fn supervised_owned_child_preserves_exit_status_and_output() {
    use std::process::{Command, Stdio};
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "printf supervised; exit 7"])
        .stdout(Stdio::piped());
    let child = systemprompt::models::subprocess::spawn_owned_supervised(command).unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(output.stdout, b"supervised");
}

#[test]
fn evaluation_pricing_rejects_unknown_and_nonfinite_rates() {
    use systemprompt::models::services::{ModelPricing, ProviderRegistry};
    use systemprompt_api::services::gateway::evaluation::request_bound;
    use systemprompt_api::services::gateway::pricing::resolve;
    assert!(resolve("missing", &["unpriced"], None, &ProviderRegistry::default()).is_err());
    assert!(request_bound(&ModelPricing::default(), 100, 10).is_err());
    let pricing = ModelPricing {
        input_per_million: 1.0,
        output_per_million: 5.0,
        ..Default::default()
    };
    assert_eq!(request_bound(&pricing, 100, 10).unwrap(), 4246);
    assert!(
        request_bound(
            &ModelPricing {
                input_per_million: f64::NAN,
                ..pricing
            },
            100,
            10
        )
        .is_err()
    );
}

#[test]
fn execution_events_and_client_capabilities_reject_unbounded_or_invalid_input() {
    use systemprompt_evaluation::experiments::ClientKind;
    use systemprompt_evaluation::experiments::execution::ClientCapabilities;
    use systemprompt_evaluation::repository::experiments::{ExecutionEvent, ExecutionStage};
    for (sequence, summary) in [
        (-1, "started".to_owned()),
        (1000, "started".to_owned()),
        (0, " ".to_owned()),
        (0, "x".repeat(8193)),
    ] {
        assert!(
            ExecutionEvent::builder(sequence, ExecutionStage::Context)
                .summary(summary)
                .build()
                .is_err()
        );
    }
    assert!(
        ExecutionEvent::builder(999, ExecutionStage::Cleanup)
            .summary("Container removal verified".into())
            .build()
            .is_ok()
    );
    for version in ["".to_owned(), "\n".to_owned(), "x".repeat(129)] {
        assert!(
            ClientCapabilities::builder()
                .client(ClientKind::ClaudeCode)
                .client_version(version)
                .adapter_version("1".into())
                .image_digest("a".repeat(64))
                .supports_session_resume(false)
                .build()
                .is_err()
        );
    }
    assert!(
        ClientCapabilities::builder()
            .client(ClientKind::ClaudeCode)
            .client_version("2.1.71".into())
            .adapter_version("1".into())
            .image_digest("A".repeat(64))
            .supports_session_resume(false)
            .build()
            .is_err()
    );
}
