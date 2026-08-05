//! The Handlebars helpers the admin `.hbs` templates are written against.
//!
//! Every one of these formats a number, a date, or a decision that an operator
//! reads off a governance page, so the exact output string is the contract.

use handlebars::Handlebars;
use systemprompt_web_admin::templates::helpers::register_helpers;

fn engine() -> Handlebars<'static> {
    let mut hbs = Handlebars::new();
    hbs.set_strict_mode(false);
    register_helpers(&mut hbs);
    hbs
}

fn render(template: &str, data: &serde_json::Value) -> String {
    engine()
        .render_template(template, data)
        .unwrap_or_else(|e| panic!("render {template}: {e}"))
}

fn render_value(template: &str, value: serde_json::Value) -> String {
    render(template, &serde_json::json!({ "v": value }))
}

#[test]
fn format_number_abbreviates_by_magnitude() {
    let cases = [
        (serde_json::json!(0), "0"),
        (serde_json::json!(999), "999"),
        (serde_json::json!(1_000), "1,000"),
        (serde_json::json!(12_345), "12,345"),
        (serde_json::json!(999_999), "999,999"),
        (serde_json::json!(1_500_000), "1.5M"),
        (serde_json::json!(2_000_000_000_i64), "2.0B"),
        (serde_json::json!(-12_345), "-12,345"),
        (serde_json::json!(-1_500_000), "-1.5M"),
    ];
    for (input, expected) in cases {
        assert_eq!(
            render_value("{{formatNumber v}}", input.clone()),
            expected,
            "{input}"
        );
    }
}

#[test]
fn format_number_treats_a_missing_or_non_numeric_value_as_zero() {
    assert_eq!(
        render("{{formatNumber missing}}", &serde_json::json!({})),
        "0"
    );
    assert_eq!(
        render_value("{{formatNumber v}}", serde_json::json!("lots")),
        "0"
    );
}

#[test]
fn format_usd_picks_precision_from_magnitude() {
    let cases = [
        (serde_json::json!(0), "$0"),
        (serde_json::json!(1_500), "$0.00150"),
        (serde_json::json!(50_000), "$0.050"),
        (serde_json::json!(2_500_000_i64), "$2.50"),
        (serde_json::json!(250_000_000_i64), "$250"),
        (serde_json::json!(-2_500_000_i64), "-$2.50"),
        (serde_json::json!(-250_000_000_i64), "-$250"),
    ];
    for (input, expected) in cases {
        assert_eq!(
            render_value("{{formatUsd v}}", input.clone()),
            expected,
            "{input}"
        );
    }
}

#[test]
fn format_usd_renders_an_absent_cost_as_an_em_dash() {
    assert_eq!(render("{{formatUsd missing}}", &serde_json::json!({})), "—");
    assert_eq!(
        render_value("{{formatUsd v}}", serde_json::Value::Null),
        "—"
    );
}

#[test]
fn percent_scales_a_fraction_to_one_decimal() {
    assert_eq!(
        render_value("{{percent v}}", serde_json::json!(0.5)),
        "50.0%"
    );
    assert_eq!(
        render_value("{{percent v}}", serde_json::json!(0.1234)),
        "12.3%"
    );
    assert_eq!(
        render_value("{{percent v}}", serde_json::json!(1)),
        "100.0%"
    );
    assert_eq!(
        render("{{percent missing}}", &serde_json::json!({})),
        "0.0%"
    );
}

#[test]
fn delta_pct_signs_the_change_and_blanks_an_undefined_one() {
    let data = serde_json::json!({ "up": 150, "down": 50, "base": 100, "zero": 0 });
    assert_eq!(render("{{deltaPct up base}}", &data), "+50% vs prev");
    assert_eq!(render("{{deltaPct down base}}", &data), "-50% vs prev");
    assert_eq!(render("{{deltaPct base base}}", &data), "0% vs prev");
    assert_eq!(render("{{deltaPct up zero}}", &data), "");
    assert_eq!(render("{{deltaPct up missing}}", &data), "");
    assert_eq!(render("{{deltaPct}}", &data), "");
}

#[test]
fn governance_color_maps_each_decision_family() {
    let cases = [
        ("allow", "success"),
        ("PASS", "success"),
        ("ok", "success"),
        ("flag", "warning"),
        ("Warn", "warning"),
        ("review", "warning"),
        ("deny", "danger"),
        ("BLOCK", "danger"),
        ("fail", "danger"),
        ("error", "danger"),
        ("something-else", "neutral"),
        ("", "neutral"),
    ];
    for (decision, color) in cases {
        assert_eq!(
            render_value("{{governanceColor v}}", serde_json::json!(decision)),
            color,
            "{decision}"
        );
    }
    assert_eq!(
        render("{{governanceColor missing}}", &serde_json::json!({})),
        "neutral"
    );
}

