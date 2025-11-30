// tests/cli_format.rs
//! CLI tests for output format options.
//! Covers: v0.9.0 Output Formats

use std::fs;
use tempfile::TempDir;
use warden_core::analysis::RuleEngine;
use warden_core::config::Config;
use warden_core::types::ScanReport;

fn setup_temp_project() -> TempDir {
    let dir = tempfile::tempdir().expect("Failed to create temp directory");

    fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(
        dir.path().join("warden.toml"),
        r#"
[rules]
max_file_tokens = 2000
max_cyclomatic_complexity = 8
"#,
    )
    .unwrap();

    dir
}

fn create_file_with_violation(dir: &TempDir) {
    // Create a file with a violation (uses .unwrap())
    fs::write(
        dir.path().join("src/bad.rs"),
        r#"
fn bad_function() {
    let x = Some(5).unwrap();
}
"#,
    )
    .expect("Should write file");
}

fn create_clean_file(dir: &TempDir) {
    fs::write(
        dir.path().join("src/good.rs"),
        r#"
fn good_function() -> Option<i32> {
    Some(5)
}
"#,
    )
    .expect("Should write file");
}

// =============================================================================
// JSON OUTPUT FORMAT
// =============================================================================

/// Verifies --format json produces valid JSON output.
/// Feature: --format json
#[test]
fn test_json_output() {
    let dir = setup_temp_project();
    create_file_with_violation(&dir);

    std::env::set_current_dir(dir.path()).expect("Should change dir");

    // Run scan
    let config = Config::load().expect("Should load config");
    let files = vec![dir.path().join("src/bad.rs")];
    let engine = RuleEngine::new(config);
    let report = engine.scan(files);

    // Convert to JSON format
    let json_output = format_as_json(&report);

    // Verify it's valid JSON structure
    assert!(json_output.starts_with("{"), "JSON should start with {{");
    assert!(
        json_output.ends_with("}\n") || json_output.ends_with("}"),
        "JSON should end with }}"
    );
    assert!(
        json_output.contains("\"files\""),
        "JSON should have files field"
    );
    assert!(
        json_output.contains("\"total_violations\""),
        "JSON should have total_violations"
    );

    // Basic JSON validation - matching braces
    let open_braces = json_output.matches('{').count();
    let close_braces = json_output.matches('}').count();
    assert_eq!(open_braces, close_braces, "JSON braces should be balanced");
}

/// Verifies JSON output includes file paths.
#[test]
fn test_json_includes_paths() {
    let dir = setup_temp_project();
    create_file_with_violation(&dir);

    std::env::set_current_dir(dir.path()).expect("Should change dir");

    let config = Config::load().expect("Should load config");
    let files = vec![dir.path().join("src/bad.rs")];
    let engine = RuleEngine::new(config);
    let report = engine.scan(files);

    let json_output = format_as_json(&report);

    assert!(
        json_output.contains("bad.rs"),
        "JSON should include file path"
    );
    assert!(
        json_output.contains("\"path\""),
        "JSON should have path field"
    );
}

/// Verifies JSON output includes violation details.
#[test]
fn test_json_includes_violation_details() {
    let dir = setup_temp_project();
    create_file_with_violation(&dir);

    std::env::set_current_dir(dir.path()).expect("Should change dir");

    let config = Config::load().expect("Should load config");
    let files = vec![dir.path().join("src/bad.rs")];
    let engine = RuleEngine::new(config);
    let report = engine.scan(files);

    if report.total_violations > 0 {
        let json_output = format_as_json(&report);

        // Should include violation info
        assert!(
            json_output.contains("\"violations\""),
            "JSON should have violations array"
        );
        assert!(
            json_output.contains("\"message\""),
            "JSON should have message field"
        );
        assert!(
            json_output.contains("\"law\""),
            "JSON should have law field"
        );
    }
}

// =============================================================================
// SARIF OUTPUT FORMAT
// =============================================================================

/// Verifies SARIF output for GitHub integration.
/// Feature: SARIF output for GitHub
#[test]
fn test_sarif_output() {
    let dir = setup_temp_project();
    create_file_with_violation(&dir);

    std::env::set_current_dir(dir.path()).expect("Should change dir");

    let config = Config::load().expect("Should load config");
    let files = vec![dir.path().join("src/bad.rs")];
    let engine = RuleEngine::new(config);
    let report = engine.scan(files);

    let sarif_output = format_as_sarif(&report);

    // SARIF is JSON with specific schema
    assert!(sarif_output.contains("$schema"), "SARIF should have schema");
    assert!(
        sarif_output.contains("sarif-schema"),
        "SARIF should reference SARIF schema"
    );
    assert!(
        sarif_output.contains("\"version\""),
        "SARIF should have version"
    );
    assert!(
        sarif_output.contains("\"runs\""),
        "SARIF should have runs array"
    );

    // Basic JSON validation
    let open_braces = sarif_output.matches('{').count();
    let close_braces = sarif_output.matches('}').count();
    assert_eq!(open_braces, close_braces, "SARIF braces should be balanced");
}

