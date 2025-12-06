// slopchop:ignore
// src/roadmap/cmd_runner.rs
use crate::roadmap::cmd_handlers;
use crate::roadmap::types::{ApplyResult, Command, Roadmap};

pub fn run(roadmap: &mut Roadmap, cmds: &[Command]) -> Vec<ApplyResult> {
    cmds.iter()
        .map(|cmd| run_single(roadmap, cmd))
        .collect()
}

fn run_single(roadmap: &mut Roadmap, cmd: &Command) -> ApplyResult {
    let res = match cmd {
        Command::Check { path } => cmd_handlers::handle_check(roadmap, path),
        Command::Uncheck { path } => cmd_handlers::handle_uncheck(roadmap, path),
        Command::Delete { path } => cmd_handlers::handle_delete(roadmap, path),
        Command::Add { parent, text, after } => {
            cmd_handlers::handle_add(roadmap, parent, text, after.as_deref())
        }
        Command::AddSection { heading } => cmd_handlers::handle_add_section(roadmap, heading),
        _ => return run_single_ext(roadmap, cmd),
    };
    
    match res {
        Ok(_) => ApplyResult::Success(format!("Applied {cmd}")),
        Err(e) => ApplyResult::Error(format!("Failed {cmd}: {e}")),
    }
}

fn run_single_ext(roadmap: &mut Roadmap, cmd: &Command) -> ApplyResult {
    let res = match cmd {
        Command::Update { path, text } => cmd_handlers::handle_update(roadmap, path, text),
        Command::Note { path, note } => cmd_handlers::handle_note(roadmap, path, note),
        Command::AddSubsection { parent, heading } => {
            cmd_handlers::handle_add_subsection(roadmap, parent, heading)
        }
        Command::Move { path, position } => {
            cmd_handlers::handle_move(roadmap, path, position.clone())
        }
        Command::Chain { parent, items } => {
            cmd_handlers::handle_chain(roadmap, parent, items.clone())
        }
        _ => Err(anyhow::anyhow!("Command not supported")),
    };

    match res {
        Ok(_) => ApplyResult::Success(format!("Applied {cmd}")),
        Err(e) => ApplyResult::Error(format!("Failed {cmd}: {e}")),
    }
}