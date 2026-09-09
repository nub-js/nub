use nub_sandbox::policy::FsAccess;
use nub_sandbox::{CompileCtx, Homes, ScopeCapabilities, compile};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn ctx(env: &[(&str, &str)]) -> CompileCtx {
    CompileCtx::new(
        Homes {
            home: PathBuf::from("/home/sandbox"),
            tmp: PathBuf::from("/tmp/nub-private"),
            cache: PathBuf::from("/home/sandbox/.cache"),
            project: PathBuf::from("/project"),
        },
        PathBuf::from("/project"),
        ScopeCapabilities::approved(),
        env.iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect::<BTreeMap<_, _>>(),
    )
}

#[test]
fn home_alias_and_cache_expand_to_their_compiler_anchors() {
    let policy = compile(
        &json!({"fs": {"$home/.config/tool": "r", "$cache/tool": "rw"}}),
        &ctx(&[]),
    )
    .expect("the four-root grammar compiles");
    let rules: Vec<_> = policy
        .fs
        .rules
        .entries
        .iter()
        .map(|rule| rule.matcher.as_str())
        .collect();
    assert!(
        rules
            .iter()
            .any(|rule| rule.contains("/home/sandbox/.config/tool"))
    );
    assert!(
        rules
            .iter()
            .any(|rule| rule.contains("/home/sandbox/.cache/tool"))
    );
    assert!(policy.fs.rules.entries.iter().any(|rule| {
        rule.matcher.as_str().contains("/home/sandbox/.config/tool")
            && rule.access == FsAccess::Read
    }));
    assert!(policy.fs.rules.entries.iter().any(|rule| {
        rule.matcher.as_str().contains("/home/sandbox/.cache/tool")
            && rule.access == FsAccess::ReadWrite
    }));
}

#[test]
fn tooldirs_adds_nonempty_documented_environment_relocations() {
    let policy = compile(
        &json!({"fs": {"$tooldirs": "r"}}),
        &ctx(&[
            ("npm_config_cache", "/relocated/npm-cache"),
            ("PNPM_HOME", "/relocated/pnpm-home"),
            ("BUN_INSTALL_CACHE_DIR", "/relocated/bun-cache"),
            ("XDG_DATA_HOME", "/relocated/data"),
            ("YARN_CACHE_FOLDER", ""),
        ]),
    )
    .expect("documented relocations compile without tool discovery");
    let rules: Vec<_> = policy
        .fs
        .rules
        .entries
        .iter()
        .map(|rule| rule.matcher.as_str())
        .collect();
    for expected in [
        "/relocated/npm-cache",
        "/relocated/pnpm-home",
        "/relocated/bun-cache",
        "/relocated/data/pnpm",
    ] {
        assert!(
            rules.iter().any(|rule| rule.contains(expected)),
            "missing {expected}: {rules:?}"
        );
        assert!(policy.fs.rules.entries.iter().any(|rule| {
            rule.matcher.as_str().contains(expected) && rule.access == FsAccess::Read
        }));
    }
    assert!(!rules.iter().any(|rule| rule.contains("YARN_CACHE_FOLDER")));
}

#[test]
fn tooldirs_preserves_literal_whitespace_and_anchors_relative_relocations_once() {
    let policy = compile(
        &json!({"fs": ["$tooldirs"]}),
        &ctx(&[("NPM_CONFIG_CACHE", " cache with spaces ")]),
    )
    .expect("a literal relative environment path compiles");
    let rules: Vec<_> = policy
        .fs
        .rules
        .entries
        .iter()
        .map(|rule| rule.matcher.as_str())
        .collect();
    assert!(
        rules
            .iter()
            .any(|rule| rule.contains("/project/ cache with spaces "))
    );
    assert!(
        !rules
            .iter()
            .any(|rule| rule.contains("/project/cache with spaces"))
    );
}

#[test]
fn tooldirs_splits_list_valued_gopath() {
    let gopath = std::env::join_paths(["/relocated/go-one", "/relocated/go-two"])
        .expect("test paths join on this platform")
        .into_string()
        .expect("test paths are utf-8");
    let policy = compile(&json!({"fs": ["$tooldirs"]}), &ctx(&[("GOPATH", &gopath)]))
        .expect("list-valued GOPATH compiles");
    let rules: Vec<_> = policy
        .fs
        .rules
        .entries
        .iter()
        .map(|rule| rule.matcher.as_str())
        .collect();
    for expected in ["/relocated/go-one", "/relocated/go-two"] {
        assert!(rules.iter().any(|rule| rule.contains(expected)));
    }
}

