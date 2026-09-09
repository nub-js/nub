# Sandbox engine

The engine compiles filesystem, network and environment permissions for native child processes. Its Rust interface accepts configuration and paths supplied by an embedder; these examples do not define a `nub.jsonc` field or a command-line policy format.

## Filesystem permissions

```json
{
  "fs": {
    "./": "rw",
    "$tooldirs": "rw",
    "$tmp": "rw"
  },
  "net": false
}
```

Directory grants cover descendants. Overlapping positive grants combine: a read-only grant does not remove an existing write grant. Filesystem deny entries such as `!~/.ssh` are rejected; Nub does not translate them into sibling grants.

| Convenience | Meaning | Example |
| --- | --- | --- |
| `$home`, `~` | Home root supplied in `CompileCtx`. | `"$home/.config/tool": "r"` |
| `$cache` | One standard OS cache root, supplied in `CompileCtx`. | `"$cache/tool": "rw"` |
| `$tmp` | Managed private storage retained for the resource's lifetime. | `"$tmp": "rw"` |
| `$tooldirs` | A union of conventional tool directories and approved environment relocations. | `"$tooldirs": "rw"` |

The standard cache roots are `XDG_CACHE_HOME` or `~/.cache` on Linux, `~/Library/Caches` on macOS, and `LOCALAPPDATA` on Windows. The cache convenience does not include every tool's storage location. A dot-directory outside that root needs its own grant unless it is a member of the tool-directory set:

```json
{
  "fs": {
    "./": "rw",
    "$cache": "rw",
    "$home/.custom-tool": "rw",
    "$tmp": "rw"
  }
}
```

The array form `{"fs":["./","$tooldirs","$tmp"]}` means read-write. The object form makes access explicit. Private temp accepts `"rw"` or `true`; `false` requests no temp access. It rejects read-only access and suffixes such as `$tmp/work`. The tool-directory set also takes no suffix.

A whole-home grant is literal:

```json
{"fs":{"$home":"rw","$tmp":"rw"}}
```

It includes SSH keys, package-manager credentials and other readable home files. Neither home grants nor tool-directory grants promise secret-free contents.

## Tool directories

The set includes package caches, stores, global installations and user-level tool state. Its members cover Nub, npm, pnpm, Yarn, Bun, pip, uv, Cargo/rustup, Go, Gradle, Maven, NuGet, Composer and Git. The [member table and environment mapping](src/compiler/builtin_sets.rs) are the implementation reference.

Examples of conventional roots:

