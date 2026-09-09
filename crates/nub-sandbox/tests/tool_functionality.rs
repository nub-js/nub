//! Native tool-directory operations, with unconfined and narrow-policy controls.
#[path = "common/tool_output.rs"]
mod tool_output;
use nub_sandbox::{CommandSpec, CompileCtx, Homes, Sandbox, ScopeCapabilities, compile};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
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
    let mut policy = compile(&json!({"fs": fs, "net": false}), &ctx).expect("tool policy compiles");
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
    eprintln!("CONFINED {program} {args:?}");
    let sandbox = Sandbox::new(policy).expect("tool sandbox acquires");
    let prepared = sandbox
        .prepare(
            CommandSpec::new(program)
                .args(args)
                .redact_stdout(true)
                .redact_stderr(true)
                .cwd(root.join("project")),
        )
        .expect("tool command prepares without degradation");
    assert!(
        prepared.degradation.lost.is_empty(),
        "native tool fixture degraded: {:?}",
        prepared.degradation
    );
    tool_output::output(prepared)
}

fn unconfined(program: &str, args: &[&str], root: &Path, extra_env: &[(&str, &str)]) -> Output {
    eprintln!("UNCONFINED {program} {args:?}");
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

#[derive(Deserialize)]
struct Tool {
    name: String,
    spec: String,
    kind: String,
    program: String,
    prefix: Vec<String>,
    #[serde(rename = "toolRoot")]
    tool_root: PathBuf,
    #[serde(rename = "runtimeRoot")]
    runtime_root: PathBuf,
    architecture: String,
    version: String,
}

fn tools() -> Vec<Tool> {
    let matrix = std::env::var("NUB_SANDBOX_TOOL_MATRIX_FILE").expect(
        "tool matrix missing: run `node scripts/sandbox-tool-fixtures.mjs` before native tests",
    );
    let text = std::fs::read_to_string(&matrix)
        .unwrap_or_else(|error| panic!("tool matrix `{matrix}` unreadable: {error}"));
    let tools: Vec<Tool> = serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("tool matrix `{matrix}` is invalid: {error}"));
    assert!(
        tools.iter().any(|tool| tool.name == "npm")
            && tools.iter().any(|tool| tool.name == "pnpm9")
            && tools.iter().any(|tool| tool.name == "pnpm10")
            && tools.iter().any(|tool| tool.name == "pnpm11")
            && tools.iter().any(|tool| tool.name == "yarn1")
            && tools.iter().any(|tool| tool.name == "yarn2")
            && tools.iter().any(|tool| tool.name == "yarn3")
            && tools.iter().any(|tool| tool.name == "yarn4")
            && tools.iter().any(|tool| tool.name == "bun132")
            && tools.iter().any(|tool| tool.name == "bun140"),
        "tool matrix omitted a required compatibility target"
    );
    tools
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

fn fixture_package(root: &Path) -> PathBuf {
    let package = root.join("project/package");
    std::fs::create_dir_all(&package).expect("local package directory");
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"sandbox-tool-fixture-bin","version":"1.0.0","bin":{"fixture-bin":"cli.js"}}"#,
    )
    .expect("local package manifest");
    std::fs::write(
        package.join("cli.js"),
        "#!/usr/bin/env node\nprocess.stdout.write('fixture-bin-ok\\n');\n",
    )
    .expect("local package bin");
    package
}

fn project_manifest(root: &Path) {
    std::fs::write(
        root.join("project/package.json"),
        r#"{"name":"sandbox-tool-project","private":true,"dependencies":{"sandbox-tool-fixture-bin":"file:./package"}}"#,
    )
    .expect("project manifest");
}