#[test]
fn default_substitutes_only_for_falsy_values() {
    let data = serde_json::json!({
        "name": "Ada", "blank": "", "nil": null, "yes": true, "no": false, "n": 7,
    });
    assert_eq!(render("{{default name \"-\"}}", &data), "Ada");
    assert_eq!(render("{{default blank \"-\"}}", &data), "-");
    assert_eq!(render("{{default nil \"-\"}}", &data), "-");
    assert_eq!(render("{{default missing \"-\"}}", &data), "-");
    assert_eq!(render("{{default no \"-\"}}", &data), "-");
    assert_eq!(render("{{default yes \"-\"}}", &data), "true");
    assert_eq!(render("{{default n \"-\"}}", &data), "7");
    assert_eq!(render("{{default blank}}", &data), "");
}

#[test]
fn json_pretty_prints_and_escapes_html() {
    let out = render_value("{{{json v}}}", serde_json::json!({ "a": 1 }));
    assert!(out.contains("\"a\": 1"), "{out}");

    let escaped = render_value("{{{json v}}}", serde_json::json!("<script>&"));
    assert!(escaped.contains("&lt;script&gt;"), "{escaped}");
    assert!(escaped.contains("&amp;"), "{escaped}");
    assert!(!escaped.contains('<'), "{escaped}");
}

#[test]
fn json_renders_a_missing_param_as_null() {
    assert_eq!(render("{{{json missing}}}", &serde_json::json!({})), "null");
}

#[test]
fn initials_takes_the_first_letter_of_up_to_two_name_parts() {
    let cases = [
        ("Ada Lovelace", "AL"),
        ("ada.lovelace@example.test", "AL"),
        ("ada_lovelace", "AL"),
        ("ada-lovelace", "AL"),
        ("Ada Byron Lovelace", "AB"),
        ("ada", "A"),
        ("", "?"),
        ("?", "?"),
        ("   ", "?"),
    ];
    for (name, expected) in cases {
        assert_eq!(
            render_value("{{initials v}}", serde_json::json!(name)),
            expected,
            "{name}"
        );
    }
    assert_eq!(render("{{initials missing}}", &serde_json::json!({})), "?");
}

#[test]
fn truncate_appends_an_ellipsis_only_past_the_limit() {
    let data = serde_json::json!({ "short": "abc", "long": "abcdefghij" });
    assert_eq!(render("{{truncate short 5}}", &data), "abc");
    assert_eq!(render("{{truncate long 5}}", &data), "abcde...");
    assert_eq!(render("{{truncate long}}", &data), "abcdefghij");
    assert_eq!(render("{{truncate missing 5}}", &data), "");
}

#[test]
fn concat_joins_scalars_and_skips_nulls() {
    let data = serde_json::json!({ "a": "x", "n": 2, "b": true, "nil": null });
    assert_eq!(render("{{concat a n b nil}}", &data), "x2true");
    assert_eq!(render("{{concat}}", &data), "");
}

#[test]
fn case_helpers_fold_both_ways() {
    let data = serde_json::json!({ "v": "MiXeD" });
    assert_eq!(render("{{toLowerCase v}}", &data), "mixed");
    assert_eq!(render("{{toUpperCase v}}", &data), "MIXED");
    assert_eq!(render("{{toLowerCase missing}}", &data), "");
}

#[test]
fn short_id_clips_to_a_character_count() {
    let data = serde_json::json!({ "id": "0123456789abcdefghij" });
    assert_eq!(render("{{shortId id 4}}", &data), "0123");
    assert_eq!(render("{{shortId id}}", &data), "0123456789ab");
    assert_eq!(render("{{shortId missing 4}}", &data), "");
    assert_eq!(render("{{shortId id 99}}", &data), "0123456789abcdefghij");
}

#[test]
fn eq_compares_values_of_any_type() {
    let data = serde_json::json!({ "a": "x", "b": "x", "c": "y", "n": 1 });
    assert_eq!(
        render("{{#if (eq a b)}}same{{else}}differ{{/if}}", &data),
        "same"
    );
    assert_eq!(
        render("{{#if (eq a c)}}same{{else}}differ{{/if}}", &data),
        "differ"
    );
    assert_eq!(
        render("{{#if (eq a n)}}same{{else}}differ{{/if}}", &data),
        "differ"
    );
    assert_eq!(
        render("{{#if (eq p q)}}same{{else}}differ{{/if}}", &data),
        "same"
    );
    assert_eq!(
        render("{{#if (eq a p)}}same{{else}}differ{{/if}}", &data),
        "differ"
    );
}

