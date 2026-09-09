# Tool compatibility

The filesystem convenience set does not make every runtime compatible with every OS sandbox. This matrix records complete tested operation sequences, not a guarantee for every command a tool supports.

## Test conditions

The [native run at `0960f8a1b8`](https://github.com/nubjs/nub/actions/runs/34348165396) compares unconfined, explicit-path and tool-directory policies. The confined cases call the Rust engine directly with this filesystem policy:

```json
{"fs":["./","$tooldirs","$tmp"]}
```

The fixtures also grant the tested interpreter's installation files and supply its environment and network requirements. Cache roots exist before acquisition. Neither the entire home directory nor the entire filesystem is granted. The exact fixtures are [JavaScript package managers](tests/tool_functionality.rs), [Python](tests/python_tool_functionality.rs), [native toolchains](tests/native_tool_functionality.rs) and [Git](tests/git_tool_functionality.rs).

## Versioned results

“Pass” means both confined variants completed the sequence and the unconfined control passed. Windows native toolchains were tested on Server 2022 x86-64; the JavaScript, Python and Git suites also ran on Windows 11 arm64.

| Tool version | Linux x86-64 | macOS arm64 | Windows | Tested sequence or limit |
| --- | --- | --- | --- | --- |
| npm 11.6.2 | Pass | Pass | Pass | Install, reinstall, installed-bin execution, user-global operations and cache maintenance, with existing cache roots. |
| pnpm 9.15.9 / 10.18.3 | Pass | Pass | Blocked | Windows confined commands reach the deadline; raw Node's global named-pipe behavior is an established backend incompatibility. |
| pnpm 11.26.0 | Pass | Pass | Blocked | Windows local installation fails with `EPERM` from `realpath` on the project directory. This is distinct from the older versions' timeout. |
| Yarn 1.22.22 | Blocked | Pass | Blocked | Linux process-memory inspection needs per-process procfs access; Windows IPC prevents completion. |
| Yarn 2.4.2 / 3.8.7 / 4.17.0 | Pass | Pass | Pass | Install, reinstall and execution through the configured store. |
| Bun 1.3.2 | Blocked | Blocked | Blocked | Installed-bin execution reports `CouldntReadCurrentDirectory`; a readable project does not grant arbitrary ancestors. |
| Bun 1.4.0 | Blocked | Blocked | Blocked | Linux reports a JSON stack-depth error; macOS cache deletion needs a writable parent. Windows does not complete the confined sequence. |
| pip 26.2.1 / uv 0.12.11 | Pass | Pass | Blocked | Windows pip cannot write inside its newly created private temp directory. Uv separately receives access denied while querying its interpreter; the exact denied operation is not established. Unix uv also passes with its default cache location. |
| Cargo 1.91.1 | Pass with project target | Pass | Blocked | Build and clean pass with the default project-local target on Unix. Deleting a separately granted target root requires its parent's write permission. Windows compiler subprocess access is denied. |
| rustup 1.29.0 | Pass | Pass | Pass | Installed toolchain and home queries; this does not certify installation of every toolchain. |
| Go 1.25.1 | Pass | Pass | Blocked | User config write/read, build, install and cache cleanup. Server passes config write/read but compilation fails opening `NUL`. |
| Gradle 8.14 | Pass | Pass | Blocked | Offline task, repeated task and daemon cleanup. On Windows, preparation reports that full networking cannot be supplied. The fixture rejects this before launching the workload; this is not a measured Gradle runtime failure. |
| Maven 3.9.11 | Pass | Pass | Pass | Offline validation and clean; both commands execute the user startup file. |
| .NET SDK 10.0.100 / NuGet | Blocked | Blocked | Pass | Restore, build and cache cleanup. Linux CoreCLR initialization fails; macOS requires shared `/tmp` coordination outside private temp. |
| Composer 2.8.12 | Pass | Pass | Blocked | Cold/warm install without plugins or scripts, then cache cleanup. Windows confined subprocess access is denied. |

The JavaScript fixtures use Node 22.18.0. Bun 1.3.2 uses x64 emulation on Windows arm64. These are recorded versions, not minimum supported versions. Distinct pnpm and Yarn versions exercise their different storage layouts; adding a directory member does not imply that an older or newer runtime was tested.

## Explicit Windows Node adapters

The raw results above do not include runtime preloads. The [explicit-adapter run](https://github.com/nubjs/nub/actions/runs/34400945240), at `585cb0ee26`, tests `windows_node_compat_options` with Node 22.18.0 on Server 2022 x86-64 and Windows 11 arm64. Both hosts pass 14 cases, including four unconfined controls and two intentional root-only cache-cleanup denials.

| Tool | Explicit-grant and tool-directory results |
| --- | --- |
| pnpm 9.15.9 / 10.18.3 / 11.26.0 | Local install, retained reinstall, installed-bin execution, global install, cache prune and another reinstall pass in one session. Denied-file canaries and explicit cleanup pass. |
| Yarn 1.22.22 | Install, reinstall, bin execution and global install pass. Root-only grants correctly refuse cache-root recreation. With an explicit dedicated writable cache parent, cleanup and another reinstall pass in the same session. Denied-file canaries and explicit cleanup pass. |

The [fixture](tests/tool_functionality.rs) retains both cache-grant variants. This is opt-in Node adaptation, not a filesystem permission expansion or a change to raw execution. The helper includes no build-jail package-network policy. See the [setup and behavioral limits](README.md#explicit-windows-node-compatibility).

## Git


The Unix sequence covers status, add, commit, clone, fetch, push, linked worktrees and Git LFS. It passes on Linux and macOS with explicit grants for the repository/common-directory locations and a dedicated writable global-config directory. macOS also passes the conventional home-level global-config update. Windows sequences remain incomplete because of device and subprocess access restrictions.

An [installation-closure test](https://github.com/nubjs/nub/actions/runs/34408888398) adds explicit read access to the entire known Git installation. This does not repair the Windows Git/LFS sequences. The [startup controls](https://github.com/nubjs/nub/actions/runs/34409754352) distinguish direct Git from its bundled MSYS shell: direct Git runs in all seven descriptor configurations on Windows 11 arm64, while the shell exits with `0xC0000005` even for a builtin-only command. On Server 2022, Git reports denied `/dev/null` access and shell startup exits with `0xC0000142`. Every plain control passes and every confined denied-file canary remains inaccessible. These results establish the failure stage, not the immediate cause of the Windows 11 access violation.

Git creates an adjacent lock file and renames it when updating global configuration. A grant on an existing file cannot substitute for parent-directory write access on Linux and Windows. For example, with `GIT_CONFIG_GLOBAL=/work/git-config/config` supplied by the embedder:

```json
{
  "fs": {"./":"rw", "$tooldirs":"rw", "/work/git-config":"rw", "$tmp":"rw"},
  "vars": {"PATH":true, "GIT_CONFIG_GLOBAL":true}
}
```

The writable directory admits the lock/rename protocol without granting all of home. The same principle applies to deleting a cache or build-output root. Configuration-file-only relocations still require explicit paths.

## Backend restrictions

### Windows Gradle network qualification

The strict matrix refuses Windows' `net-full` degradation before launching Gradle. A [separate execution probe](https://github.com/nubjs/nub/actions/runs/34403539949) explicitly acknowledges that limitation and passes on Server 2022 and Windows 11 arm64: Gradle 8.14 with Temurin 21.0.8 runs an offline task twice and performs daemon cleanup through one retained session. Its plain controls also pass. The [diagnostic test](tests/native_tool_functionality.rs) asserts the exact degradation rather than ignoring all unsupported permissions.

This establishes that the tested Gradle workflow works with the narrower capability. It does not establish full host networking, access to arbitrary host-loopback services, or an unqualified strict-policy pass. No network grants or backend behavior were changed for this probe.

### OS restrictions

- **Linux procfs:** ordinary grants under `/proc` are rejected. The two explicit read-only [self-metadata permissions](README.md#explicit-linux-process-metadata) provide the requesting process's maps or statistics without granting another process's files. Neither is enabled by default.
- **Windows private ACLs:** applications can create protected directory ACLs that omit the AppContainer identity. Broader grants on an ancestor do not repair that behavior. Python's private-directory behavior exists in maintained older versions too; selecting an old minor release is not a general workaround.
- **Windows devices and IPC:** filesystem paths do not grant access to every named pipe, the `NUL` device or additional networking capabilities. Server and Windows 11 results differ. The engine does not install administrator device permissions or loopback exemptions.
- **macOS shared temp:** private `TMPDIR` does not relocate paths hardcoded by a runtime. An explicit shared-path grant changes isolation and is not silently added by the tool-directory set.

### Linux failure isolation

The [selective syscall-injection run](https://github.com/nubjs/nub/actions/runs/34404715809) tests otherwise unconfined tools, with both plain and tracing-only controls. All controls pass. A separate file-read probe verifies that injection denies only the selected procfs path while ordinary file reads still work.

- Denying only `/proc/self/maps` reproduces Bun 1.4.0's JSON nesting/stack-overflow error during a local install.
- The same isolated denial reproduces CoreCLR initialization error `0x8007000E` during restore. This run selected SDK 10.0.400 and runtime 10.0.12, as recorded by the loaded paths in its trace; the provisioning version did not pin the workload. Denying FIFO creation alone does **not** prevent that restore.
- Denying only `/proc/self/stat` reproduces Node 22.18.0's `uv_resident_set_memory` error from `process.memoryUsage()`, the call made by Yarn Classic's unconditional memory reporter.

These denials are sufficient to reproduce the reported failures; fixing one does not prove no further restriction will be encountered. The [diagnostic harness](../../scripts/sandbox-procfs-diagnostics.py) changes no sandbox policy. Default policies still exclude procfs.

### Windows subprocess controls

The [focused subprocess run](https://github.com/nubjs/nub/actions/runs/34369854867) separates executable access from stream setup. It tests Rust 1.98.1, Python 3.12.10 and the diagnostic executable with seven descriptor configurations, each confined and unconfined. All unconfined cases pass. The confined cases retain a denied-file canary.

On Server 2022, the same executables launch with inherited streams, piped stdin or regular-file streams. Opening `NUL` for stdin or stdout fails with OS error 5. Default Rust `Command::output()` fails, while changing only its stdin to an existing empty file succeeds. All seven configurations pass on Windows 11 arm64.

The parent sandbox launcher can supply an existing handle; it cannot make an unmodified descendant's later `NUL` open succeed. The [diagnostic fixture](tests/windows_subprocess_diagnostics.rs) preserves the individual results rather than treating unsupported configurations as working.

### Windows 11 native tool sequences

The [Windows 11 run](https://github.com/nubjs/nub/actions/runs/34405367987), at `04566d64c9`, exercises native toolchains separately from the Server results above. Cargo 1.91.1, Go 1.25.1 and Composer 2.8.12 pass their complete exact-grant and tool-directory sequences, as do their plain controls. Maven 3.9.11 and rustup 1.29.1 also pass. Gradle passes with the explicitly acknowledged network limitation; its two strict-policy cases refuse preparation.

The runner is Windows 11 arm64. Cargo, Go and the JVM use x64 emulation with x64 MSVC libraries; .NET uses the ARM64 host. These results are not Server results or an all-ARM64 toolchain test. The .NET workload's `global.json` pins SDK 10.0.100, while the original provisioning metadata lists the runner's other installed SDKs too. Installing an SDK alone does not select it for a project.

The [SDK-selection run](https://github.com/nubjs/nub/actions/runs/34407256061), at `f5d73291ad`, asserts `dotnet --version` is exactly `10.0.100` inside each plain, explicit-grant and tool-directory fixture. All three restore/build/cache-cleanup sequences pass.

## Nub build-jail coverage

The build-jail frontend supplies a provisioned Node runtime and its own stdio support; it is not the same configuration as these raw engine tests. The [paired full-application run](https://github.com/nubjs/nub/actions/runs/34344382169) passes 16 framework fixtures per OS on Linux, macOS and Windows, both at `ee71d441c3` and at the exact preceding branch baseline. Each fixture includes an unconfined control, denied read/write/environment canaries and a frozen reinstall. Those results do not turn the raw-runtime failures above into passes.