fn tool_env(tool: &Tool, root: &Path) -> (PathBuf, PathBuf, Vec<(String, String)>) {
    let cache = root.join("cache").join(&tool.name);
    let global = root.join("home/.tool-global").join(&tool.name);
    let cache_text = cache.to_string_lossy().into_owned();
    let global_text = global.to_string_lossy().into_owned();
    let env = match tool.kind.as_str() {
        "npm" => vec![
            ("NPM_CONFIG_CACHE".into(), cache_text),
            ("NPM_CONFIG_PREFIX".into(), global_text),
        ],
        "pnpm" => vec![
            ("PNPM_CONFIG_CACHE_DIR".into(), cache_text.clone()),
            (
                "PNPM_CONFIG_STORE_DIR".into(),
                cache.join("store").to_string_lossy().into(),
            ),
            ("PNPM_HOME".into(), global_text),
            (
                "PATH".into(),
                format!(
                    "{}{}{}{}{}",
                    global.to_string_lossy(),
                    if cfg!(windows) { ";" } else { ":" },
                    global.join("bin").to_string_lossy(),
                    if cfg!(windows) { ";" } else { ":" },
                    std::env::var("PATH").expect("runner PATH")
                ),
            ),
        ],
        "yarn1" | "yarn" => vec![
            ("YARN_CACHE_FOLDER".into(), cache_text),
            ("YARN_GLOBAL_FOLDER".into(), global_text),
        ],
        "bun" => vec![
            ("BUN_INSTALL_CACHE_DIR".into(), cache_text),
            ("BUN_INSTALL".into(), global_text),
        ],
        other => panic!("unknown tool kind {other}"),
    };
    (cache, global, env)
}

fn args(tool: &Tool, tail: &[&str]) -> Vec<String> {
    tool.prefix
        .iter()
        .cloned()
        .chain(tail.iter().map(|value| (*value).to_string()))
        .collect()
}

fn args_owned(tool: &Tool, tail: &[String]) -> Vec<String> {
    tool.prefix
        .iter()
        .cloned()
        .chain(tail.iter().cloned())
        .collect()
}

fn invoke_unconfined(tool: &Tool, tail: &[&str], root: &Path, env: &[(String, String)]) -> Output {
    let args = args(tool, tail);
    let env: Vec<_> = env
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    unconfined(
        &tool.program,
        &args.iter().map(String::as_str).collect::<Vec<_>>(),
        root,
        &env,
    )
}

fn invoke_confined(
    tool: &Tool,
    tail: &[&str],
    root: &Path,
    env: &[(String, String)],
    policy: &nub_sandbox::SandboxPolicy,
) -> Output {
    let args = args(tool, tail);
    let _ = env; // Environment is captured while compiling the policy.
    confined(
        &tool.program,
        &args.iter().map(String::as_str).collect::<Vec<_>>(),
        root,
        policy,
    )
}

fn invoke_owned(
    tool: &Tool,
    tail: &[String],
    root: &Path,
    env: &[(String, String)],
    policy: Option<&nub_sandbox::SandboxPolicy>,
) -> Output {
    let args = args_owned(tool, tail);
    let refs: Vec<_> = args.iter().map(String::as_str).collect();
    match policy {
        Some(policy) => confined(&tool.program, &refs, root, policy),
        None => {
            let env: Vec<_> = env
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_str()))
                .collect();
            unconfined(&tool.program, &refs, root, &env)
        }
    }
}

fn grant_policy(
    tool: &Tool,
    root: &Path,
    cache: &Path,
    global: &Path,
    env: &[(String, String)],
    tooldirs: bool,
) -> nub_sandbox::SandboxPolicy {
    let env: Vec<_> = env
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    let fs = if tooldirs {
        let mut entries = Map::new();
        entries.insert("$tooldirs".into(), Value::String("rw".into()));
        entries.insert(
            tool.tool_root.to_string_lossy().into_owned(),
            Value::String("r".into()),
        );
        entries.insert(
            tool.runtime_root.to_string_lossy().into_owned(),
            Value::String("r".into()),
        );
        Value::Object(entries)
    } else {
        let yarn_home = root.join("home/.yarn");
        let mut grants = vec![
            (cache, "rw"),
            (global, "rw"),
            (&tool.tool_root, "r"),
            (&tool.runtime_root, "r"),
        ];
        // Yarn Classic falls back to this user-global state root even with
        // `YARN_GLOBAL_FOLDER` relocated. Keep this narrow rather than granting home.
        if tool.kind == "yarn1" {
            grants.push((&yarn_home, "rw"));
        }
        exact_grants(&grants)
    };
    policy(root, fs, &env)
}

