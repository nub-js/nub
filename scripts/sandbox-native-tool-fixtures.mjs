#!/usr/bin/env node
/** Provision the pinned native tool matrix used by native_tool_functionality.rs. */
import { appendFileSync, existsSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { delimiter, dirname, join, resolve } from 'node:path';
import { execFileSync } from 'node:child_process';

const root = resolve(process.env.NUB_SANDBOX_NATIVE_TOOL_ROOT ?? '.sandbox-native-tool-fixtures');
const pins = { cargo: '1.91.1', go: '1.25.1', gradle: '8.14', maven: '3.9.11', nuget: '10.0.100', composer: '2.8.12' };
const selected = process.argv[2]?.startsWith('--tools=') ? process.argv[2].slice(8).split(',') : null;
if (process.argv.length > 3 || (process.argv[2] && !selected) || selected?.some(name => ![...Object.keys(pins), 'rustup'].includes(name))) {
  throw new Error('usage: sandbox-native-tool-fixtures.mjs [--tools=cargo,go,composer,...]');
}
const wants = name => !selected || selected.includes(name);
const windows = process.platform === 'win32';
const platform = windows ? 'windows' : process.platform;
const arch = process.arch === 'arm64' ? 'arm64' : 'amd64';

function find(command) {
  const names = windows ? [command, `${command}.exe`, `${command}.cmd`, `${command}.bat`] : [command];
  for (const dir of (process.env.PATH ?? '').split(delimiter)) for (const name of names) {
    const candidate = join(dir, name);
    if (existsSync(candidate)) return candidate;
  }
  throw new Error(`${command} is missing; provision it before this fixture`);
}
function output(program, args, options = {}) {
  if (!existsSync(program)) throw new Error(`provisioned executable is missing: ${program}`);
  const command = windows && /\.(cmd|bat)$/i.test(program) ? process.env.COMSPEC ?? 'cmd.exe' : program;
  const quote = (value) => `"${value.replaceAll('"', '\\"')}"`;
  const commandArgs = command === program ? args : ['/d', '/s', '/c', `"${[program, ...args].map(quote).join(' ')}"`];
  return execFileSync(command, commandArgs, {
    encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'],
    windowsVerbatimArguments: command !== program,
    ...options,
  }).trim();
}
async function fetchFile(url, destination) {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`download ${url} failed: ${response.status}`);
  writeFileSync(destination, Buffer.from(await response.arrayBuffer()));
}
function verify(file, expected, algorithm) {
  const actual = createHash(algorithm).update(readFileSync(file)).digest('hex');
  if (actual !== expected.trim().split(/\s+/)[0]) throw new Error(`checksum mismatch for ${file}: ${actual}`);
}
async function downloadChecked(url, checksumUrl, algorithm, destination) {
  await fetchFile(url, destination);
  const response = await fetch(checksumUrl);
  if (!response.ok) throw new Error(`checksum download ${checksumUrl} failed: ${response.status}`);
  verify(destination, await response.text(), algorithm);
}
function extractArchive(archive, destination) {
  mkdirSync(destination, { recursive: true });
  if (windows && archive.endsWith('.zip')) {
    execFileSync('powershell.exe', ['-NoProfile', '-Command', `Expand-Archive -LiteralPath '${archive.replaceAll("'", "''")}' -DestinationPath '${destination.replaceAll("'", "''")}' -Force`], { stdio: 'inherit' });
  } else {
    execFileSync(archive.endsWith('.zip') ? 'unzip' : 'tar', archive.endsWith('.zip') ? ['-q', archive, '-d', destination] : ['-xf', archive, '-C', destination], { stdio: 'inherit' });
  }
}
async function installRust() {
  const hostRustup = find('rustup');
  const toolEnv = { RUSTUP_HOME: join(root, 'rustup'), CARGO_HOME: join(root, 'cargo') };
  if (process.env.RUSTUP_OVERRIDE_HOST_TRIPLE) {
    toolEnv.RUSTUP_OVERRIDE_HOST_TRIPLE = process.env.RUSTUP_OVERRIDE_HOST_TRIPLE;
  }
  mkdirSync(toolEnv.RUSTUP_HOME, { recursive: true });
  mkdirSync(toolEnv.CARGO_HOME, { recursive: true });
  output(hostRustup, ['toolchain', 'install', pins.cargo, '--profile', 'minimal', '--no-self-update'], { env: { ...process.env, ...toolEnv } });
  const runtimeRoots = [join(toolEnv.RUSTUP_HOME, 'toolchains')];
  if (windows) {
    const vswhere = join(process.env['ProgramFiles(x86)'], 'Microsoft Visual Studio', 'Installer', 'vswhere.exe');
    const installation = output(vswhere, ['-latest', '-products', '*', '-requires', 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64', '-property', 'installationPath']);
    if (!installation) throw new Error('MSVC toolchain missing from the native runner');
    const setup = join(installation, 'Common7', 'Tools', 'VsDevCmd.bat');
    const environment = output(process.env.COMSPEC, ['/d', '/s', '/c', `""${setup}" -arch=x64 -host_arch=x64 >nul && set"`], { windowsVerbatimArguments: true });
    const vars = new Map(environment.split(/\r?\n/).filter(line => line.includes('=')).map(line => {
      const at = line.indexOf('=');
      return [line.slice(0, at).toUpperCase(), line.slice(at + 1)];
    }));
    // Git Bash's link.exe is not the MSVC linker. Carry the developer prompt's
    // explicit executable/library closure into both confined and plain controls.
    for (const key of ['PATH', 'LIB', 'LIBPATH', 'INCLUDE']) {
      if (!vars.get(key)) throw new Error(`MSVC setup omitted ${key}`);
      toolEnv[key] = vars.get(key);
    }
    for (const key of ['VCTOOLSINSTALLDIR', 'WINDOWSSDKDIR']) {
      if (!vars.get(key)) throw new Error(`MSVC setup omitted ${key}`);
      runtimeRoots.push(vars.get(key));
    }
  }
  return { cargo: find('cargo'), rustup: hostRustup, toolEnv, runtimeRoots };
}
async function installGo() {
  const extension = windows ? 'zip' : 'tar.gz';
  const filename = `go${pins.go}.${platform}-${arch}.${extension}`;
  const releases = await (await fetch('https://go.dev/dl/?mode=json&include=all')).json();
  const file = releases.find((release) => release.version === `go${pins.go}`)?.files.find((candidate) => candidate.filename === filename);
  if (!file) throw new Error(`official Go metadata omitted ${filename}`);
  const archive = join(root, filename);
  await fetchFile(`https://go.dev/dl/${filename}`, archive); verify(archive, file.sha256, 'sha256'); extractArchive(archive, join(root, 'go-dist'));
  return { program: join(root, 'go-dist', 'go', 'bin', windows ? 'go.exe' : 'go'), root: join(root, 'go-dist', 'go') };
}
async function installJvmTools() {
  const gradleArchive = join(root, `gradle-${pins.gradle}-bin.zip`);
  await downloadChecked(`https://services.gradle.org/distributions/gradle-${pins.gradle}-bin.zip`, `https://services.gradle.org/distributions/gradle-${pins.gradle}-bin.zip.sha256`, 'sha256', gradleArchive); extractArchive(gradleArchive, join(root, 'gradle-dist'));
  const mavenArchive = join(root, `apache-maven-${pins.maven}-bin.zip`);
  await downloadChecked(`https://archive.apache.org/dist/maven/maven-3/${pins.maven}/binaries/apache-maven-${pins.maven}-bin.zip`, `https://archive.apache.org/dist/maven/maven-3/${pins.maven}/binaries/apache-maven-${pins.maven}-bin.zip.sha512`, 'sha512', mavenArchive); extractArchive(mavenArchive, join(root, 'maven-dist'));
  const ext = windows ? '.bat' : '';
  const mavenRoot = join(root, 'maven-dist', `apache-maven-${pins.maven}`);
  const maven = join(mavenRoot, 'bin', windows ? 'mvn.cmd' : 'mvn');
  const seedProject = join(root, 'maven-seed-project');
  const seedRepository = join(root, 'maven-seed-repository');
  mkdirSync(seedProject, { recursive: true });
  writeFileSync(join(seedProject, 'pom.xml'), '<project xmlns="http://maven.apache.org/POM/4.0.0"><modelVersion>4.0.0</modelVersion><groupId>example</groupId><artifactId>seed</artifactId><version>1</version><build><plugins><plugin><groupId>org.apache.maven.plugins</groupId><artifactId>maven-clean-plugin</artifactId><version>3.4.1</version></plugin></plugins></build></project>\n');
  output(maven, ['-B', `-Dmaven.repo.local=${seedRepository}`, 'dependency:go-offline'], { cwd: seedProject });
  return { gradle: { program: join(root, 'gradle-dist', `gradle-${pins.gradle}`, 'bin', `gradle${ext}`), root: join(root, 'gradle-dist', `gradle-${pins.gradle}`) }, maven: { program: maven, root: mavenRoot, seedRepository } };
}
async function installComposer() {
  const composerRoot = join(root, 'composer-dist');
  mkdirSync(composerRoot, { recursive: true });
  const phar = join(composerRoot, 'composer.phar');
  await downloadChecked(`https://getcomposer.org/download/${pins.composer}/composer.phar`, `https://getcomposer.org/download/${pins.composer}/composer.phar.sha256sum`, 'sha256', phar);
  const php = find('php');
  const config = JSON.parse(output(php, ['-r', 'echo json_encode([php_ini_loaded_file(), php_ini_scanned_files()]);']));
  const configRoots = config.filter(Boolean).flatMap(value => value.split(/,\s*/))
    .map(value => value.trim()).filter(Boolean)
    .flatMap(file => [dirname(file), dirname(realpathSync(file))]);
  return { php, phar, root: composerRoot, configRoots: [...new Set(configRoots)] };
}
function entry(name, program, args, toolRoot, { prefix = [], runtimeRoots = [], toolEnv = {}, mavenSeed, cwd } = {}) {
  const version = output(program, [...prefix, ...args], { env: { ...process.env, ...toolEnv }, cwd });
  // The runner supplies rustup; record its exact version without updating the
  // engine's toolchain manager. Cargo itself uses the isolated pinned toolchain.
  if (pins[name] && !version.includes(pins[name])) throw new Error(`${name} version drift: expected ${pins[name]}, got ${version}`);
  if (name === 'nuget' && version !== pins.nuget) throw new Error(`selected .NET SDK drift: ${version}`);
  return { name, program, prefix, version, toolRoot, runtimeRoots, toolEnv, mavenSeed, shell: windows && /\.(cmd|bat)$/i.test(program) };
}

mkdirSync(root, { recursive: true });
const rust = wants('cargo') || wants('rustup') ? await installRust() : null;
const go = wants('go') ? await installGo() : null;
const jvm = wants('gradle') || wants('maven') ? await installJvmTools() : null;
const composer = wants('composer') ? await installComposer() : null;
const dotnet = wants('nuget') ? find('dotnet') : null;
const dotnetVersionRoot = join(root, 'dotnet-version');
if (dotnet) {
  mkdirSync(dotnetVersionRoot, { recursive: true });
  writeFileSync(join(dotnetVersionRoot, 'global.json'), `${JSON.stringify({ sdk: { version: pins.nuget, rollForward: 'disable' } })}\n`);
}
const runtimeRoots = [process.env.JAVA_HOME, process.env.DOTNET_ROOT].filter(Boolean);
if ((jvm || dotnet) && runtimeRoots.length !== 2) throw new Error('JAVA_HOME and DOTNET_ROOT must be set by workflow setup actions');
const runtimeEnv = { JAVA_HOME: process.env.JAVA_HOME, DOTNET_ROOT: process.env.DOTNET_ROOT };
const matrix = [
  wants('cargo') && entry('cargo', rust.cargo, ['--version'], rust.toolEnv.CARGO_HOME, { prefix: [`+${pins.cargo}`], runtimeRoots: rust.runtimeRoots, toolEnv: rust.toolEnv }),
  wants('rustup') && entry('rustup', rust.rustup, ['--version'], rust.toolEnv.RUSTUP_HOME, { runtimeRoots: [rust.toolEnv.RUSTUP_HOME], toolEnv: rust.toolEnv }),
  wants('go') && entry('go', go.program, ['version'], go.root, { runtimeRoots: [go.root] }),
  wants('gradle') && entry('gradle', jvm.gradle.program, ['--version'], jvm.gradle.root, { runtimeRoots, toolEnv: runtimeEnv }),
  wants('maven') && entry('maven', jvm.maven.program, ['--version'], jvm.maven.root, { runtimeRoots, toolEnv: runtimeEnv, mavenSeed: jvm.maven.seedRepository }),
  wants('nuget') && entry('nuget', dotnet, ['--version'], process.env.DOTNET_ROOT, { runtimeRoots, toolEnv: runtimeEnv, cwd: dotnetVersionRoot }),
  wants('composer') && entry('composer', composer.php, ['--version'], composer.root, { prefix: [composer.phar], runtimeRoots: [resolve(composer.php, '..'), ...composer.configRoots] }),
].filter(Boolean);
const matrixFile = join(root, 'matrix.json');
writeFileSync(matrixFile, `${JSON.stringify(matrix, null, 2)}\n`);
const line = `NUB_SANDBOX_NATIVE_TOOL_MATRIX_FILE=${matrixFile}`;
if (process.env.GITHUB_ENV) appendFileSync(process.env.GITHUB_ENV, `${line}\n`); else process.stdout.write(`${line}\n`);
