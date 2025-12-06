pub mod checker;
pub mod display;
pub mod scanner;
pub mod types;

// Fix for legacy roadmap audit calls that assumed flat tasks
// Since legacy Roadmap struct has nested tasks in sections,
// we provide a helper here if needed, or consumers should iterate sections.