fn assert_success(tool: &Tool, phase: &str, output: &Output) {
    assert!(
        output.status.success(),
        "{} {} failed:\nstdout:\n{}\nstderr:\n{}",
        tool.name,
        phase,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_normal_operations(
    tool: &Tool,
    root: &Path,
    env: &[(String, String)],
    policy: Option<&nub_sandbox::SandboxPolicy>,
) {
    fixture_package(root);
    project_manifest(root);
    let run = |tail: &[&str]| match policy {
        Some(policy) => invoke_confined(tool, tail, root, env, policy),
        None => invoke_unconfined(tool, tail, root, env),
    };
    let install: &[&str] = match tool.kind.as_str() {
        "npm" => &["install", "--ignore-scripts", "--no-audit", "--no-fund"],
        "pnpm" => &["install", "--ignore-scripts"],
        "yarn1" => &["install", "--ignore-scripts"],
        "yarn" if tool.name == "yarn2" => &["install", "--skip-builds"],
        "yarn" => &["install", "--mode=skip-build"],
        "bun" => &["install", "--ignore-scripts"],
        _ => unreachable!(),
    };
    assert_success(tool, "local install", &run(install));
    assert_success(tool, "warm reinstall", &run(install));
    let exec: &[&str] = match tool.kind.as_str() {
        "npm" => &["exec", "--", "fixture-bin"],
        "pnpm" => &["exec", "fixture-bin"],
        "yarn1" => &["run", "fixture-bin"],
        "yarn" if tool.name == "yarn2" => &["run", "fixture-bin"],
        "yarn" => &["exec", "fixture-bin"],
        "bun" => &["x", "--no-install", "fixture-bin"],
        _ => unreachable!(),
    };
    let output = run(exec);
    assert_success(tool, "installed-bin execution", &output);
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("fixture-bin-ok"),
        "{} did not execute the installed fixture bin: {}",
        tool.name,
        String::from_utf8_lossy(&output.stdout)
    );
}

fn run_global_install_and_cache_prune(
    tool: &Tool,
    root: &Path,
    env: &[(String, String)],
    policy: Option<&nub_sandbox::SandboxPolicy>,
) {
    if matches!(tool.kind.as_str(), "yarn") {
        eprintln!(
            "TOOL {} has no supported user-global command in this Berry fixture",
            tool.name
        );
        return;
    }
    let package = root.join("project/package").to_string_lossy().into_owned();
    let global = match tool.kind.as_str() {
        "npm" => vec!["install".into(), "--global".into(), package],
        "pnpm" => vec!["add".into(), "--global".into(), package],
        "yarn1" => vec!["global".into(), "add".into(), package],
        "bun" => vec!["install".into(), "--global".into(), package],
        _ => unreachable!(),
    };
    assert_success(
        tool,
        "configured global install",
        &invoke_owned(tool, &global, root, env, policy),
    );
    let prune: Vec<String> = match tool.kind.as_str() {
        "npm" => vec!["cache".into(), "clean".into(), "--force".into()],
        "pnpm" => vec!["store".into(), "prune".into()],
        "yarn1" => vec!["cache".into(), "clean".into()],
        "bun" => vec!["pm".into(), "cache".into(), "rm".into()],
        _ => unreachable!(),
    };
    assert_success(
        tool,
        "configured cache prune",
        &invoke_owned(tool, &prune, root, env, policy),
    );
}

#[derive(Clone, Copy)]
enum ToolControl {
    Unconfined,
    Exact,
    Tooldirs,
}

impl ToolControl {
    fn label(self) -> &'static str {
        match self {
            Self::Unconfined => "unconfined",
            Self::Exact => "exact",
            Self::Tooldirs => "$tooldirs",
        }
    }

    fn tooldirs(self) -> Option<bool> {
        match self {
            Self::Unconfined => None,
            Self::Exact => Some(false),
            Self::Tooldirs => Some(true),
        }
    }
}

fn run_tool_control(name: &str, control: ToolControl) {
    let tools = tools();
    let tool = tools
        .iter()
        .find(|tool| tool.name == name)
        .unwrap_or_else(|| panic!("tool matrix omitted {name}"));
    eprintln!(
        "TOOL {} {} ({}, {})",
        tool.name, tool.version, tool.spec, tool.architecture
    );
    let root = fixture();
    let (cache, global, env) = tool_env(tool, root.path());
    // Precondition for native inode/ACL backends: these existing roots are what their grants
    // can name. Cold-root denial is covered separately below.
    std::fs::create_dir_all(&cache).expect("materialized cache precondition");
    std::fs::create_dir_all(&global).expect("materialized global precondition");
    if tool.kind == "yarn1" {
        std::fs::create_dir_all(root.path().join("home/.yarn"))
            .expect("materialized Yarn Classic state precondition");
    }
    let policy = control
        .tooldirs()
        .map(|tooldirs| grant_policy(tool, root.path(), &cache, &global, &env, tooldirs));
    eprintln!("TOOL {} {} control", tool.name, control.label());
    run_normal_operations(tool, root.path(), &env, policy.as_ref());
    run_global_install_and_cache_prune(tool, root.path(), &env, policy.as_ref());
}

