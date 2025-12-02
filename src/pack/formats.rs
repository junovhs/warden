// src/pack/formats.rs
use super::PackOptions;
use crate::skeleton;
use anyhow::Result;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

/// Packs files into the Nabla format.
///
/// # Errors
/// Returns an error if file reading fails.
pub fn pack_nabla(files: &[PathBuf], out: &mut String, opts: &PackOptions) -> Result<()> {
    for path in files {
        let p_str = path.to_string_lossy().replace('\\', "/");
        writeln!(out, "∇∇∇ {p_str} ∇∇∇")?;

        match fs::read_to_string(path) {
            Ok(content) => {
                if should_skeletonize(path, opts) {
                    out.push_str(&skeleton::clean(path, &content));
                } else {
                    out.push_str(&content);
                }
            }
            Err(e) => writeln!(out, "// <ERROR READING FILE: {e}>")?,
        }
        writeln!(out, "\n∆∆∆\n")?;
    }
    Ok(())
}

/// Packs files into an XML format.
///
/// # Errors
/// Returns an error if file reading fails.
pub fn pack_xml(files: &[PathBuf], out: &mut String, opts: &PackOptions) -> Result<()> {
    writeln!(out, "<documents>")?;
    for path in files {
        let p_str = path.to_string_lossy().replace('\\', "/");
        writeln!(out, "  <document path=\"{p_str}\"><![CDATA[")?;

        match fs::read_to_string(path) {
            Ok(content) => {
                if should_skeletonize(path, opts) {
                    out.push_str(
                        &skeleton::clean(path, &content).replace("]]>", "]]]]><![CDATA[>"),
                    );
                } else {
                    out.push_str(&content.replace("]]>", "]]]]><![CDATA[>"));
                }
            }
            Err(e) => writeln!(out, "<!-- ERROR: {e} -->")?,
        }
        writeln!(out, "]]></document>")?;
    }
    writeln!(out, "</documents>")?;
    Ok(())
}

fn should_skeletonize(path: &Path, opts: &PackOptions) -> bool {
    // If global skeleton flag is on, everything is skeletonized
    if opts.skeleton {
        return true;
    }

    // If a target is specified, everything EXCEPT the target is skeletonized
    if let Some(target) = &opts.target {
        // We do a loose match: if the path ends with the target string.
        // This allows "warden pack src/main.rs" to match "./src/main.rs"
        return !path.ends_with(target);
    }

    false
}
