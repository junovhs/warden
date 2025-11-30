use std::fs;
use tempfile::tempdir;
use warden_core::graph::{imports, resolver};

#[test]
fn test_rust_graph_resolution() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    let main_rs = src.join("main.rs");
    let utils_rs = src.join("utils.rs");
    let config_dir = src.join("config");
    let config_mod = config_dir.join("mod.rs");

    fs::create_dir_all(&config_dir).unwrap();
    
    // Write files
    fs::write(&main_rs, "mod utils; mod config;").unwrap();
    fs::write(&utils_rs, "// utils").unwrap();
    fs::write(&config_mod, "// config mod").unwrap();

    // 1. Extract from main.rs
    let main_content = fs::read_to_string(&main_rs).unwrap();
    let imported = imports::extract(&main_rs, &main_content);

    assert!(imported.contains(&"utils".to_string()));
    assert!(imported.contains(&"config".to_string()));

    // 2. Resolve 'utils'
    let resolved_utils = resolver::resolve(root, &main_rs, "utils");
    assert_eq!(resolved_utils, Some(utils_rs));

    // 3. Resolve 'config' (mod.rs variant)
    let resolved_config = resolver::resolve(root, &main_rs, "config");
    assert_eq!(resolved_config, Some(config_mod));
}

#[test]
fn test_python_graph_resolution() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    let main_py = root.join("main.py");
    let lib_py = root.join("lib.py");
    
    fs::write(&main_py, "import lib").unwrap();
    fs::write(&lib_py, "# lib").unwrap();

    let main_content = fs::read_to_string(&main_py).unwrap();
    let imported = imports::extract(&main_py, &main_content);

    assert!(imported.contains(&"lib".to_string()));

    let resolved = resolver::resolve(root, &main_py, "lib");
    assert_eq!(resolved, Some(lib_py));
}

#[test]
fn test_ts_graph_resolution() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    let app_ts = root.join("app.ts");
    let comp_tsx = root.join("Component.tsx");
    
    fs::write(&app_ts, "import C from './Component'").unwrap();
    fs::write(&comp_tsx, "// component").unwrap();

    let content = fs::read_to_string(&app_ts).unwrap();
    let imported = imports::extract(&app_ts, &content);

    assert!(imported.contains(&"./Component".to_string()));

    let resolved = resolver::resolve(root, &app_ts, "./Component");
    assert_eq!(resolved, Some(comp_tsx));
}