#[test]
fn tooldirs_rejects_environment_roots_instead_of_granting_the_disk() {
    for root in ["/", "C:/"] {
        assert!(
            compile(
                &json!({"fs": ["$tooldirs"]}),
                &ctx(&[("NPM_CONFIG_CACHE", root)]),
            )
            .is_err(),
            "must reject relocation root {root}"
        );
    }
}

#[cfg(windows)]
#[test]
fn tooldirs_uses_redirected_windows_profile_roots() {
    let policy = compile(
        &json!({"fs": {"$tooldirs": "r"}}),
        &ctx(&[("LOCALAPPDATA", "C:/redirected/local-app-data")]),
    )
    .expect("redirected Windows roots compile");
    assert!(policy.fs.rules.entries.iter().any(|rule| {
        rule.matcher
            .as_str()
            .contains("C:/redirected/local-app-data/npm-cache")
    }));
}

#[test]
fn managed_tmp_and_user_denies_fail_loudly() {
    for surface in [
        json!({"fs": ["$tmp/subdir"]}),
        json!({"fs": {"$tmp": "r"}}),
        json!({"fs": ["!/private"]}),
        json!({"fs": {"/private": false}}),
        json!({"fs": ["!$tooldirs"]}),
        json!({"fs": {"$tooldirs": false}}),
        json!({"fs": ["$unknown"]}),
    ] {
        assert!(
            compile(&surface, &ctx(&[])).is_err(),
            "must reject {surface}"
        );
    }
}

#[test]
fn fs_false_remains_the_explicit_axis_off_form() {
    let policy = compile(&json!({"fs": false}), &ctx(&[])).expect("fs false compiles");
    assert_eq!(
        policy.fs.rules.default_effect,
        nub_sandbox::policy::Effect::Deny
    );
    assert!(policy.fs.rules.entries.is_empty());
}

#[test]
fn tooldirs_include_git_config_and_its_atomic_lock() {
    let policy = compile(&json!({"fs": {"$tooldirs": "rw"}}), &ctx(&[]))
        .expect("default tool roots compile");
    let rules: Vec<_> = policy
        .fs
        .rules
        .entries
        .iter()
        .map(|rule| rule.matcher.as_str())
        .collect();
    for expected in [".gitconfig", ".gitconfig.lock"] {
        assert!(rules.iter().any(|rule| rule.contains(expected)));
        assert!(policy.fs.rules.entries.iter().any(|rule| {
            rule.matcher.as_str().contains(expected) && rule.access == FsAccess::ReadWrite
        }));
        assert!(policy.fs.rules.entries.iter().any(|rule| {
            rule.matcher.as_str().ends_with(expected) && !rule.matcher.as_str().contains("/**")
        }));
    }
}

#[test]
fn tooldirs_include_the_current_os_js_package_manager_layouts() {
    let policy = compile(&json!({"fs": ["$tooldirs"]}), &ctx(&[]))
        .expect("current static tool layouts compile");
    let rules: Vec<_> = policy
        .fs
        .rules
        .entries
        .iter()
        .map(|rule| rule.matcher.as_str())
        .collect();
    let expected: &[&str] = if cfg!(target_os = "macos") {
        &[
            "/.npm",
            "/Library/pnpm",
            "/Library/Caches/Yarn",
            "/.bun/install",
        ]
    } else if cfg!(windows) {
        &[
            "/AppData/Local/npm-cache",
            "/AppData/Local/pnpm",
            "/AppData/Local/Yarn",
            "/.bun/install",
        ]
    } else {
        &[
            "/.npm",
            "/.local/share/pnpm",
            "/.cache/yarn",
            "/.bun/install",
        ]
    };
    for expected in expected {
        assert!(rules.iter().any(|rule| rule.contains(expected)));
    }
}

#[test]
fn authored_filesystem_policy_has_no_implicit_secret_or_policy_file_denies() {
    let policy = compile(
        &json!({"fs": ["."]}),
        &ctx(&[]).with_policy_files(vec![PathBuf::from("/project/policy.jsonc")]),
    )
    .expect("positive-only authored filesystem policy compiles");
    assert!(
        policy
            .fs
            .rules
            .entries
            .iter()
            .all(|rule| rule.effect == nub_sandbox::policy::Effect::Allow)
    );
    assert!(!policy.fs.rules.entries.iter().any(|rule| {
        rule.matcher.as_str().contains(".env") || rule.matcher.as_str().contains("policy.jsonc")
    }));
}
