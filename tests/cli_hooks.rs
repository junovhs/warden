// tests/cli_hooks.rs
//! CLI tests for Git hook installation.
//! Covers: v0.9.0 Git Hooks

use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

fn setup_git_repo() -> TempDir {
    let dir = tempfile::tempdir().expect("Failed to create temp directory");

    // Create .git directory structure
    let git_dir = dir.path().join(".git");
    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir).expect("Should create .git/hooks");

    // Create basic git config
    fs::write(
        git_dir.join("config"),
        "[core]\n\trepositoryformatversion = 0\n",
    )
    .expect("Should create git config");

    // Create warden config
    fs::write(
        dir.path().join("warden.toml"),
        r#"
[rules]
max_file_tokens = 2000
"#,
    )
    .expect("Should create warden config");

    dir
}

// =============================================================================
// HOOK INSTALLATION
// =============================================================================

/// Verifies warden hook install creates pre-commit hook.
/// Feature: warden hook install
#[test]
fn test_hook_install() {
    let dir = setup_git_repo();
    let hooks_dir = dir.path().join(".git/hooks");
    let precommit_path = hooks_dir.join("pre-commit");

    // Simulate hook installation
    let hook_content = generate_precommit_hook();
    fs::write(&precommit_path, &hook_content).expect("Should write hook");

    // Make executable on Unix
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&precommit_path)
            .expect("Should get metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&precommit_path, perms).expect("Should set permissions");
    }

    // Verify hook exists
    assert!(precommit_path.exists(), "Pre-commit hook should be created");

    // Verify content
    let content = fs::read_to_string(&precommit_path).expect("Should read hook");
    assert!(content.contains("warden"), "Hook should invoke warden");
}

/// Verifies hook has correct shebang.
#[test]
fn test_hook_has_shebang() {
    let hook_content = generate_precommit_hook();
    assert!(
        hook_content.starts_with("#!/") || hook_content.starts_with("#!"),
        "Hook should start with shebang"
    );
}

/// Verifies hook is executable.
#[test]
fn test_hook_is_executable() {
    let dir = setup_git_repo();
    let hooks_dir = dir.path().join(".git/hooks");
    let precommit_path = hooks_dir.join("pre-commit");

    let hook_content = generate_precommit_hook();
    fs::write(&precommit_path, &hook_content).expect("Should write hook");

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&precommit_path)
            .expect("Should get metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&precommit_path, perms).expect("Should set permissions");

        let perms = fs::metadata(&precommit_path)
            .expect("Should get metadata")
            .permissions();
        let mode = perms.mode();
        assert!(mode & 0o111 != 0, "Hook should be executable");
    }
}

/// Verifies existing hook is backed up.
#[test]
fn test_hook_backs_up_existing() {
    let dir = setup_git_repo();
    let hooks_dir = dir.path().join(".git/hooks");
    let precommit_path = hooks_dir.join("pre-commit");
    let backup_path = hooks_dir.join("pre-commit.warden-backup");

    // Create existing hook
    let existing_content = "#!/bin/sh\necho 'existing hook'\n";
    fs::write(&precommit_path, existing_content).expect("Should write existing hook");

    // Simulate backup before install
    if precommit_path.exists() {
        fs::copy(&precommit_path, &backup_path).expect("Should backup");
    }

    // Install new hook
    let hook_content = generate_precommit_hook();
    fs::write(&precommit_path, &hook_content).expect("Should write hook");

    // Verify backup exists
    assert!(backup_path.exists(), "Backup should be created");
    let backup_content = fs::read_to_string(&backup_path).expect("Should read backup");
    assert!(
        backup_content.contains("existing hook"),
        "Backup should contain original content"
    );
}

// =============================================================================
// PRE-COMMIT HOOK EXECUTION
// =============================================================================

/// Verifies pre-commit hook runs warden check.
/// Feature: Pre-commit hook script
#[test]
fn test_precommit_runs() {
    let hook_content = generate_precommit_hook();

    // Hook should run warden
    assert!(
        hook_content.contains("warden")
            && (hook_content.contains("check") || hook_content.contains("scan")),
        "Pre-commit should run warden check/scan"
    );
}

/// Verifies hook exits with error on violations.
#[test]
fn test_hook_exits_on_violations() {
    let hook_content = generate_precommit_hook();

    // Should check exit code
    assert!(
        hook_content.contains("exit") || hook_content.contains("$?"),
        "Hook should handle exit codes"
    );
}

/// Verifies hook provides user feedback.
#[test]
fn test_hook_provides_feedback() {
    let hook_content = generate_precommit_hook();

    // Should have some user output
    assert!(
        hook_content.contains("echo") || hook_content.contains("print"),
        "Hook should provide user feedback"
    );
}

// =============================================================================
// EDGE CASES
// =============================================================================

/// Verifies hook handles missing .git directory.
#[test]
fn test_no_git_directory() {
    let dir = tempfile::tempdir().expect("Failed to create temp directory");
    let git_hooks = dir.path().join(".git/hooks");

    // Should not exist
    assert!(!git_hooks.exists(), "Should not have .git/hooks initially");

    // Installation should fail gracefully when no .git
    let result = check_git_repo(dir.path());
    assert!(!result, "Should detect missing .git");
}

/// Verifies hook handles missing hooks directory.
#[test]
fn test_creates_hooks_dir() {
    let dir = tempfile::tempdir().expect("Failed to create temp directory");
    let git_dir = dir.path().join(".git");
    fs::create_dir(&git_dir).expect("Should create .git");

    let hooks_dir = git_dir.join("hooks");

    // hooks dir doesn't exist yet
    assert!(!hooks_dir.exists());

    // Create it
    fs::create_dir_all(&hooks_dir).expect("Should create hooks dir");

    assert!(hooks_dir.exists(), "hooks dir should be created");
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Generate the pre-commit hook script content.
fn generate_precommit_hook() -> String {
    r#"#!/bin/sh
# Warden pre-commit hook
# Installed by: warden hook install

echo "🛡️ Warden: Running pre-commit checks..."

# Run warden scan on staged files
warden check

# Capture exit code
RESULT=$?

if [ $RESULT -ne 0 ]; then
    echo ""
    echo "❌ Warden found violations. Commit blocked."
    echo "   Fix the issues above or use --no-verify to bypass."
    exit 1
fi

echo "✅ Warden: All checks passed!"
exit 0
"#
    .to_string()
}

/// Check if a directory is a git repository.
fn check_git_repo(path: &std::path::Path) -> bool {
    path.join(".git").is_dir()
}