macro_rules! tool_controls {
    ($tool:literal, $unconfined:ident, $exact:ident, $tooldirs:ident) => {
        #[test]
        #[ignore = "requires the pinned native tool matrix"]
        fn $unconfined() {
            run_tool_control($tool, ToolControl::Unconfined);
        }

        #[test]
        #[ignore = "requires the pinned native tool matrix"]
        fn $exact() {
            run_tool_control($tool, ToolControl::Exact);
        }

        #[test]
        #[ignore = "requires the pinned native tool matrix"]
        fn $tooldirs() {
            run_tool_control($tool, ToolControl::Tooldirs);
        }
    };
}

tool_controls!(
    "npm",
    npm_unconfined_normal_operations,
    npm_exact_normal_operations,
    npm_tooldirs_normal_operations
);
tool_controls!(
    "pnpm9",
    pnpm9_unconfined_normal_operations,
    pnpm9_exact_normal_operations,
    pnpm9_tooldirs_normal_operations
);
tool_controls!(
    "pnpm10",
    pnpm10_unconfined_normal_operations,
    pnpm10_exact_normal_operations,
    pnpm10_tooldirs_normal_operations
);
tool_controls!(
    "pnpm11",
    pnpm11_unconfined_normal_operations,
    pnpm11_exact_normal_operations,
    pnpm11_tooldirs_normal_operations
);
tool_controls!(
    "yarn1",
    yarn1_unconfined_normal_operations,
    yarn1_exact_normal_operations,
    yarn1_tooldirs_normal_operations
);
tool_controls!(
    "yarn2",
    yarn2_unconfined_normal_operations,
    yarn2_exact_normal_operations,
    yarn2_tooldirs_normal_operations
);
tool_controls!(
    "yarn3",
    yarn3_unconfined_normal_operations,
    yarn3_exact_normal_operations,
    yarn3_tooldirs_normal_operations
);
tool_controls!(
    "yarn4",
    yarn4_unconfined_normal_operations,
    yarn4_exact_normal_operations,
    yarn4_tooldirs_normal_operations
);
tool_controls!(
    "bun132",
    bun132_unconfined_normal_operations,
    bun132_exact_normal_operations,
    bun132_tooldirs_normal_operations
);
tool_controls!(
    "bun140",
    bun140_unconfined_normal_operations,
    bun140_exact_normal_operations,
    bun140_tooldirs_normal_operations
);

#[cfg(windows)]
fn run_node_adapter_control(name: &str, tooldirs: bool) {
    let tools = tools();
    let tool = tools.iter().find(|tool| tool.name == name).unwrap();
    let root = fixture();
    let (cache, global, env) = tool_env(tool, root.path());
    for path in [&cache, &global, &root.path().join("home/.yarn")] {
        std::fs::create_dir_all(path).unwrap();
    }
    fixture_package(root.path());
    project_manifest(root.path());
    let canary = root.path().join("outside-secret");
    std::fs::write(&canary, "DENIED_CANARY").unwrap();
    let mut policy = grant_policy(tool, root.path(), &cache, &global, &env, tooldirs);
    // Deliberate opt-in to the existing Node adapters, not backend auto-detection.
    // The build-jail helper also carries its userland network gate; this fixture's
    // OS policy already denies networking and never installs a registry package.
    let options = format!(
        "{} {}",
        nub_sandbox::windows_build_jail_node_options(None, None),
        nub_sandbox::realpath_shim_node_options(&[
            root.path().join("project"),
            cache,
            global,
            root.path().join("home/.yarn"),
            tool.tool_root.clone(),
            tool.runtime_root.clone(),
        ])
    );
    policy
        .env
        .constructed
        .insert("NODE_OPTIONS".into(), options);
    let sandbox = Sandbox::acquire(&policy).expect("adapted session acquires");
    let run = |argv: Vec<String>| {
        eprintln!(
            "ADAPTED {} {} {argv:?}",
            tool.name,
            if tooldirs { "$tooldirs" } else { "exact" }
        );
        let prepared = sandbox
            .prepare(
                CommandSpec::new(&tool.program)
                    .args(argv)
                    .cwd(root.path().join("project"))
                    .redact_stdout(true)
                    .redact_stderr(true),
            )
            .expect("adapted command prepares");
        assert!(
            prepared.degradation.lost.is_empty(),
            "{:?}",
            prepared.degradation
        );
        tool_output::output(prepared)
    };
    let install = ["install", "--ignore-scripts"];
    assert_success(tool, "adapted local install", &run(args(tool, &install)));
    assert_success(
        tool,
        "adapted retained reinstall",
        &run(args(tool, &install)),
    );
    let exec = if tool.kind == "pnpm" { "exec" } else { "run" };
    let output = run(args(tool, &[exec, "fixture-bin"]));
    assert_success(tool, "adapted installed bin", &output);
    assert!(String::from_utf8_lossy(&output.stdout).contains("fixture-bin-ok"));
    let package = root
        .path()
        .join("project/package")
        .to_string_lossy()
        .into_owned();
    let global = if tool.kind == "pnpm" {
        vec!["add".into(), "--global".into(), package]
    } else {
        vec!["global".into(), "add".into(), package]
    };
    assert_success(
        tool,
        "adapted global install",
        &run(args_owned(tool, &global)),
    );
    let prune = if tool.kind == "pnpm" {
        ["store", "prune"]
    } else {
        ["cache", "clean"]
    };
    assert_success(tool, "adapted cache prune", &run(args(tool, &prune)));
    let check = format!(
        "const fs=require('node:fs');try{{fs.readFileSync({});process.exit(91)}}catch(e){{if(!['EACCES','EPERM'].includes(e.code))throw e}};console.log('CANARY_DENIED')",
        serde_json::to_string(&canary).unwrap()
    );
    let output = run(vec!["-e".into(), check]);
    assert_success(tool, "adapted canary denial", &output);
    assert!(String::from_utf8_lossy(&output.stdout).contains("CANARY_DENIED"));
    sandbox.close();
    nub_sandbox::cleanup().expect("adapted idle resources are reclaimed");
}

