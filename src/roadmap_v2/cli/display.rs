use crate::roadmap_v2::types::{SectionStatus, TaskStatus, TaskStore};
use crate::roadmap_v2::RoadmapCommand;
use anyhow::{anyhow, Result};
use colored::Colorize;

pub fn print_stats(store: &TaskStore) {
    let total = store.tasks.len();
    let done = store.tasks.iter().filter(|t| t.status == TaskStatus::Done).count();
    let pending = total - done;
    println!("Tasks: {total} ({done} done, {pending} pending)");
}

pub fn print_tree(store: &TaskStore) {
    println!("{}", store.meta.title.cyan().bold());
    println!();

    for section in &store.sections {
        print_section(store, section);
    }
}

fn print_section(store: &TaskStore, section: &crate::roadmap_v2::types::Section) {
    let status_icon = match section.status {
        SectionStatus::Complete => "✓".green(),
        SectionStatus::Current => "→".yellow(),
        SectionStatus::Pending => "○".dimmed(),
    };
    println!("{status_icon} {}", section.title.bold());

    let section_tasks: Vec<_> = store.tasks.iter()
        .filter(|t| t.section == section.id)
        .collect();

    for task in section_tasks {
        print_task(task);
    }
    println!();
}

fn print_task(task: &crate::roadmap_v2::types::Task) {
    let mark = match task.status {
        TaskStatus::Done | TaskStatus::NoTest => "[x]".green(),
        TaskStatus::Pending => "[ ]".dimmed(),
    };
    let test_info = task.test.as_ref().map_or(String::new(), |t| {
        format!(" {}", format!("({t})").dimmed())
    });
    println!("    {mark} {}{test_info}", task.text);
}

pub fn print_dry_run(commands: &[RoadmapCommand]) {
    println!("{}", "[DRY RUN]".yellow());
    for cmd in commands {
        println!("  Would apply: {cmd:?}");
    }
}

pub fn print_audit_header() {
    println!("{}", " 🕵️  Roadmap Traceability Audit ".cyan().bold());
    println!("{}", "─────────────────────────────────────".dimmed());
}

pub fn print_audit_failure(text: &str, id: &str, reason: &str) {
    println!(
        "{} Traceability Fail: {} (id: {})",
        "⚠️ ".yellow(),
        text,
        id.dimmed()
    );
    println!("   └─ {reason}");
}

pub fn print_audit_result(failures: usize, strict: bool) -> Result<()> {
    if failures > 0 {
        println!(
            "{} Found {} task(s) without verified tests.",
            "❌".red(),
            failures
        );
        println!(
            "{}",
            "   (Tip: Add test = \"tests/my_test.rs::fn_name\" to tasks.toml)".dimmed()
        );
        if strict {
            return Err(anyhow!("Audit failed in strict mode."));
        }
    } else {
        println!("{} All tasks have verified test coverage!", "✅".green());
    }

    Ok(())
}