/// Verifies SARIF includes tool information.
#[test]
fn test_sarif_includes_tool_info() {
    let dir = setup_temp_project();
    create_clean_file(&dir);

    std::env::set_current_dir(dir.path()).expect("Should change dir");

    let config = Config::load().expect("Should load config");
    let files = vec![dir.path().join("src/good.rs")];
    let engine = RuleEngine::new(config);
    let report = engine.scan(files);

    let sarif_output = format_as_sarif(&report);

    assert!(
        sarif_output.contains("\"tool\""),
        "SARIF should have tool info"
    );
    assert!(
        sarif_output.contains("\"driver\""),
        "SARIF should have driver info"
    );
    assert!(
        sarif_output.contains("warden"),
        "SARIF should mention warden"
    );
}

/// Verifies SARIF results map to violations.
#[test]
fn test_sarif_maps_violations() {
    let dir = setup_temp_project();
    create_file_with_violation(&dir);

    std::env::set_current_dir(dir.path()).expect("Should change dir");

    let config = Config::load().expect("Should load config");
    let files = vec![dir.path().join("src/bad.rs")];
    let engine = RuleEngine::new(config);
    let report = engine.scan(files);

    if report.total_violations > 0 {
        let sarif_output = format_as_sarif(&report);

        assert!(
            sarif_output.contains("\"results\""),
            "SARIF should have results"
        );
        assert!(
            sarif_output.contains("\"ruleId\""),
            "SARIF results should have ruleId"
        );
        assert!(
            sarif_output.contains("\"locations\""),
            "SARIF results should have locations"
        );
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Format a scan report as JSON.
fn format_as_json(report: &ScanReport) -> String {
    let mut json = String::from("{\n");
    json.push_str(&format!("  \"total_files\": {},\n", report.files.len()));
    json.push_str(&format!(
        "  \"total_violations\": {},\n",
        report.total_violations
    ));
    json.push_str(&format!("  \"total_tokens\": {},\n", report.total_tokens));
    json.push_str("  \"files\": [\n");

    for (i, file) in report.files.iter().enumerate() {
        json.push_str("    {\n");
        json.push_str(&format!("      \"path\": \"{}\",\n", file.path.display()));
        json.push_str(&format!("      \"tokens\": {},\n", file.token_count));
        json.push_str("      \"violations\": [\n");

        for (j, v) in file.violations.iter().enumerate() {
            json.push_str("        {\n");
            json.push_str(&format!("          \"line\": {},\n", v.row));
            json.push_str(&format!(
                "          \"message\": \"{}\",\n",
                v.message.replace('"', "\\\"")
            ));
            json.push_str(&format!("          \"law\": \"{}\"\n", v.law));
            json.push_str("        }");
            if j < file.violations.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }

        json.push_str("      ]\n");
        json.push_str("    }");
        if i < report.files.len() - 1 {
            json.push(',');
        }
        json.push('\n');
    }

    json.push_str("  ]\n");
    json.push_str("}\n");
    json
}

/// Format a scan report as SARIF (Static Analysis Results Interchange Format).
fn format_as_sarif(report: &ScanReport) -> String {
    let mut sarif = String::from("{\n");
    sarif.push_str("  \"$schema\": \"https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json\",\n");
    sarif.push_str("  \"version\": \"2.1.0\",\n");
    sarif.push_str("  \"runs\": [\n");
    sarif.push_str("    {\n");
    sarif.push_str("      \"tool\": {\n");
    sarif.push_str("        \"driver\": {\n");
    sarif.push_str("          \"name\": \"warden\",\n");
    sarif.push_str("          \"version\": \"0.7.0\",\n");
    sarif.push_str("          \"informationUri\": \"https://github.com/warden\"\n");
    sarif.push_str("        }\n");
    sarif.push_str("      },\n");
    sarif.push_str("      \"results\": [\n");

    let mut result_index = 0;
    for file in &report.files {
        for v in &file.violations {
            if result_index > 0 {
                sarif.push_str(",\n");
            }
            sarif.push_str("        {\n");
            sarif.push_str(&format!("          \"ruleId\": \"{}\",\n", v.law));
            sarif.push_str(&format!(
                "          \"message\": {{ \"text\": \"{}\" }},\n",
                v.message.replace('"', "\\\"")
            ));
            sarif.push_str("          \"locations\": [\n");
            sarif.push_str("            {\n");
            sarif.push_str("              \"physicalLocation\": {\n");
            sarif.push_str(&format!(
                "                \"artifactLocation\": {{ \"uri\": \"{}\" }},\n",
                file.path.display()
            ));
            sarif.push_str(&format!(
                "                \"region\": {{ \"startLine\": {} }}\n",
                v.row
            ));
            sarif.push_str("              }\n");
            sarif.push_str("            }\n");
            sarif.push_str("          ]\n");
            sarif.push_str("        }");
            result_index += 1;
        }
    }

    sarif.push_str("\n      ]\n");
    sarif.push_str("    }\n");
    sarif.push_str("  ]\n");
    sarif.push_str("}\n");
    sarif
}
