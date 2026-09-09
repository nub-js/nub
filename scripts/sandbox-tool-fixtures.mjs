#!/usr/bin/env node
/**
 * Provision the pinned JavaScript package-manager matrix used by
 * `crates/nub-sandbox/tests/tool_functionality.rs`.
 *
 * This runs outside the sandbox. The fixture tests install only a local package; no registry
 * traffic belongs to a native-enforcement assertion. Version pins are intentionally explicit:
 * pnpm 9/10/11 are separate compatibility targets, Yarn 1 is Classic, and Yarn 2/3/4 are Berry.
 */
import { appendFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { execFileSync } from 'node:child_process';

const root = resolve(process.env.NUB_SANDBOX_TOOL_ROOT ?? '.sandbox-tool-fixtures');
const node = process.env.NUB_SANDBOX_NODE ?? process.execPath;
const winArm64 = process.platform === 'win32' && process.arch === 'arm64';
// npm's public package is the provisioning source. These are pinned, not dist-tags:
// npm 11.6.2, pnpm 9.15.9 / 10.18.3 / 11.26.0, Yarn 1.22.22,
// @yarnpkg/cli-dist 2.4.2 / 3.8.7 / 4.17.0, Bun 1.3.2 / 1.4.0.
const packages = [
  ['npm', 'npm@11.6.2', 'node_modules/npm/bin/npm-cli.js', 'npm'],
  ['pnpm9', 'pnpm@9.15.9', 'node_modules/pnpm/bin/pnpm.cjs', 'pnpm'],
  ['pnpm10', 'pnpm@10.18.3', 'node_modules/pnpm/bin/pnpm.cjs', 'pnpm'],
  ['pnpm11', 'pnpm@11.26.0', 'node_modules/pnpm/bin/pnpm.cjs', 'pnpm'],
  ['yarn1', 'yarn@1.22.22', 'node_modules/yarn/bin/yarn.js', 'yarn1'],
  ['yarn2', '@yarnpkg/cli-dist@2.4.2', 'node_modules/@yarnpkg/cli-dist/bin/yarn.js', 'yarn'],
  ['yarn3', '@yarnpkg/cli-dist@3.8.7', 'node_modules/@yarnpkg/cli-dist/bin/yarn.js', 'yarn'],
  ['yarn4', '@yarnpkg/cli-dist@4.17.0', 'node_modules/@yarnpkg/cli-dist/bin/yarn.js', 'yarn'],
  // Bun 1.3.2 predates a Windows arm64 package. Windows-on-Arm can run the pinned x64
  // package under emulation; recording that distinction prevents it reading as native arm64 coverage.
  [
    'bun132',
    winArm64 ? '@oven/bun-windows-x64@1.3.2' : 'bun@1.3.2',
    process.platform === 'win32' ? 'node_modules/bun/bin/bun.exe' : 'node_modules/bun/bin/bun',
    'bun',
  ],
  ['bun140', 'bun@1.4.0', process.platform === 'win32' ? 'node_modules/bun/bin/bun.exe' : 'node_modules/bun/bin/bun', 'bun'],
];

function npmCli() {
  const candidates = [
    process.env.NUB_SANDBOX_NPM_CLI,
    join(dirname(node), '..', 'lib', 'node_modules', 'npm', 'bin', 'npm-cli.js'),
    join(dirname(node), 'node_modules', 'npm', 'bin', 'npm-cli.js'),
  ].filter(Boolean);
  const cli = candidates.find(existsSync);
  if (!cli) throw new Error(`npm-cli.js was not found; set NUB_SANDBOX_NPM_CLI (looked in ${candidates.join(', ')})`);
  return cli;
}

function install(prefix, spec) {
  mkdirSync(prefix, { recursive: true });
  execFileSync(node, [
    npmCli(), 'install', '--prefix', prefix, '--ignore-scripts=false', '--no-audit', '--no-fund', spec,
  ], {
    stdio: 'inherit',
    env: { ...process.env, npm_config_update_notifier: 'false' },
  });
}

function unpackWindowsX64Bun(prefix, spec) {
  const destination = join(prefix, 'node_modules/@oven/bun-windows-x64');
  mkdirSync(destination, { recursive: true });
  // Fetch the platform package without npm install's host-CPU admission check.
  // npm pack verifies registry integrity; this package contains a standalone binary.
  const packed = JSON.parse(execFileSync(node, [
    npmCli(), 'pack', spec, '--ignore-scripts', '--json', '--pack-destination', prefix,
  ], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }));
  if (packed.length !== 1) throw new Error(`unexpected packed files for ${spec}`);
  execFileSync('tar', ['-xzf', join(prefix, packed[0].filename), '-C', destination, '--strip-components=1'], {
    stdio: 'inherit',
  });
}

mkdirSync(root, { recursive: true });
const runtimeRoot = process.platform === 'win32' ? dirname(node) : dirname(dirname(node));
const matrix = [];
for (const [name, spec, relativeProgram, kind] of packages) {
  const prefix = join(root, name);
  if (name === 'bun132' && winArm64) unpackWindowsX64Bun(prefix, spec);
  else install(prefix, spec);
  // Bun publishes bin/bun.exe on Unix too; use the installed manifest rather
  // than assuming an OS-specific filename across release layouts.
  const bunPackage = name === 'bun132' && winArm64
    ? '@oven/bun-windows-x64'
    : 'bun';
  const architecture = name === 'bun132' && winArm64
    ? 'x64-emulated-on-win32-arm64'
    : process.arch;
  const program = kind === 'bun'
    ? join(
      prefix,
      'node_modules',
      bunPackage,
      winArm64 && name === 'bun132'
        ? 'bin/bun.exe'
        : JSON.parse(readFileSync(join(prefix, 'node_modules', bunPackage, 'package.json'), 'utf8')).bin.bun,
    )
    : join(prefix, relativeProgram);
  if (!existsSync(program)) throw new Error(`${name} ${spec} did not provide ${program}`);
  const output = execFileSync(
    kind === 'bun' ? program : node,
    kind === 'bun' ? ['--version'] : [program, '--version'],
    { encoding: 'utf8' },
  ).trim();
  if (!output) throw new Error(`${name} did not report a version`);
  matrix.push(kind === 'bun'
    ? { name, spec, kind, program, prefix: [], toolRoot: prefix, runtimeRoot, architecture, version: output }
    : { name, spec, kind, program: node, prefix: [program], toolRoot: prefix, runtimeRoot, architecture, version: output },
  );
}
const matrixFile = join(root, 'matrix.json');
writeFileSync(matrixFile, `${JSON.stringify(matrix, null, 2)}\n`);
const lines = [
  `NUB_SANDBOX_TOOL_MATRIX_FILE=${matrixFile}`,
  `NUB_SANDBOX_NODE=${node}`,
];
if (process.env.GITHUB_ENV) appendFileSync(process.env.GITHUB_ENV, `${lines.join('\n')}\n`);
else process.stdout.write(`${lines.join('\n')}\n`);
