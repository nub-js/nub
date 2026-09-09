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
| npm | `~/.npm` | `~/.npm` | `~/AppData/Local/npm-cache`, `~/AppData/Roaming/npm` |
| pnpm | `~/.local/share/pnpm`, `~/.cache/pnpm`, `~/.config/pnpm`, `~/.local/state/pnpm` | `~/Library/pnpm`, `~/Library/Caches/pnpm`, `~/Library/Preferences/pnpm`, `~/.local/state/pnpm` | `~/AppData/Local/pnpm`, `~/AppData/Local/pnpm-cache`, `~/AppData/Local/pnpm-state`; legacy `~/.pnpm`, `~/.pnpm-cache`, `~/.pnpm-state`, `~/.config/pnpm` |
| Bun | `~/.bun/install` | `~/.bun/install` | `~/.bun/install` |
| Yarn | `~/.cache/yarn`, `~/.config/yarn`, `~/.yarn` | `~/Library/Caches/Yarn`, `~/.config/yarn`, `~/.yarn` | `~/AppData/Local/Yarn`, `~/AppData/Roaming/Yarn`, `~/.yarn` |
| pip | `~/.cache/pip`, `~/.config/pip`, `~/.pip`, `~/.local/lib` | `~/Library/Caches/pip`, `~/Library/Application Support/pip`, `~/.config/pip`, `~/.pip`, `~/Library/Python` | `~/AppData/Local/pip`, `~/AppData/Roaming/pip`, `~/pip`, `~/AppData/Roaming/Python` |
| uv | `~/.cache/uv`, `~/.local/share/uv`, `~/.config/uv`, `~/.local/bin` | The Linux roots, plus legacy `~/Library/Caches/uv` and `~/Library/Application Support/uv` | `~/AppData/Local/uv`, `~/AppData/Roaming/uv`, `~/.local/bin` |
| Cargo/rustup | `~/.cargo`, `~/.rustup` | `~/.cargo`, `~/.rustup` | `~/.cargo`, `~/.rustup` |
| Go | `~/go`, `$cache/go-build` | `~/go`, `~/Library/Caches/go-build` | `~/go`, `~/AppData/Local/go-build` |
| Gradle/Maven | `~/.gradle`, `~/.m2` | `~/.gradle`, `~/.m2` | `~/.gradle`, `~/.m2` |
| .NET/NuGet | `~/.dotnet`, `~/.nuget`, `~/.local/share/NuGet` | The Linux roots | `~/.dotnet`, `~/.nuget`, `~/AppData/Local/NuGet` |
| Composer | `~/.cache/composer`, `~/.composer`, `~/.config/composer` | `~/.composer`, `~/Library/Caches/composer`, `~/Library/Application Support/Composer` | `~/AppData/Local/Composer`, `~/AppData/Roaming/Composer` |
| Git | `~/.config/git`, `~/.git-credential-cache`; files `~/.gitconfig`, `~/.gitconfig.lock`, `~/.git-credentials` | Same | Same |

Documented environment locations are expanded from the compilation snapshot, including `NPM_CONFIG_CACHE`, `PNPM_HOME`, `YARN_CACHE_FOLDER`, `BUN_INSTALL`, `UV_CACHE_DIR`, `CARGO_HOME`, `GOPATH`, `GRADLE_USER_HOME`, `NUGET_PACKAGES` and `COMPOSER_HOME`. The set also includes tool-specific children of supplied XDG and Windows app-data roots. It does not execute tools, inspect PATH, parse their configuration files or scan the disk.

For example, a supplied `UV_CACHE_DIR=/work/uv-cache` adds that location to `$tooldirs`. A relocation found only in a tool's configuration file needs an explicit grant instead:

```json
{"fs":{"./":"rw","$tooldirs":"rw","/work/custom-store":"rw","$tmp":"rw"}}
```

The uv defaults use XDG locations on both Linux and macOS. Its executable fallback follows `XDG_DATA_HOME/../bin`; `UV_INSTALL_DIR` and `UV_PROJECT_ENVIRONMENT` add their configured locations too. [uv storage reference](https://docs.astral.sh/uv/reference/storage/).

Operational limits:

- Linux Landlock and Windows ACL grants need existing objects. Missing speculative set members are skipped, not created. Initialize the cache root before acquiring a sandbox, or place it under an already writable directory. macOS path rules can admit later creation.
- An exact writable file is not a writable parent directory. Git's default global-config update creates an adjacent `.gitconfig.lock` and renames it; granting the existing `.gitconfig` alone is insufficient on inode-based backends. A dedicated, writable Git config directory supports that protocol without granting all of home.
- Linked Git worktrees and relocated common directories may sit outside the project. Supply `GIT_DIR`/`GIT_COMMON_DIR` or explicit grants for those locations.
- Filesystem access does not provide network access, an interpreter's installation files, macOS Keychain access or Windows Credential Manager access. Embedders supply those capabilities separately.

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

The API requires no elevation or setup command. Windows' full-disk build-jail compatibility path is deliberately unconfined and reports that loss; it still uses an owned Job. On Unix, owner-loss cleanup uses a private guardian process group. Linux additionally blocks group/session escape syscalls. macOS does not have a verified equivalent restriction: a process that deliberately leaves the group can survive owner loss, although it remains confined. Do not treat ordinary descendant tests as proof against deliberate detachment.
