//! Native tool-directory operations, with unconfined and narrow-policy controls.
use nub_sandbox::{CommandSpec, CompileCtx, Homes, Sandbox, ScopeCapabilities, compile};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Command, Output};

fn fixture() -> tempfile::TempDir {
    // Keep denied siblings outside OS temporary directories, which may have
    // independent runtime access on some backends.
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .expect("runner home");
    let root = tempfile::Builder::new()
        .prefix("sandbox-tool-fixture-")
        .tempdir_in(home)
        .expect("fixture root");
    for path in ["home", "project", "cache"] {
        std::fs::create_dir_all(root.path().join(path)).expect("fixture directory");
    }
    root
}

fn env_for(root: &Path, extra: &[(&str, &str)]) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    for key in [
        "PATH",
        "SystemRoot",
        "SYSTEMROOT",
        "WINDIR",
        "COMSPEC",
        "PATHEXT",
    ] {
        if let Ok(value) = std::env::var(key) {
            env.insert(key.to_string(), value);
        }
    }
    env.insert("HOME".into(), root.join("home").to_string_lossy().into());
    env.insert(
        "USERPROFILE".into(),
        root.join("home").to_string_lossy().into(),
    );
    env.insert(
        "XDG_CONFIG_HOME".into(),
        root.join("home/.config").to_string_lossy().into(),
    );
    for (key, value) in extra {
        env.insert((*key).to_string(), (*value).to_string());
    }
    env
}

fn policy(root: &Path, fs: Value, extra_env: &[(&str, &str)]) -> nub_sandbox::SandboxPolicy {
    let mut fs = match fs {
        Value::Object(entries) => entries,
        Value::Array(entries) => entries
            .into_iter()
            .map(|entry| {
                (
                    entry.as_str().unwrap().to_string(),
                    Value::String("rw".into()),
                )
            })
            .collect(),
        _ => panic!("fixture filesystem policy must be an object or array"),
    };
    fs.insert("./".into(), Value::String("rw".into()));
    fs.insert("$tmp".into(), Value::String("rw".into()));
    let homes = Homes {
        home: root.join("home"),
        cache: root.join("cache"),
        tmp: root.join("tmp"),
        project: root.join("project"),
    };
    let env = env_for(root, extra_env);
    let ctx = CompileCtx::new(
        homes,
        root.join("project"),
        ScopeCapabilities::approved(),
        env.clone(),
    );
    let mut policy = compile(&json!({"fs": fs, "env": false, "net": false}), &ctx)
        .expect("tool policy compiles");
    policy.env.constructed = env;
    policy
}

fn exact_grants(paths: &[(&Path, &str)]) -> Value {
    let mut entries = Map::new();
    for (path, access) in paths {
        entries.insert(
            path.to_string_lossy().into_owned(),
            Value::String((*access).into()),
        );
    }
    Value::Object(entries)
}

fn confined(
    program: &str,
    args: &[&str],
    root: &Path,
    policy: &nub_sandbox::SandboxPolicy,
) -> Output {
    let sandbox = Sandbox::new(policy).expect("tool sandbox acquires");
    let prepared = sandbox
        .prepare(
            CommandSpec::new(program)
                .args(args)
                .cwd(root.join("project")),
        )
        .expect("tool command prepares without degradation");
    assert!(
        prepared.degradation.lost.is_empty(),
        "native tool fixture degraded: {:?}",
        prepared.degradation
    );
    prepared.output().expect("tool command launches")
}

fn unconfined(program: &str, args: &[&str], root: &Path, extra_env: &[(&str, &str)]) -> Output {
    let mut command = Command::new(program);
    command.args(args).current_dir(root.join("project"));
    command.env_clear();
    command.envs(env_for(root, extra_env));
    command.output().expect("unconfined tool launches")
}

fn require_git() -> &'static str {
    assert!(
        Command::new("git")
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success()),
        "standard GitHub runners must provide git"
    );
    "git"
}

fn optional_pinned_tool(variable: &str) -> Option<String> {
    let Some(tool) = std::env::var_os(variable) else {
        eprintln!(
            "SKIP {variable}: native tool coverage requires a parent-provisioned pinned executable"
        );
        return None;
    };
    let tool = tool.to_string_lossy().into_owned();
    if Command::new(&tool)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
    {
        Some(tool)
    } else {
        eprintln!("SKIP {variable}: `{tool}` is not an executable pinned tool");
        None
    }
}

