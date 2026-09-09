//! Build-time ESM bytecode for native extracted artifacts. Node still loads the
//! original source and owns cache validation; this only seeds its existing cache.

use std::io::{Seek, SeekFrom};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use nub_core::compile::{AppFile, COMPILE_BOOTSTRAP_NAME};
use serde::Deserialize;

const PACK_NAME: &str = "__nub_code_cache.bin";
const GENERATOR: &str = include_str!("code_cache_generate.cjs");
const INSTALLER: &str = include_str!("code_cache_install.cjs");
const MIN_SOURCE_BYTES: usize = 256 * 1024;

#[derive(Deserialize)]
struct Index {
    version: String,
    arch: String,
    tag: u32,
}

pub(super) fn attach(files: &mut Vec<AppFile<Vec<u8>>>, node: &Path) -> Result<bool> {
    // An included file can already own the optional pack's name. It must never
    // be overwritten or turn a previously valid build into a collision error.
    if files
        .iter()
        .any(|file| file.name.eq_ignore_ascii_case(PACK_NAME))
    {
        return Ok(false);
    }
    let Some(bootstrap) = files
        .iter()
        .position(|file| file.name == COMPILE_BOOTSTRAP_NAME)
    else {
        return Ok(false);
    };
    let sources: Vec<_> = files
        .iter()
        .filter(|file| file.name.ends_with(".mjs"))
        .filter_map(|file| {
            std::str::from_utf8(&file.bytes)
                .ok()
                .map(|source| (&file.name, source))
        })
        .collect();
    if sources
        .iter()
        .map(|(_, source)| source.len())
        .sum::<usize>()
        < MIN_SOURCE_BYTES
    {
        return Ok(false);
    }

    // File-backed stdin avoids blocking a producer against the child's output
    // pipe for a large bundle. No application module is linked or evaluated.
    let mut input = tempfile::tempfile().context("staging compile-cache sources")?;
    serde_json::to_writer(&mut input, &sources)?;
    input.seek(SeekFrom::Start(0))?;
    let output = Command::new(node)
        .args([
            "--experimental-vm-modules",
            "--no-warnings",
            "-e",
            GENERATOR,
        ])
        .env_remove("NODE_OPTIONS")
        .env_remove("NODE_COMPILE_CACHE")
        .env_remove("NODE_DISABLE_COMPILE_CACHE")
        .env_remove("NODE_REPL_EXTERNAL_MODULE")
        .stdin(Stdio::from(input))
        .output()
        .context("generating ESM compile caches with the target Node")?;
    if !output.status.success() {
        bail!(
            "target Node could not generate ESM caches: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let pack = output.stdout;
    let length: [u8; 4] = pack
        .get(..4)
        .context("missing compile-cache index")?
        .try_into()?;
    let index_end = 4usize
        .checked_add(u32::from_le_bytes(length) as usize)
        .context("compile-cache index is too large")?;
    let index: Index = serde_json::from_slice(
        pack.get(4..index_end)
            .context("truncated compile-cache index")?,
    )?;
    let id = crate::cli::sha256_hex(&pack);
    let args = serde_json::to_string(&(PACK_NAME, id, index.version, index.arch, index.tag))?;
    let suffix = format!("\n;{INSTALLER}(...{args});\n");
    files[bootstrap].bytes.extend_from_slice(suffix.as_bytes());
    files.push(AppFile::plain(PACK_NAME.into(), pack));
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_node_accepts_generated_caches_and_preserves_fallbacks() {
        let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/compile/code_cache.test.cjs");
        let output = Command::new("node")
            .arg("--test")
            .arg(script)
            .env_remove("NODE_OPTIONS")
            .output()
            .expect("Node is required by the compile test suite");
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn small_programs_do_not_start_a_cache_generator() {
        let mut files = vec![
            AppFile::plain(COMPILE_BOOTSTRAP_NAME.into(), Vec::new()),
            AppFile::plain("main.mjs".into(), b"console.log(1)".to_vec()),
        ];
        assert!(!attach(&mut files, Path::new("nonexistent-node")).unwrap());
        assert_eq!(files.len(), 2);
        assert!(files[0].bytes.is_empty());
    }

    #[test]
    fn an_included_pack_name_is_not_overwritten() {
        let mut files = vec![AppFile::plain(
            "__NUB_CODE_CACHE.BIN".into(),
            b"user asset".to_vec(),
        )];
        assert!(!attach(&mut files, Path::new("nonexistent-node")).unwrap());
        assert_eq!(files[0].bytes, b"user asset");
    }
}
