use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

fn root() -> Option<PathBuf> {
    cfg!(windows)
        .then(|| std::env::var_os("NUB_SANDBOX_TOOL_MSYS_ROOT").map(PathBuf::from))
        .flatten()
}

pub fn grant(fs: &mut Map<String, Value>) {
    if let Some(root) = root() {
        fs.insert(
            root.to_string_lossy().into_owned(),
            Value::String("r".into()),
        );
    }
}

pub fn command(program: &Path, args: Vec<String>) -> (PathBuf, Vec<String>) {
    let Some(root) = root() else {
        return (program.to_owned(), args);
    };
    let shell = root.join("usr/bin/bash.exe");
    assert!(
        shell.is_file(),
        "MSYS shell is provisioned: {}",
        shell.display()
    );
    let mut wrapped = vec![
        "--noprofile".into(),
        "--norc".into(),
        "-c".into(),
        // Keep Bash alive to wait for the tool; exec would omit that ownership path.
        "\"$@\"".into(),
        "sandbox-tools".into(),
        program.to_string_lossy().replace('\\', "/"),
    ];
    wrapped.extend(args);
    (shell, wrapped)
}