#[test]
fn gt_compares_numerically_and_treats_absent_as_zero() {
    let data = serde_json::json!({ "hi": 5, "lo": 2, "neg": -1 });
    assert_eq!(render("{{#if (gt hi lo)}}y{{else}}n{{/if}}", &data), "y");
    assert_eq!(render("{{#if (gt lo hi)}}y{{else}}n{{/if}}", &data), "n");
    assert_eq!(render("{{#if (gt hi hi)}}y{{else}}n{{/if}}", &data), "n");
    assert_eq!(
        render("{{#if (gt hi missing)}}y{{else}}n{{/if}}", &data),
        "y"
    );
    assert_eq!(
        render("{{#if (gt neg missing)}}y{{else}}n{{/if}}", &data),
        "n"
    );
}

#[test]
fn not_reports_falsiness_per_json_type() {
    let data = serde_json::json!({
        "nil": null, "no": false, "yes": true, "blank": "", "text": "x",
        "zero": 0, "one": 1, "empty": [], "full": [1], "obj": {},
    });
    for (key, expected) in [
        ("nil", "y"),
        ("no", "y"),
        ("blank", "y"),
        ("zero", "y"),
        ("empty", "y"),
        ("missing", "y"),
        ("yes", "n"),
        ("text", "n"),
        ("one", "n"),
        ("full", "n"),
        ("obj", "n"),
    ] {
        let out = render(
            &format!("{{{{#if (not {key})}}}}y{{{{else}}}}n{{{{/if}}}}"),
            &data,
        );
        assert_eq!(out, expected, "{key}");
    }
}

#[test]
fn add_and_sub_work_on_integers_and_default_missing_to_zero() {
    let data = serde_json::json!({ "a": 7, "b": 3 });
    assert_eq!(render("{{add a b}}", &data), "10");
    assert_eq!(render("{{sub a b}}", &data), "4");
    assert_eq!(render("{{sub b a}}", &data), "-4");
    assert_eq!(render("{{add a missing}}", &data), "7");
    assert_eq!(render("{{sub missing a}}", &data), "-7");
}

#[test]
fn format_date_passes_through_what_it_cannot_parse() {
    assert_eq!(
        render_value("{{formatDate v}}", serde_json::json!("-")),
        "-"
    );
    assert_eq!(render_value("{{formatDate v}}", serde_json::json!("")), "-");
    assert_eq!(
        render("{{formatDate missing}}", &serde_json::json!({})),
        "-"
    );
    assert_eq!(
        render_value("{{formatDate v}}", serde_json::json!("not a date")),
        "not a date"
    );
}

#[test]
fn format_date_accepts_both_rfc3339_and_naive_timestamps() {
    for input in ["2026-03-04T05:06:07Z", "2026-03-04T05:06:07.123"] {
        let out = render_value("{{formatDate v}}", serde_json::json!(input));
        assert!(out.contains("2026"), "{input} -> {out}");
        assert!(out.contains("Mar"), "{input} -> {out}");
        assert_ne!(out, input);
    }
}

#[test]
fn relative_time_buckets_by_age() {
    let now = chrono::Utc::now();
    let cases = [
        (now, "just now"),
        (now - chrono::Duration::minutes(5), "5m ago"),
        (now - chrono::Duration::hours(3), "3h ago"),
        (now - chrono::Duration::days(4), "4d ago"),
    ];
    for (instant, expected) in cases {
        let out = render_value(
            "{{relativeTime v}}",
            serde_json::json!(instant.to_rfc3339()),
        );
        assert_eq!(out, expected);
    }
}

#[test]
fn relative_time_falls_back_to_an_absolute_date_past_thirty_days() {
    let old = chrono::Utc::now() - chrono::Duration::days(400);
    let out = render_value("{{relativeTime v}}", serde_json::json!(old.to_rfc3339()));
    assert!(!out.ends_with("d ago"), "{out}");
    assert!(out.contains(&old.format("%Y").to_string()), "{out}");
}

#[test]
fn relative_time_passes_through_what_it_cannot_parse() {
    assert_eq!(
        render_value("{{relativeTime v}}", serde_json::json!("-")),
        "-"
    );
    assert_eq!(
        render_value("{{relativeTime v}}", serde_json::json!("")),
        "-"
    );
    assert_eq!(
        render("{{relativeTime missing}}", &serde_json::json!({})),
        "-"
    );
    assert_eq!(
        render_value("{{relativeTime v}}", serde_json::json!("soon")),
        "soon"
    );
}

#[test]
fn css_version_always_renders_something_cache_bustable() {
    let out = render("{{css_version}}", &serde_json::json!({}));
    assert!(!out.is_empty());
    assert!(!out.contains(' '), "{out}");
}
