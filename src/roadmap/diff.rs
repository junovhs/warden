// slopchop:ignore
use crate::roadmap::types::{Command, Roadmap, Task, TaskStatus};
use crate::roadmap::str_utils::slugify;

pub fn generate_update_commands(old: &Roadmap, new: &Roadmap) -> Vec<Command> {
    let mut cmds = Vec::new();

    let old_tasks = get_all_tasks(old);
    let new_tasks = get_all_tasks(new);

    for new_task in &new_tasks {
        if let Some(old_task) = old_tasks.iter().find(|t| t.id == new_task.id) {
            if old_task.status != new_task.status {
                match new_task.status {
                    TaskStatus::Complete => cmds.push(Command::Check {
                        path: new_task.id.clone(),
                    }),
                    TaskStatus::Pending => cmds.push(Command::Uncheck {
                        path: new_task.id.clone(),
                    }),
                }
            }
            if old_task.text != new_task.text {
                 cmds.push(Command::Update { 
                     path: new_task.id.clone(), 
                     text: new_task.text.clone() 
                 });
            }
        } else {
            cmds.push(Command::Add {
                parent: "unknown".to_string(), 
                text: new_task.text.clone(),
                after: None, 
            });
        }
    }

    for old_task in &old_tasks {
        if !new_tasks.iter().any(|t| t.id == old_task.id) {
            cmds.push(Command::Delete {
                path: old_task.id.clone(),
            });
        }
    }

    cmds
}

fn get_all_tasks(roadmap: &Roadmap) -> Vec<&Task> {
    roadmap.sections.iter()
        .flat_map(|s| &s.tasks)
        .collect()
}

pub fn diff_text(old_text: &str, new_text: &str) -> String {
    if old_text == new_text {
        return "No changes".to_string();
    }
    format!("OLD:\n{old_text}\n\nNEW:\n{new_text}")
}

pub fn infer_command(old: Option<&Task>, new: &Task) -> Command {
    match old {
        Some(o) => {
            if o.status != new.status {
                 if new.status == TaskStatus::Complete {
                     Command::Check { path: new.id.clone() }
                 } else {
                     Command::Uncheck { path: new.id.clone() }
                 }
            } else {
                Command::Update { path: new.id.clone(), text: new.text.clone() }
            }
        },
        None => Command::Add {
            parent: "unknown".to_string(),
            text: new.text.clone(),
            after: None,
        }
    }
}

pub fn generate_slug_for_legacy(title: &str) -> String {
    slugify(title)
}