#[cfg(windows)]
macro_rules! node_adapter_controls {
    ($tool:literal, $plain:ident, $exact:ident, $tooldirs:ident) => {
        #[test]
        #[ignore = "requires the pinned native tool matrix"]
        fn $plain() {
            run_tool_control($tool, ToolControl::Unconfined);
        }
        #[test]
        #[ignore = "requires the pinned native tool matrix"]
        fn $exact() {
            run_node_adapter_control($tool, false);
        }
        #[test]
        #[ignore = "requires the pinned native tool matrix"]
        fn $tooldirs() {
            run_node_adapter_control($tool, true);
        }
    };
}

#[cfg(windows)]
node_adapter_controls!(
    "pnpm9",
    windows_node_adapter_pnpm9_plain,
    windows_node_adapter_pnpm9_exact,
    windows_node_adapter_pnpm9_tooldirs
);
#[cfg(windows)]
node_adapter_controls!(
    "pnpm10",
    windows_node_adapter_pnpm10_plain,
    windows_node_adapter_pnpm10_exact,
    windows_node_adapter_pnpm10_tooldirs
);
#[cfg(windows)]
node_adapter_controls!(
    "pnpm11",
    windows_node_adapter_pnpm11_plain,
    windows_node_adapter_pnpm11_exact,
    windows_node_adapter_pnpm11_tooldirs
);
#[cfg(windows)]
node_adapter_controls!(
    "yarn1",
    windows_node_adapter_yarn1_plain,
    windows_node_adapter_yarn1_exact,
    windows_node_adapter_yarn1_tooldirs
);

#[test]
#[ignore = "requires the pinned native tool matrix"]
fn npm_cold_cache_root_remains_a_backend_limit_control() {
    let tools = tools();
    let npm = tools
        .iter()
        .find(|tool| tool.name == "npm")
        .expect("matrix includes npm");
    let root = fixture();
    let (cache, global, env) = tool_env(npm, root.path());
    assert!(!cache.exists(), "cold cache fixture must start absent");
    let policy = grant_policy(npm, root.path(), &cache, &global, &env, true);
    let output = invoke_confined(npm, &["cache", "verify"], root.path(), &env, &policy);
    if cfg!(target_os = "macos") {
        assert_success(npm, "cold $tooldirs cache creation", &output);
        assert!(cache.exists());
        return;
    }
    assert!(
        !output.status.success(),
        "cold speculative npm cache unexpectedly worked"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(cache.to_string_lossy().as_ref()),
        "npm cold-cache denial omitted {}: {stderr}",
        cache.display()
    );
}