| Tool | Linux | macOS | Windows |
| --- | --- | --- | --- |
| Nub | `~/.cache/nub`, `~/.config/nub`, `~/.local/share/nub/store` | Same; also `$cache/nub/pm` for embedder-supplied cache anchors | `~/.cache/nub`, `~/.config/nub`, `~/AppData/Local/nub`, `~/AppData/Roaming/nub` |
| npm | `~/.npm` | `~/.npm` | `~/AppData/Local/npm-cache`, `~/AppData/Roaming/npm` |
| pnpm | `~/.local/share/pnpm`, `~/.cache/pnpm`, `~/.config/pnpm`, `~/.local/state/pnpm` | `~/Library/pnpm`, `~/Library/Caches/pnpm`, `~/Library/Preferences/pnpm`, `~/.local/state/pnpm` | `~/AppData/Local/pnpm`, `~/AppData/Local/pnpm-cache`, `~/AppData/Local/pnpm-state`; legacy `~/.pnpm`, `~/.pnpm-cache`, `~/.pnpm-state`, `~/.config/pnpm` |
| Bun | `~/.bun/install` | `~/.bun/install` | `~/.bun/install` |
| Yarn | `~/.cache/yarn`, `~/.config/yarn`, `~/.yarn` | `~/Library/Caches/Yarn`, `~/.config/yarn`, `~/.yarn` | `~/AppData/Local/Yarn`, `~/AppData/Roaming/Yarn`, `~/.yarn` |
| pip | `~/.cache/pip`, `~/.config/pip`, `~/.pip`, `~/.local/lib` | `~/Library/Caches/pip`, `~/Library/Application Support/pip`, `~/.config/pip`, `~/.pip`, `~/Library/Python` | `~/AppData/Local/pip`, `~/AppData/Roaming/pip`, `~/pip`, `~/AppData/Roaming/Python` |
| uv | `~/.cache/uv`, `~/.local/share/uv`, `~/.config/uv`, `~/.local/bin` | The Linux roots, plus legacy `~/Library/Caches/uv` and `~/Library/Application Support/uv` | `~/AppData/Local/uv`, `~/AppData/Roaming/uv`, `~/.local/bin` |
| Cargo/rustup | `~/.cargo`, `~/.rustup` | `~/.cargo`, `~/.rustup` | `~/.cargo`, `~/.rustup` |
| Go | `~/go`, `$cache/go-build` | `~/go`, `~/Library/Caches/go-build` | `~/go`, `~/AppData/Local/go-build` |
| Gradle/Maven | `~/.gradle`, `~/.m2` | `~/.gradle`, `~/.m2` | `~/.gradle`, `~/.m2` |
| .NET/NuGet | `~/.dotnet`, `~/.nuget`, `~/.local/share/NuGet` | The Linux roots | `~/.dotnet`, `~/.nuget`, `~/AppData/Local/NuGet`, `~/AppData/Roaming/NuGet` |
| Composer | `~/.cache/composer`, `~/.composer`, `~/.config/composer` | `~/.composer`, `~/Library/Caches/composer`, `~/Library/Application Support/Composer` | `~/AppData/Local/Composer`, `~/AppData/Roaming/Composer` |
| Git | `~/.config/git`, `~/.git-credential-cache`; files `~/.gitconfig`, `~/.gitconfig.lock`, `~/.git-credentials` | Same | Same |

Documented environment locations are expanded from the compilation snapshot, including `NPM_CONFIG_CACHE`, `PNPM_HOME`, `YARN_CACHE_FOLDER`, `BUN_INSTALL`, `UV_CACHE_DIR`, `CARGO_HOME`, `GOPATH`, `GRADLE_USER_HOME`, `NUGET_PACKAGES` and `COMPOSER_HOME`. The set also includes tool-specific children of supplied XDG and Windows app-data roots. It does not execute tools, inspect PATH, parse their configuration files or scan the disk.

These values add grants alongside the conventional roots. They do not change the access mode selected for `$tooldirs`, and expanding a variable does not itself pass that variable to the command.

| Tool | Additional path variables |
| --- | --- |
| PM storage | `NPM_CONFIG_CACHE_DIR`, `NPM_CONFIG_STORE_DIR`, `NPM_CONFIG_VIRTUAL_STORE_DIR`, `NPM_CONFIG_GLOBAL_VIRTUAL_STORE_DIR`; their lowercase spellings also work. |
| npm | `NPM_CONFIG_CACHE`, `NPM_CONFIG_PREFIX`, `NPM_CONFIG_USERCONFIG`, `NPM_CONFIG_GLOBALCONFIG`; their lowercase spellings also work. |
| pnpm | `PNPM_HOME`, `PNPM_CONFIG_STORE_DIR`, `PNPM_CONFIG_CACHE_DIR`, `PNPM_CONFIG_STATE_DIR`, `PNPM_CONFIG_CONFIG_DIR` |
| Yarn | `YARN_CACHE_FOLDER`, `YARN_GLOBAL_FOLDER` |
| Bun | `BUN_INSTALL`, `BUN_INSTALL_CACHE_DIR`, `BUN_INSTALL_GLOBAL_DIR`, `BUN_INSTALL_BIN` |
| pip/Python | `PIP_CACHE_DIR`, `PIP_CONFIG_FILE`, `PYTHONUSERBASE` |
| uv | `UV_CACHE_DIR`, `UV_TOOL_DIR`, `UV_TOOL_BIN_DIR`, `UV_PYTHON_INSTALL_DIR`, `UV_PYTHON_BIN_DIR`, `UV_INSTALL_DIR`, `UV_PROJECT_ENVIRONMENT` |
| Cargo/rustup | `CARGO_HOME`, `RUSTUP_HOME`, `CARGO_TARGET_DIR` |
| Go | `GOPATH`, `GOMODCACHE`, `GOCACHE`, `GOBIN`, `GOTMPDIR` |
| Gradle | `GRADLE_USER_HOME` |
| .NET/NuGet | `NUGET_PACKAGES`, `NUGET_HTTP_CACHE_PATH`, `NUGET_SCRATCH`, `NUGET_PLUGINS_CACHE_PATH`, `DOTNET_CLI_HOME`, `DOTNET_BUNDLE_EXTRACT_BASE_DIR` |
| Composer | `COMPOSER_HOME`, `COMPOSER_CACHE_DIR`, `COMPOSER_VENDOR_DIR`, `COMPOSER_BIN_DIR` |
| Git | `GIT_DIR`, `GIT_COMMON_DIR`, `GIT_CONFIG_GLOBAL`, `GIT_TEMPLATE_DIR`, `GIT_EXEC_PATH` |

