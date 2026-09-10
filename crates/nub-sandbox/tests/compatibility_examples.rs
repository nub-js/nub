use nub_sandbox::{CompileCtx, Homes, ScopeCapabilities, compile};

#[test]
fn documented_compatibility_policies_compile_with_the_supplied_environment() {
    let root = tempfile::tempdir().unwrap();
    let environment = [
        ("PATH", "/usr/bin"),
        ("GIT_CONFIG_GLOBAL", "/work/git-config/config"),
        ("NUGET_PACKAGES", "/work/nuget/packages"),
        ("NUGET_HTTP_CACHE_PATH", "/work/nuget/http-cache"),
        ("NUGET_SCRATCH", "/work/nuget/scratch"),
        ("NUGET_PLUGINS_CACHE_PATH", "/work/nuget/plugins-cache"),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_string(), value.to_string()))
    .collect();
    let ctx = CompileCtx::new(
        Homes {
            home: root.path().join("home"),
            cache: root.path().join("cache"),
            tmp: root.path().join("tmp"),
            project: root.path().into(),
        },
        root.path().into(),
        ScopeCapabilities::approved(),
        environment,
    );
    let document = include_str!("../COMPATIBILITY.md").replace("\r\n", "\n");
    let mut count = 0;
    for block in document.split("```json\n").skip(1) {
        let example = block.split("```").next().unwrap();
        let value: serde_json::Value = serde_json::from_str(example).unwrap();
        compile(&value, &ctx).unwrap_or_else(|error| panic!("{example}\n{error}"));
        count += 1;
    }
    assert!(
        count >= 3,
        "the documented policy examples must be exercised"
    );
}
