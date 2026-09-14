//! India imports must retain complete source coverage and explicit membership.

use crate::support::repo_root;
use std::collections::BTreeSet;

#[test]
fn india_contains_exactly_the_two_imported_suites() {
    let root = repo_root();
    let inventory: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("scripts/india/skills-inventory.json")).unwrap(),
    )
    .unwrap();
    let skills = inventory["skills"].as_array().unwrap();
    assert_eq!(skills.len(), 34);
    let mut unique = BTreeSet::new();
    for (plugin, count) in [("astound-india-ba", 16), ("astound-india-sfcore", 18)] {
        let doc: serde_yaml::Value = serde_yaml::from_str(
            &std::fs::read_to_string(root.join(format!("services/plugins/{plugin}/config.yaml")))
                .unwrap(),
        )
        .unwrap();
        let selected: BTreeSet<_> = doc["plugin"]["skills"]["include"]
            .as_sequence()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap())
            .collect();
        let imported: BTreeSet<_> = skills
            .iter()
            .filter(|s| s["plugin"] == plugin)
            .map(|s| s["id"].as_str().unwrap())
            .collect();
        assert_eq!(selected, imported);
        assert_eq!(selected.len(), count);
        for skill in skills.iter().filter(|s| s["plugin"] == plugin) {
            assert!(unique.insert(skill["id"].as_str().unwrap()));
            let body =
                std::fs::read_to_string(root.join(skill["destination"].as_str().unwrap())).unwrap();
            assert!(body.contains("Systemprompt execution contract"));
            for obsolete in [
                "`read_csv_tracker`",
                "`push_to_jira`",
                "`send_email`",
                "`trigger_scheduled_job`",
                "`atlassian-rovo`",
                "Rovo",
                "advertised Atlassian capability for",
                "`searchJiraIssuesUsingJql`",
                "`createJiraIssue`",
                "`editJiraIssue`",
                "`getJiraIssue`",
                "`transitionJiraIssue`",
                "`getTransitionsForJiraIssue`",
                "`addCommentToJiraIssue`",
                "`createIssueLink`",
                "`getIssueLinkTypes`",
                "`getVisibleJiraProjects`",
                "`lookupJiraAccountId`",
                "`atlassianUserInfo`",
                "`searchConfluenceUsingCql`",
                "`getConfluencePage`",
                "`createConfluencePage`",
                "`updateConfluencePage`",
                "`getConfluenceSpaces`",
                ".cursor/skills/",
                "`Apex_Architect` agent",
                "Fetch a transcript by numeric ID",
                "Send an email via the configured Resend integration",
                "Update multiple issues in one call",
            ] {
                assert!(
                    !body.contains(obsolete),
                    "{} retains {obsolete}",
                    skill["id"]
                );
            }
        }
    }
}