Empty values are ignored. Go's `GOPATH` is split into separate roots using the OS path-list separator. Standard XDG and Windows app-data variables add the tool-specific children in the member table, not the entire app-data directory. For Nub these are `XDG_CACHE_HOME/nub`, `XDG_DATA_HOME/nub/store` and `XDG_CONFIG_HOME/nub`, plus the Windows app-data `nub` children. An override that resolves to a filesystem root is rejected.

For example, a supplied `UV_CACHE_DIR=/work/uv-cache` adds that location to `$tooldirs`. A relocation found only in a tool's configuration file needs an explicit grant instead:

```json
{"fs":{"./":"rw","$tooldirs":"rw","/work/custom-store":"rw","$tmp":"rw"}}
```

The uv defaults use XDG locations on both Linux and macOS. Its executable fallback follows `XDG_DATA_HOME/../bin`; `UV_INSTALL_DIR` and `UV_PROJECT_ENVIRONMENT` add their configured locations too. [uv storage reference](https://docs.astral.sh/uv/reference/storage/).

Operational limits:

- Linux Landlock and Windows ACL grants need existing objects. Missing speculative set members are skipped, not created. Initialize the cache root before acquiring a sandbox, or place it under an already writable directory. macOS path rules can admit later creation.
- An exact writable file is not a writable parent directory. Git's default global-config update creates an adjacent `.gitconfig.lock` and renames it; granting the existing `.gitconfig` alone is insufficient on inode-based backends. A dedicated, writable Git config directory supports that protocol without granting all of home.
- Removing or replacing a grant root may require write access to its parent. This affects commands such as `cargo clean` and cache deletion. Put disposable output below a writable directory, or explicitly grant its parent; the engine does not synthesize sibling exceptions.
- Linked Git worktrees and relocated common directories may sit outside the project. Supply `GIT_DIR`/`GIT_COMMON_DIR` or explicit grants for those locations.
- Filesystem access does not provide network access, an interpreter's installation files, macOS Keychain access or Windows Credential Manager access. Embedders supply those capabilities separately.

## Network and environment permissions

Network filtering is independent of filesystem access:

```json
{
  "fs": {"./": "rw", "$tooldirs": "rw", "$tmp": "rw"},
  "net": ["registry.npmjs.org", "*.example.com", "!admin.example.com"],
  "vars": {"PATH": true, "HOME": true, "CI?": true},
  "secrets": {"API_TOKEN?": true}
}
```

Network entries accept host patterns and CIDRs. Unlike filesystem grants, network rules retain ordered allow/deny matching. A host grant is not an HTTP-method restriction and does not prevent uploads to that host. The boolean `false` denies egress; `true` disables Nub's network filtering. Windows AppContainer capabilities still constrain networking even without a Nub host filter.

| OS | Host-filtered networking | Limits |
| --- | --- | --- |
| Linux | A seccomp notification supervisor redirects external TCP connections through the policy proxy. | No client proxy configuration is required. DNS uses the configured resolver; direct IPv4 loopback remains available. General UDP is denied. Host rules are not an all-channel data-loss boundary. |
| macOS | Seatbelt allows the proxy's loopback port; the proxy checks destinations. | Clients need HTTP CONNECT or SOCKS proxy support. Bypassing the proxy does not grant direct external access. |
| Windows | A same-AppContainer helper provides the proxy; the command has no direct Internet capability. | Clients need proxy support. The helper rejects TLS-inspection and credential-broker policies. No administrator loopback exemption is installed. |

Coarse `net: true` and `net: false` policies do not start a host-filtering proxy. The build jail uses coarse catalog network permissions rather than enforcing its recorded observed-host lists. On Windows, the coarse allow grants public outbound networking, not unrestricted host/LAN/loopback access.

The environment example inherits named values from the supplied snapshot. A trailing `?` makes a missing value optional. Secret values are sensitive data supplied to the child, not values hidden from it; an allowed child can use them. Unlisted environment values are not implicitly inherited by this explicit policy.

Filtering the environment does not hide files granted through `fs`. The default Linux policy also withholds other processes' `/proc` entries, which can break tools that inspect their own process metadata. Granting all of `/proc` would broaden access to other processes and is not an automatic compatibility fix.

## Resource and command ownership

Compile once, acquire resources, prepare commands and release the session:

```rust,ignore
let policy = nub_sandbox::compile(&permissions, &context)?;
let sandbox = nub_sandbox::Sandbox::acquire(&policy)?;
let command = nub_sandbox::CommandSpec::new(program).args(arguments).cwd(project);
let prepared = sandbox.prepare(command)?;
// Surface prepared.degradation before treating this as confined execution.
let output = prepared.output()?;
let next = sandbox.prepare(next_command)?.spawn()?;
drop(next); // Stops and reaps this command tree.
sandbox.close();
nub_sandbox::cleanup()?;
```

Each prepared/running command retains its resource lease. Closing the session releases that caller's handle; it does not invalidate commands already prepared through it. A running command owns its streams, cancellation and exit status. There is no detach or reconnect operation.

| OS | Enforcement | Persistent changes |
| --- | --- | --- |
| Linux | Landlock filesystem restrictions and seccomp syscall filters; the general network path uses a notification supervisor. | No host ACL/firewall changes. Session temp, pipes and supervisor resources have owners. |
| macOS | Seatbelt profiles applied through `sandbox-exec`. | No host ACL/firewall changes. Session temp and proxy resources have owners. |
| Windows | AppContainer restricted identities, filesystem ACL grants and a kill-on-close Job for each command tree. | Profile storage, owned ACEs and a persistent recovery journal. |

Landlock is a Linux Security Module. Seccomp means secure computing; its BPF (Berkeley Packet Filter) programs filter system calls. An ACL is an Access Control List; an ACE is one entry in it. Windows identifies an AppContainer with a SID, or Security Identifier. A Job Object controls process lifetime independently of file permissions.

Windows automatically reuses equivalent resolved resource policies, including runtime grants and backend version. Different external paths produce different identities; managed temp is an identity-owned slot rather than a fresh hash input. Active leases are never evicted. The idle cache is bounded by 64 entries, 24 hours and 1 GiB of owned private data; caller project outputs and shared tool caches are not deletion targets. Explicit cleanup reports failures and retains their ownership records for recovery.

On Unix, managed temp storage lives in a private per-user directory under the OS temp root. A file-lock lease distinguishes a live session from an abandoned one. Acquisition and explicit cleanup recover abandoned owned directories; normal close removes them immediately. Cleanup checks the recorded directory identity and does not follow payload symlinks. Legacy temporary directories without ownership records are not deletion targets.

The CLI runs the same recovery operation without loading project configuration. A nonzero exit reports incomplete cleanup; its ownership records remain available for retry:

```sh
nub sandbox cleanup
```

The Windows build jail also publishes read access to Nub-owned public package caches. Those cache permissions are intentional shared storage metadata, not a particular session's grants; sandbox cleanup does not revoke them.

The API requires no elevation or setup command. Windows' full-disk build-jail compatibility path is deliberately unconfined and reports that loss; it still uses an owned Job. On Unix, owner-loss cleanup uses a private guardian process group. Linux additionally blocks group/session escape syscalls. macOS does not have a verified equivalent restriction: a process that deliberately leaves the group can survive owner loss, although it remains confined. Do not treat ordinary descendant tests as proof against deliberate detachment.
