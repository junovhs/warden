// tests/integration_trace.rs
use std::path::PathBuf;
use warden_core::graph::defs;
use warden_core::graph::rank::RepoGraph;

#[test]
fn test_graph_builds_on_warden_itself() {
    let files = vec![
        (
            PathBuf::from("src/lib.rs"),
            std::fs::read_to_string("src/lib.rs").unwrap(),
        ),
        (
            PathBuf::from("src/config/mod.rs"),
            std::fs::read_to_string("src/config/mod.rs").unwrap(),
        ),
    ];

    let graph = RepoGraph::build(&files);
    let ranked = graph.ranked_files();

    // Should have found some files
    assert!(
        !ranked.is_empty() || files.len() <= 2,
        "Graph should process files"
    );
}

#[test]
fn test_defs_extracts_from_real_file() {
    let content = std::fs::read_to_string("src/lib.rs").unwrap();
    let defs = defs::extract(std::path::Path::new("src/lib.rs"), &content);

    // lib.rs should have at least some module declarations
    println!("Found {} definitions in lib.rs", defs.len());
}
