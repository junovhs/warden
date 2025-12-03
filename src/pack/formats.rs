// src/pack/formats.rs
use super::PackOptions;
use crate::skeleton;
use anyhow::Result;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

/// Packs files into the Warden format.
///
/// # Errors
/// Returns an error if file reading fails.
pub fn pack_warden(files: &[PathBuf], out: &mut String, opts: &PackOptions) -> Result<()> {
    for path in files {
        let p_str = path.to_string_lossy().replace('\\', "/");
        writeln!(out, "#__WARDEN_FILE__# {p_str}")?;

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
        writeln!(out, "\n#__WARDEN_END__#\n")?;
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
    if opts.skeleton {
        return true;
    }

    if let Some(target) = &opts.target {
        return !path.ends_with(target);
    }

    false
}