#[test]
fn git_global_config_updates_an_existing_xdg_config_and_its_lock() {
    let git = require_git();
    let unconfined_root = fixture();
    let args = [
        "config",
        "--global",
        "user.email",
        "fixture@example.invalid",
    ];
    let prepare_config = |root: &Path| {
        let config = root.join("home/.config/git/config");
        std::fs::create_dir_all(config.parent().unwrap()).unwrap();
        std::fs::write(&config, "[user]\n\tname = Fixture\n").unwrap();
        config
    };
    let control_config = prepare_config(unconfined_root.path());
    let control = unconfined(git, &args, unconfined_root.path(), &[]);
    assert!(
        control.status.success(),
        "unconfined git control failed: {}",
        String::from_utf8_lossy(&control.stderr)
    );
    assert!(
        std::fs::read_to_string(control_config)
            .unwrap()
            .contains("fixture@example.invalid")
    );

    // The whole Git config directory is the narrow, portable positive grant
    // that permits Git's create-lock/rename protocol without writable home.
    let exact_root = fixture();
    let git_config = prepare_config(exact_root.path());
    let exact = confined(
        git,
        &args,
        exact_root.path(),
        &policy(
            exact_root.path(),
            exact_grants(&[(git_config.parent().unwrap(), "rw")]),
            &[],
        ),
    );
    assert!(
        exact.status.success(),
        "Git config directory grant failed: {}",
        String::from_utf8_lossy(&exact.stderr)
    );
    assert!(
        std::fs::read_to_string(&git_config)
            .unwrap()
            .contains("fixture@example.invalid")
    );
    assert!(!git_config.with_extension("lock").exists());

    let tool_root = fixture();
    let tool_config = prepare_config(tool_root.path());
    let tool_dirs = confined(
        git,
        &args,
        tool_root.path(),
        &policy(tool_root.path(), json!(["$tooldirs"]), &[]),
    );
    assert!(
        tool_dirs.status.success(),
        "$tooldirs Git config write failed: {}",
        String::from_utf8_lossy(&tool_dirs.stderr)
    );
    assert!(
        std::fs::read_to_string(&tool_config)
            .unwrap()
            .contains("fixture@example.invalid")
    );
    assert!(!tool_config.with_extension("lock").exists());
}

#[test]
fn npm_cold_cache_creation_requires_a_materialized_parent_grant() {
    let Some(npm) = optional_pinned_tool("NUB_SANDBOX_NPM") else {
        return;
    };
    let args = ["cache", "verify"];

    let unconfined_root = fixture();
    let unconfined_cache = unconfined_root.path().join("cache/npm");
    let unconfined_cache_string = unconfined_cache.to_string_lossy().into_owned();
    let control = unconfined(
        &npm,
        &args,
        unconfined_root.path(),
        &[("NPM_CONFIG_CACHE", &unconfined_cache_string)],
    );
    assert!(
        control.status.success(),
        "unconfined npm control failed: {}",
        String::from_utf8_lossy(&control.stderr)
    );
    assert!(
        unconfined_cache.exists(),
        "npm did not create its cold cache root"
    );

    let exact_root = fixture();
    let exact_cache = exact_root.path().join("cache/npm");
    std::fs::create_dir_all(&exact_cache).expect("materialized exact cache root");
    let exact_cache_string = exact_cache.to_string_lossy().into_owned();
    let exact = confined(
        &npm,
        &args,
        exact_root.path(),
        &policy(
            exact_root.path(),
            exact_grants(&[(&exact_cache, "rw")]),
            &[("NPM_CONFIG_CACHE", &exact_cache_string)],
        ),
    );
    assert!(
        exact.status.success(),
        "materialized exact npm cache grant failed: {}",
        String::from_utf8_lossy(&exact.stderr)
    );

    let tool_root = fixture();
    let tool_cache = tool_root.path().join("cache/npm");
    let tool_cache_string = tool_cache.to_string_lossy().into_owned();
    let tool_dirs = confined(
        &npm,
        &args,
        tool_root.path(),
        &policy(
            tool_root.path(),
            json!(["$tooldirs"]),
            &[("NPM_CONFIG_CACHE", &tool_cache_string)],
        ),
    );
    // Seatbelt authorizes future names; inode/ACL-backed grants require an
    // existing root. This is a backend-limit control, not a functionality pass.
    if cfg!(target_os = "macos") {
        assert!(
            tool_dirs.status.success(),
            "{}",
            String::from_utf8_lossy(&tool_dirs.stderr)
        );
        assert!(tool_cache.exists());
        return;
    }
    assert!(
        !tool_dirs.status.success(),
        "cold speculative cache unexpectedly worked"
    );
    let stderr = String::from_utf8_lossy(&tool_dirs.stderr);
    assert!(
        stderr.contains(&tool_cache_string),
        "npm cold-cache denial omitted the requested path {tool_cache_string}: {stderr}"
    );
}

#[test]
fn uv_honors_a_materialized_custom_cache_under_tooldirs() {
    let Some(uv) = optional_pinned_tool("NUB_SANDBOX_UV") else {
        return;
    };
    let args = ["cache", "clean"];
    let root = fixture();
    let cache = root.path().join("cache/uv-custom");
    std::fs::create_dir_all(&cache).expect("materialized uv cache root");
    let cache_string = cache.to_string_lossy().into_owned();

    let control = unconfined(&uv, &args, root.path(), &[("UV_CACHE_DIR", &cache_string)]);
    assert!(
        control.status.success(),
        "unconfined uv control failed: {}",
        String::from_utf8_lossy(&control.stderr)
    );
    std::fs::create_dir_all(&cache).expect("restore cache root for confined controls");

    let exact = confined(
        &uv,
        &args,
        root.path(),
        &policy(
            root.path(),
            exact_grants(&[(&cache, "rw")]),
            &[("UV_CACHE_DIR", &cache_string)],
        ),
    );
    assert!(
        exact.status.success(),
        "exact custom uv cache grant failed: {}",
        String::from_utf8_lossy(&exact.stderr)
    );
    std::fs::create_dir_all(&cache).expect("restore cache root for $tooldirs control");

    let tool_dirs = confined(
        &uv,
        &args,
        root.path(),
        &policy(
            root.path(),
            json!(["$tooldirs"]),
            &[("UV_CACHE_DIR", &cache_string)],
        ),
    );
    assert!(
        tool_dirs.status.success(),
        "$tooldirs custom uv cache grant failed: {}",
        String::from_utf8_lossy(&tool_dirs.stderr)
    );
}
