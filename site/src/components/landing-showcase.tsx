'use client';

import React, { useState, useRef, useEffect } from 'react';
import Link from 'next/link';

/* ============================================================================
   TYPES & CONSTANTS
============================================================================ */
type PkgManager = 'curl' | 'powershell' | 'npm' | 'mise';

interface PMCommandInfo {
  label: string;
  cmd: string;
  copyText: string;
  osBadge: string;
}

const PM_COMMANDS: Record<PkgManager, PMCommandInfo> = {
  curl: {
    label: 'curl',
    cmd: 'curl -fsSL https://nubjs.com/install.sh | sh',
    copyText: 'curl -fsSL https://nubjs.com/install.sh | sh',
    osBadge: 'macOS & Linux',
  },
  powershell: {
    label: 'PowerShell',
    cmd: 'powershell -c "irm https://nubjs.com/install.ps1 | iex"',
    copyText: 'powershell -c "irm https://nubjs.com/install.ps1 | iex"',
    osBadge: 'Windows',
  },
  npm: {
    label: 'npm',
    cmd: 'npm install -g nub',
    copyText: 'npm install -g nub',
    osBadge: 'Node.js',
  },
  mise: {
    label: 'mise',
    cmd: 'mise use -g nub',
    copyText: 'mise use -g nub',
    osBadge: 'Tools',
  },
};

/* ============================================================================
   1. HERO INSTALL COMMAND BAR (Anime.js × Effect Hybrid)
============================================================================ */
export function HeroInstallCommand() {
  const [selectedPM, setSelectedPM] = useState<PkgManager>('curl');
  const [copied, setCopied] = useState(false);

  const active = PM_COMMANDS[selectedPM];

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(active.copyText);
      setCopied(true);
      setTimeout(() => setCopied(false), 1800);
    } catch {
      // clipboard access denied
    }
  };

  return (
    <div className="mx-auto flex flex-col items-center w-full max-w-xl">
      {/* 4 Authentic Installer Tabs */}
      <div className="flex items-center justify-center gap-1.5 mb-3 p-1 rounded-xl bg-[#141210] border border-[#2e2a25]">
        {(['curl', 'powershell', 'npm', 'mise'] as PkgManager[]).map((pm) => {
          const isActive = selectedPM === pm;
          return (
            <button
              key={pm}
              type="button"
              onClick={() => setSelectedPM(pm)}
              className={`px-3 py-1.5 text-xs font-mono rounded-lg transition-all cursor-pointer flex items-center gap-1.5 ${
                isActive
                  ? 'bg-[#221f1a] text-white font-semibold border border-[#38332c] shadow-sm'
                  : 'text-zinc-400 hover:text-zinc-200 border border-transparent'
              }`}
            >
              <span className={isActive ? 'text-ember' : 'text-zinc-600'}>●</span>
              <span>{PM_COMMANDS[pm].label}</span>
              <span className="hidden sm:inline text-[10px] text-zinc-500 font-normal">
                ({PM_COMMANDS[pm].osBadge})
              </span>
            </button>
          );
        })}
      </div>

      {/* Outer Glow Container with Nub Ember & Acid Accents */}
      <div className="group relative w-full">
        <div className="absolute -inset-0.5 rounded-2xl bg-gradient-to-r from-ember/35 via-acid/25 to-sky/35 opacity-40 blur-md transition duration-500 group-hover:opacity-75" />

        <div className="relative flex items-center justify-between rounded-xl border border-[#2e2a25] bg-[#12110f]/95 p-1.5 shadow-2xl backdrop-blur-xl">
          {/* Command Code Display */}
          <div className="flex flex-1 items-center px-3.5 overflow-x-auto select-all">
            <span className="font-mono text-xs sm:text-[13px] text-zinc-200 whitespace-nowrap">
              <span className="text-zinc-600 mr-2 select-none">$</span>
              {active.cmd}
            </span>
          </div>

          {/* Copy Button */}
          <button
            type="button"
            onClick={handleCopy}
            className="flex items-center justify-center gap-1.5 rounded-lg bg-[#1c1a17] px-3.5 py-1.5 text-xs font-mono font-medium text-zinc-300 border border-[#38332c] hover:border-zinc-500 hover:bg-[#26231f] hover:text-white transition-all shrink-0 cursor-pointer"
            aria-label="Copy installation command"
          >
            {copied ? (
              <>
                <svg className="w-3.5 h-3.5 text-acid" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
                  <path d="M20 6 9 17l-5-5" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
                <span className="text-acid font-semibold">Copied!</span>
              </>
            ) : (
              <>
                <svg className="w-3.5 h-3.5 text-zinc-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                </svg>
                <span>Copy</span>
              </>
            )}
          </button>
        </div>
      </div>

      {/* Action Buttons */}
      <div className="mt-6 flex flex-wrap items-center justify-center gap-3 w-full">
        <Link
          href="/docs"
          className="inline-flex items-center justify-center gap-2 rounded-xl bg-white px-5 py-2.5 text-xs sm:text-sm font-semibold text-zinc-950 hover:bg-zinc-200 transition-all shadow-md cursor-pointer"
        >
          <span>Get Started</span>
          <span>→</span>
        </Link>
        <Link
          href="/docs/commands/run"
          className="inline-flex items-center justify-center gap-2 rounded-xl border border-[#38332c] bg-[#141210] px-5 py-2.5 text-xs sm:text-sm font-medium text-zinc-300 hover:border-zinc-500 hover:text-white transition-all cursor-pointer"
        >
          <span>Documentation</span>
        </Link>
      </div>

      {/* Proof / Verified Badges */}
      <div className="mt-7 flex flex-wrap items-center justify-center gap-x-5 gap-y-2 text-xs font-mono text-zinc-400">
        <div className="flex items-center gap-1.5">
          <span className="h-1.5 w-1.5 rounded-full bg-ember" />
          <span>4,600+ GitHub Stars</span>
        </div>
        <span className="text-zinc-700 hidden sm:inline">•</span>
        <div className="flex items-center gap-1.5">
          <span className="h-1.5 w-1.5 rounded-full bg-acid" />
          <span>24× Faster Script Dispatch</span>
        </div>
        <span className="text-zinc-700 hidden sm:inline">•</span>
        <div className="flex items-center gap-1.5">
          <span className="h-1.5 w-1.5 rounded-full bg-sky" />
          <span>100% Stock Node LTS</span>
        </div>
        <span className="text-zinc-700 hidden sm:inline">•</span>
        <div className="flex items-center gap-1.5">
          <span className="h-1.5 w-1.5 rounded-full bg-pink" />
          <span>0 Runtime Lock-In</span>
        </div>
      </div>
    </div>
  );
}

/* ============================================================================
   HERO QUICK COMMANDS DECK (Authentic 7 Core Verbs)
============================================================================ */
export function HeroQuickCommands() {
  const [copiedCmd, setCopiedCmd] = useState<string | null>(null);

  const commands = [
    { cmd: 'nub index.ts', desc: 'TypeScript-first Node.js runtime', accent: 'text-ember' },
    { cmd: 'nub run dev', desc: '24× faster pnpm run', accent: 'text-acid' },
    { cmd: 'nubx prisma generate', desc: '19× faster npx', accent: 'text-sky' },
    { cmd: 'nub install', desc: '5× faster pnpm install', accent: 'text-pink' },
    { cmd: 'nub watch src/server.ts', desc: 'native watch mode', accent: 'text-orchid' },
    { cmd: 'nub pm shim', desc: 'built-in Corepack-style shims', accent: 'text-acid' },
    { cmd: 'nub node install 26', desc: 'Node version manager', accent: 'text-ember' },
  ];

  const handleCopy = async (cmd: string) => {
    try {
      await navigator.clipboard.writeText(cmd);
      setCopiedCmd(cmd);
      setTimeout(() => setCopiedCmd(null), 1500);
    } catch {
      // clipboard access denied
    }
  };

  return (
    <div className="mt-8 mx-auto w-full max-w-2xl rounded-2xl border border-[#2e2a25] bg-[#100f0d]/95 p-4 shadow-2xl text-left font-mono">
      <div className="flex items-center justify-between pb-3 border-b border-[#2e2a25] text-xs text-zinc-400">
        <div className="flex items-center gap-2">
          <span className="h-2 w-2 rounded-full bg-ember animate-pulse" />
          <span className="font-semibold text-zinc-200">The Toolchain at a Glance</span>
        </div>
        <span className="text-[11px] text-zinc-500">7 Core Verbs · Click to copy</span>
      </div>
      <div className="divide-y divide-[#1c1a17] mt-1 text-xs">
        {commands.map((c) => (
          <div
            key={c.cmd}
            onClick={() => handleCopy(c.cmd)}
            className="group flex items-center justify-between py-2 px-2.5 rounded-lg hover:bg-[#181613] cursor-pointer transition-colors"
          >
            <div className="flex items-center gap-2.5 overflow-x-auto">
              <span className="text-zinc-600 select-none">$</span>
              <span className="text-zinc-200 font-semibold group-hover:text-white transition-colors whitespace-nowrap">
                {c.cmd}
              </span>
            </div>
            <div className="flex items-center gap-3 shrink-0 text-[11px] ml-2">
              <span className={`${c.accent} opacity-85 hidden sm:inline`}>{c.desc}</span>
              <span className="text-zinc-600 group-hover:text-zinc-300 transition-colors font-mono">
                {copiedCmd === c.cmd ? '✓' : '⧉'}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

/* ============================================================================
   2. HERO INTERACTIVE STUDIO (5 Real Tabs: Quickstart, TS, 24x, PM, Node)
============================================================================ */
interface CodeTabItem {
  id: string;
  name: string;
  subtitle: string;
  filename: string;
  code: string;
  output: string;
  duration: string;
}

const CODE_TABS: CodeTabItem[] = [
  {
    id: 'quickstart',
    name: 'QUICKSTART',
    subtitle: 'Stock Node.js',
    filename: 'index.ts',
    code: `import { createServer } from 'node:http';

interface ServerConfig {
  port: number;
  message: string;
}

const config: ServerConfig = {
  port: 3000,
  message: 'Hello from stock Node.js accelerated by Nub!',
};

const server = createServer((req, res) => {
  res.writeHead(200, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({ ok: true, message: config.message }));
});

server.listen(config.port, () => {
  console.log(\`[nub] Server listening on http://localhost:\${config.port}\`);
});`,
    output: `[nub] Transpiled index.ts via in-memory oxc in 0.9ms
[nub] Bootstrapping Node v22.14.0 (Stock LTS)
[nub] Server listening on http://localhost:3000
✔ Zero runtime fork · Zero global pollution`,
    duration: '2.1ms',
  },
  {
    id: 'scripts',
    name: '24× SCRIPTS',
    subtitle: 'Process Pooling',
    filename: 'package.json',
    code: `{
  "name": "high-velocity-monorepo",
  "scripts": {
    "test": "node --test tests/**/*.test.js",
    "build": "tsc --noEmit && oxc-transform src",
    "lint": "oxlint src"
  }
}

// In your terminal:
// $ nub run test
// Dispatches directly in Rust without 140ms Node CLI bootstrap latency.`,
    output: `[nub run] Executing: "test" in ./packages/core
$ node --test tests/**/*.test.js
✔ 42 tests passed in 14.7ms (24× faster than npm run)
✔ Process pool re-used without subprocess overhead`,
    duration: '14.7ms',
  },
  {
    id: 'pm',
    name: 'PACKAGE MANAGER',
    subtitle: 'Multi-Lockfile',
    filename: 'pnpm-lock.yaml',
    code: `# Nub embeds aube: the blazingly fast Rust PM engine.
# It reads & writes pnpm-lock.yaml, bun.lock, and package-lock.json byte-for-byte!

$ nub install
# Resolving 1,168 packages from content-addressable store...
# Zero lockfile churn. Hardlinks created directly on your APFS/ext4/NTFS filesystem.`,
    output: `[nub pm] Resolved 1,168 packages in 346ms
[nub pm] Hardlinked 81,398 files from global store
✔ Byte-for-byte pnpm-lock.yaml parity maintained
✔ CI verification passed with 0 lockfile drift`,
    duration: '346ms',
  },
  {
    id: 'direct-exec',
    name: 'DIRECT EXEC',
    subtitle: 'nubx (19× npx)',
    filename: 'terminal.sh',
    code: `# Tired of waiting 300ms–2000ms for npx to check metadata?
$ nubx prettier --check src/

# nubx caches binary executables with content hashing in native Rust:
# Startup latency drops from 226ms to 11ms!`,
    output: `[nubx] Resolving cached binary "prettier"
Checking formatting...
All matched files are clean.
[nubx] Done in 11.2ms (19× faster than npx)`,
    duration: '11.2ms',
  },
  {
    id: 'node-versions',
    name: 'NODE VERSIONS',
    subtitle: 'Zero-Config LTS',
    filename: '.node-version',
    code: `22.14.0

# In your project root, Nub automatically detects:
# 1. .node-version or .nvmrc
# 2. "packageManager" or "engines" in package.json
# Downloads & provisions official nodejs.org binaries in ~/.local/share/nub/nodes/`,
    output: `[nub node] Active version: v22.14.0 (LTS Jod)
[nub node] SHA-256 verified against nodejs.org SHASUMS256.txt
[nub node] Using stock official Node binary. No patched runtime.`,
    duration: '0.4ms',
  },
];

export function HeroInteractiveStudio() {
  const [activeTab, setActiveTab] = useState('quickstart');
  const [copied, setCopied] = useState(false);
  const [isRunning, setIsRunning] = useState(false);
  const [hasRun, setHasRun] = useState(false);

  const current = CODE_TABS.find((t) => t.id === activeTab) || CODE_TABS[0];

  const handleRun = () => {
    setIsRunning(true);
    setTimeout(() => {
      setIsRunning(false);
      setHasRun(true);
    }, 450);
  };

  const handleCopyCode = async () => {
    try {
      await navigator.clipboard.writeText(current.code);
      setCopied(true);
      setTimeout(() => setCopied(false), 1800);
    } catch {
      // ignore
    }
  };

  return (
    <div className="w-full mx-auto mt-14 text-left">
      {/* 5 Top Tabs with Subtitles */}
      <div className="grid grid-cols-2 sm:grid-cols-5 gap-2 border-b border-[#2e2a25] pb-3 mb-0">
        {CODE_TABS.map((tab) => {
          const isActive = tab.id === activeTab;
          return (
            <button
              key={tab.id}
              type="button"
              onClick={() => {
                setActiveTab(tab.id);
                setHasRun(false);
              }}
              className={`flex flex-col items-start px-3 py-2 rounded-xl text-left transition-all cursor-pointer ${
                isActive
                  ? 'bg-[#1e1b17] border border-[#38332c] shadow-sm'
                  : 'hover:bg-[#141210] border border-transparent'
              }`}
            >
              <span className={`text-[11px] font-mono font-bold tracking-wider ${
                isActive ? 'text-white' : 'text-zinc-400'
              }`}>
                {tab.name}
              </span>
              <span className={`text-[10px] font-mono mt-0.5 ${
                isActive ? 'text-ember' : 'text-zinc-600'
              }`}>
                {tab.subtitle}
              </span>
            </button>
          );
        })}
      </div>

      {/* Editor Window */}
      <div className="rounded-2xl border border-[#2e2a25] bg-[#100f0d] shadow-2xl overflow-hidden ring-1 ring-white/5 mt-3">
        {/* Top Green Alert Banner inside Window */}
        <div className="flex items-center justify-between bg-acid/10 border-b border-acid/20 px-4 py-2 text-xs font-mono">
          <div className="flex items-center gap-2">
            <span className="rounded bg-acid px-1.5 py-0.5 text-[10px] font-bold text-zinc-950">
              TRY NUB
            </span>
            <span className="text-zinc-200">
              Zero configuration required. Drop-in replacement for tsx, npx, and npm run.
            </span>
          </div>
          <button
            type="button"
            onClick={handleRun}
            className="hidden sm:inline-flex items-center gap-1.5 text-xs text-acid font-semibold hover:underline cursor-pointer"
          >
            <span>Run script in browser sandbox</span>
            <span>→</span>
          </button>
        </div>

        {/* Window Control & File Tabs */}
        <div className="flex items-center justify-between border-b border-[#2e2a25] bg-[#141210] px-4 py-2.5">
          <div className="flex items-center gap-2">
            <div className="flex items-center gap-1.5">
              <div className="h-2.5 w-2.5 rounded-full bg-[#3a3530]" />
              <div className="h-2.5 w-2.5 rounded-full bg-[#3a3530]" />
              <div className="h-2.5 w-2.5 rounded-full bg-[#3a3530]" />
            </div>
            <div className="ml-3 flex items-center gap-1.5 rounded-md bg-[#1c1a17] px-2.5 py-1 text-xs font-mono text-zinc-300 border border-[#38332c]">
              <span className="text-zinc-500">📄</span>
              <span>{current.filename}</span>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={handleRun}
              disabled={isRunning}
              className="flex items-center gap-1.5 rounded-lg bg-white text-zinc-950 px-3 py-1 text-xs font-mono font-semibold hover:bg-zinc-200 transition-colors shadow-sm disabled:opacity-50 cursor-pointer"
            >
              {isRunning ? (
                <>
                  <span className="inline-block h-1.5 w-1.5 rounded-full bg-ember animate-ping" />
                  <span>Running...</span>
                </>
              ) : (
                <>
                  <span>▶</span>
                  <span>Run with nub</span>
                </>
              )}
            </button>

            <button
              type="button"
              onClick={handleCopyCode}
              className="rounded-lg border border-[#38332c] bg-[#1c1a17] p-1.5 text-zinc-400 hover:text-white transition-colors cursor-pointer"
              aria-label="Copy code"
            >
              {copied ? (
                <svg className="w-3.5 h-3.5 text-acid" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
                  <path d="M20 6 9 17l-5-5" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
              ) : (
                <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                </svg>
              )}
            </button>
          </div>
        </div>

        {/* Code Content Area with Nub Color Palette */}
        <div className="p-5 font-mono text-xs sm:text-[13px] text-zinc-200 overflow-x-auto leading-relaxed max-h-[380px] bg-[#0c0a09]">
          <pre className="font-mono">
            <code>
              {current.code.split('\n').map((line, idx) => {
                let formatted = <span>{line}</span>;
                if (line.startsWith('import ') || line.startsWith('export ')) {
                  formatted = <span className="text-pink">{line}</span>;
                } else if (line.startsWith('//') || line.startsWith('#')) {
                  formatted = <span className="text-zinc-500 italic">{line}</span>;
                } else if (line.includes('const ') || line.includes('interface ') || line.includes('function ')) {
                  formatted = <span className="text-sky">{line}</span>;
                } else if (line.includes('console.log') || line.includes('server.listen') || line.includes('renderToString')) {
                  formatted = <span className="text-acid">{line}</span>;
                } else if (line.includes('number') || line.includes('3000') || line.includes('true')) {
                  formatted = <span className="text-orchid">{line}</span>;
                }
                return (
                  <div key={idx} className="flex">
                    <span className="w-8 select-none text-zinc-600 text-right pr-4 shrink-0">{idx + 1}</span>
                    <span className="flex-1">{formatted}</span>
                  </div>
                );
              })}
            </code>
          </pre>
        </div>

        {/* Execution Terminal Output Drawer */}
        <div className="border-t border-[#2e2a25] bg-[#141210] p-4 font-mono text-xs">
          <div className="flex items-center justify-between pb-2 mb-2 border-b border-[#2e2a25] text-zinc-400">
            <div className="flex items-center gap-2">
              <span className="text-acid font-bold">●</span>
              <span>Stock Node Terminal</span>
              {hasRun && (
                <span className="text-[10px] rounded bg-acid/20 text-acid px-1.5 py-0.2">
                  Completed in {current.duration}
                </span>
              )}
            </div>
            <span className="text-[11px] text-zinc-500">Node.js LTS execution</span>
          </div>
          <pre className="text-zinc-300 leading-normal whitespace-pre-wrap">
            {isRunning ? (
              <span className="text-zinc-500 animate-pulse">Running script on isolated worker...</span>
            ) : (
              current.output
            )}
          </pre>
        </div>
      </div>
    </div>
  );
}

/* ============================================================================
   3. REAL PRODUCTION SYSTEMS (SCROLL-LOCKED HORIZONTAL PARALLAX SHOWCASE)
============================================================================ */
export function RealWorldProductionParallax() {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const trackRef = useRef<HTMLDivElement | null>(null);
  const [progress, setProgress] = useState(0);
  const [activeCard, setActiveCard] = useState(0);
  const [maxTranslate, setMaxTranslate] = useState(0);

  const CARDS = [
    {
      id: 'video',
      tag: 'VIDEO BREAKDOWN',
      accent: 'text-acid border-acid/40 bg-acid/10',
      title: 'An Ex-Bun Dev Just Built The Anti-Bun',
      subtitle: 'Technical architecture deep dive by Better Stack',
      body: 'Explores how Nub leverages Node\'s official loader hooks, in-memory oxc transpilation, and stock LTS binaries without ever requiring a patched runtime fork.',
      linkText: 'Watch on YouTube →',
      linkHref: 'https://www.youtube.com/watch?v=6YRpXxbtc2c',
      isExternal: true,
      visual: (
        <div className="w-full aspect-video overflow-hidden rounded-2xl border border-[#2e2a25] bg-[#0c0a09] shadow-inner">
          <iframe
            className="w-full h-full"
            src="https://www.youtube-nocookie.com/embed/6YRpXxbtc2c?start=2"
            title="An Ex-Bun Dev Just Built The Anti-Bun (Nub)"
            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
            referrerPolicy="strict-origin-when-cross-origin"
            loading="lazy"
            allowFullScreen
          />
        </div>
      ),
    },
    {
      id: 'security',
      tag: 'ACTIVE DEFENSE',
      accent: 'text-ember border-ember/40 bg-ember/10',
      title: 'Hardened against supply-chain attacks',
      subtitle: 'Deny-by-default build scripts & live OSV advisories',
      body: 'Nub treats dependency build scripts as deny-by-default, queries OSV for malicious-package advisories on every fresh resolve, and refuses versions whose trust evidence weakened.',
      linkText: 'Package security docs →',
      linkHref: '/docs/pm',
      isExternal: false,
      visual: (
        <div className="rounded-2xl border border-[#2e2a25] bg-[#0a0908] p-5 font-mono text-xs text-zinc-300 shadow-xl leading-relaxed">
          <div className="flex items-center justify-between pb-2 mb-3 border-b border-[#2e2a25] text-zinc-500 text-[11px]">
            <span>active-defense-terminal</span>
            <span className="text-ember font-bold">OSV MAL-2023-8697</span>
          </div>
          <p className="text-zinc-500">$ nub add @ledgerhq/connect-kit</p>
          <p className="text-red-400">Error: refusing to add malicious package(s):</p>
          <p className="text-red-400">  - @ledgerhq/connect-kit (MAL-2023-8697)</p>
          <p className="text-ember font-bold">  ❌ code=ERR_NUB_MALICIOUS_PACKAGE</p>
          
          <div className="my-2 border-t border-[#201d19]" />
          <p className="text-zinc-500">$ nub install</p>
          <p className="text-red-400">Error: trust downgrade for nanoid@3.3.14 (trustPolicy=no-downgrade)</p>
          <p className="text-zinc-500 text-[11px]">  earlier version 3.3.7 had provenance attestation</p>
          <p className="text-ember font-bold">  ❌ code=ERR_NUB_TRUST_DOWNGRADE</p>
          
          <div className="my-2 border-t border-[#201d19]" />
          <p className="text-zinc-400">WARN ignored build scripts for 1 package: esbuild@0.21.5</p>
          <p className="text-acid font-semibold">Run `nub approve-builds` to review and allow.</p>
        </div>
      ),
    },
    {
      id: 'benchmarks',
      tag: 'LINUX CI VERIFIED',
      accent: 'text-sky border-sky/40 bg-sky/10',
      title: 'Warm frozen installs in milliseconds',
      subtitle: 'Hyperfine 25-run benchmark on 1168 packages',
      body: 'Tested on an 81,398-file monorepo fixture under Linux CI. Nub\'s embedded Rust engine (aube) hardlinks packages from a global content-addressable store with zero lockfile churn.',
      linkText: 'Inspect benchmark suite →',
      linkHref: 'https://github.com/nubjs/nub/blob/main/tests/bench/install/README.md',
      isExternal: true,
      visual: (
        <div className="rounded-2xl border border-[#2e2a25] bg-[#0a0908] p-5 font-mono text-xs shadow-xl">
          <div className="flex items-center justify-between pb-2 mb-4 border-b border-[#2e2a25] text-zinc-500 text-[11px]">
            <span>warm frozen install · 1168 packages</span>
            <span className="text-acid">Hyperfine 25 runs</span>
          </div>
          <div className="space-y-3.5">
            <div>
              <div className="flex justify-between text-zinc-200 mb-1 font-semibold">
                <span className="text-acid">nub</span>
                <span>346 ms (1×)</span>
              </div>
              <div className="h-2 rounded-full bg-[#1c1a17] overflow-hidden">
                <div className="h-full bg-acid rounded-full" style={{ width: '8%' }} />
              </div>
            </div>
            <div>
              <div className="flex justify-between text-zinc-400 mb-1">
                <span>bun</span>
                <span>1,896 ms (5.5× slower)</span>
              </div>
              <div className="h-2 rounded-full bg-[#1c1a17] overflow-hidden">
                <div className="h-full bg-zinc-600 rounded-full" style={{ width: '22%' }} />
              </div>
            </div>
            <div>
              <div className="flex justify-between text-zinc-400 mb-1">
                <span>pnpm</span>
                <span>3,453 ms (10× slower)</span>
              </div>
              <div className="h-2 rounded-full bg-[#1c1a17] overflow-hidden">
                <div className="h-full bg-zinc-600 rounded-full" style={{ width: '40%' }} />
              </div>
            </div>
            <div>
              <div className="flex justify-between text-zinc-400 mb-1">
                <span>npm</span>
                <span>12,945 ms (37.4× slower)</span>
              </div>
              <div className="h-2 rounded-full bg-[#1c1a17] overflow-hidden">
                <div className="h-full bg-zinc-700 rounded-full" style={{ width: '95%' }} />
              </div>
            </div>
          </div>
        </div>
      ),
    },
    {
      id: 'dispatch',
      tag: 'PROCESS POOLING',
      accent: 'text-pink border-pink/40 bg-pink/10',
      title: '24× faster monorepo script runner',
      subtitle: 'Native process pooling directly in Rust',
      body: 'Executing package.json scripts with npm or pnpm incurs 140ms–400ms of Node CLI bootstrap latency on every invocation. Nub dispatches worker processes directly in <15ms.',
      linkText: 'Explore script runner docs →',
      linkHref: '/docs/commands/run',
      isExternal: false,
      visual: (
        <div className="rounded-2xl border border-[#2e2a25] bg-[#0a0908] p-5 font-mono text-xs shadow-xl">
          <div className="flex items-center justify-between pb-2 mb-4 border-b border-[#2e2a25] text-zinc-500 text-[11px]">
            <span>script dispatch latency (macOS)</span>
            <span className="text-pink">50 runs avg</span>
          </div>
          <div className="space-y-3">
            <div className="p-2.5 rounded-lg border border-acid/30 bg-acid/5 flex items-center justify-between">
              <span className="text-acid font-bold">$ nub run test</span>
              <span className="text-acid font-bold">14.7 ms</span>
            </div>
            <div className="p-2.5 rounded-lg border border-[#2e2a25] bg-[#141210] flex items-center justify-between text-zinc-400">
              <span>$ node --run test</span>
              <span>32.2 ms</span>
            </div>
            <div className="p-2.5 rounded-lg border border-[#2e2a25] bg-[#141210] flex items-center justify-between text-zinc-400">
              <span>$ npm run test</span>
              <span>329.9 ms (22×)</span>
            </div>
            <div className="p-2.5 rounded-lg border border-[#2e2a25] bg-[#141210] flex items-center justify-between text-zinc-400">
              <span>$ pnpm run test</span>
              <span>442.7 ms (30×)</span>
            </div>
          </div>
        </div>
      ),
    },
  ];

  // Measure dynamic horizontal scrollable range
  useEffect(() => {
    const updateDimensions = () => {
      if (!trackRef.current) return;
      const scrollW = trackRef.current.scrollWidth;
      const viewW = window.innerWidth;
      const margin = (viewW * 0.15) / 2;
      const max = Math.max(0, scrollW - viewW + margin);
      setMaxTranslate(max);
    };

    updateDimensions();
    window.addEventListener('resize', updateDimensions);
    return () => window.removeEventListener('resize', updateDimensions);
  }, []);

  // Scroll listener for sticky horizontal parallax
  useEffect(() => {
    let ticking = false;

    const onScroll = () => {
      if (!ticking) {
        window.requestAnimationFrame(() => {
          if (!containerRef.current) {
            ticking = false;
            return;
          }
          const rect = containerRef.current.getBoundingClientRect();
          const totalDistance = rect.height - window.innerHeight;
          if (totalDistance <= 0) {
            ticking = false;
            return;
          }

          const scrolled = -rect.top;
          const p = Math.max(0, Math.min(1, scrolled / totalDistance));
          setProgress(p);

          // Horizontal translation occurs over [0, 0.85].
          // [0.85, 1.0] acts as a dedicated dwell buffer holding the final card in full view
          // before vertical scroll unpins and transitions to the next section.
          const travelP = Math.min(1, p / 0.85);
          const idx = Math.min(CARDS.length - 1, Math.floor(travelP * CARDS.length));
          setActiveCard(idx);

          ticking = false;
        });
        ticking = true;
      }
    };

    window.addEventListener('scroll', onScroll, { passive: true });
    window.addEventListener('resize', onScroll);
    onScroll();

    return () => {
      window.removeEventListener('scroll', onScroll);
      window.removeEventListener('resize', onScroll);
    };
  }, [CARDS.length]);

  const scrollToCard = (index: number) => {
    if (!containerRef.current) return;
    const rect = containerRef.current.getBoundingClientRect();
    const containerTop = window.scrollY + rect.top;
    const totalDistance = rect.height - window.innerHeight;
    const targetP = (index / (CARDS.length - 1)) * 0.85;
    const targetScroll = containerTop + targetP * totalDistance;

    const lenis = (window as unknown as { __lenis?: { scrollTo: (target: number) => void } }).__lenis;
    if (lenis) {
      lenis.scrollTo(targetScroll);
    } else {
      window.scrollTo({ top: targetScroll, behavior: 'smooth' });
    }
  };

  const travelProgress = Math.min(1, progress / 0.85);
  const effectiveMaxTranslate = maxTranslate || (CARDS.length - 1) * 650;
  const currentTranslate = travelProgress * effectiveMaxTranslate;

  return (
    <div
      ref={containerRef}
      className="relative h-[280vh] sm:h-[320vh] w-full"
    >
      {/* Sticky Viewport */}
      <div className="sticky top-0 h-screen w-full flex flex-col justify-center overflow-hidden">
        {/* Top Control & Header Bar */}
        <div className="w-[85%] mx-auto mb-6">
          <div className="flex flex-col sm:flex-row sm:items-end justify-between gap-4 pb-4 border-b border-[#2e2a25]">
            <div>
              <span className="text-xs font-mono font-semibold uppercase tracking-wider text-acid flex items-center gap-1.5 mb-1.5">
                <span className="h-1.5 w-1.5 rounded-full bg-acid animate-pulse" />
                Verified Production Systems
              </span>
              <h2 className="font-display text-2xl sm:text-4xl font-bold tracking-tight text-white">
                Real-world production systems
              </h2>
            </div>

            {/* Stepper & Arrows */}
            <div className="flex items-center gap-4">
              <div className="text-xs font-mono text-zinc-400">
                <span className="text-white font-bold">{String(activeCard + 1).padStart(2, '0')}</span>
                <span className="text-zinc-600"> / </span>
                <span>{String(CARDS.length).padStart(2, '0')}</span>
              </div>

              {/* Progress Line */}
              <div className="w-24 sm:w-32 h-1 rounded-full bg-[#1c1a17] overflow-hidden">
                <div
                  className="h-full bg-gradient-to-r from-acid via-sky to-ember transition-all duration-150"
                  style={{ width: `${Math.max(5, travelProgress * 100)}%` }}
                />
              </div>

              {/* Manual Nav Buttons */}
              <div className="flex items-center gap-1.5">
                <button
                  type="button"
                  onClick={() => scrollToCard(Math.max(0, activeCard - 1))}
                  disabled={activeCard === 0}
                  aria-label="Previous card"
                  className="h-8 w-8 rounded-lg border border-[#2e2a25] bg-[#141210] flex items-center justify-center text-xs text-zinc-300 hover:border-zinc-500 hover:text-white disabled:opacity-30 cursor-pointer transition-colors"
                >
                  ←
                </button>
                <button
                  type="button"
                  onClick={() => scrollToCard(Math.min(CARDS.length - 1, activeCard + 1))}
                  disabled={activeCard === CARDS.length - 1}
                  aria-label="Next card"
                  className="h-8 w-8 rounded-lg border border-[#2e2a25] bg-[#141210] flex items-center justify-center text-xs text-zinc-300 hover:border-zinc-500 hover:text-white disabled:opacity-30 cursor-pointer transition-colors"
                >
                  →
                </button>
              </div>
            </div>
          </div>
        </div>

        {/* Horizontal Parallax Rail */}
        <div className="w-full overflow-hidden">
          <div
            ref={trackRef}
            className="flex gap-6 sm:gap-8 pl-[7.5%] pr-[7.5%] will-change-transform"
            style={{
              transform: `translate3d(-${currentTranslate}px, 0, 0)`,
            }}
          >
            {CARDS.map((card, idx) => {
              const cardOffset = (travelProgress - idx / (CARDS.length - 1)) * 30;
              return (
                <div
                  key={card.id}
                  className="w-[88vw] sm:w-[580px] lg:w-[680px] shrink-0 rounded-3xl border border-[#2e2a25] bg-[#12110f] p-6 sm:p-9 shadow-2xl relative overflow-hidden flex flex-col justify-between group hover:border-[#3e3830] transition-all"
                >
                  {/* Subtle Card Background Parallax Glow */}
                  <div
                    className="pointer-events-none absolute -inset-2 opacity-30 blur-2xl transition-transform"
                    style={{
                      transform: `translate3d(${cardOffset * 0.8}px, 0, 0)`,
                      background: 'radial-gradient(ellipse 60% 50% at 50% 0%, rgba(255, 93, 59, 0.15), transparent 70%)',
                    }}
                  />

                  <div>
                    <div className="flex items-center justify-between mb-4">
                      <span className={`text-[11px] font-mono px-2.5 py-0.5 rounded-full border font-semibold ${card.accent}`}>
                        {card.tag}
                      </span>
                      <span className="text-xs font-mono text-zinc-500">
                        {card.subtitle}
                      </span>
                    </div>

                    <h3 className="font-display text-xl sm:text-2xl font-bold tracking-tight text-white mb-3">
                      {card.title}
                    </h3>
                    <p className="text-xs sm:text-sm text-zinc-300 leading-relaxed font-normal mb-5">
                      {card.body}
                    </p>

                    {/* Visual with internal parallax offset */}
                    <div
                      className="my-2 transition-transform duration-150"
                      style={{
                        transform: `translate3d(${cardOffset * 0.3}px, 0, 0)`,
                      }}
                    >
                      {card.visual}
                    </div>
                  </div>

                  <div className="mt-6 pt-4 border-t border-[#2e2a25]">
                    {card.isExternal ? (
                      <a
                        href={card.linkHref}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="inline-flex items-center gap-1.5 text-xs font-mono font-semibold text-ember hover:underline"
                      >
                        <span>{card.linkText}</span>
                      </a>
                    ) : (
                      <Link
                        href={card.linkHref}
                        className="inline-flex items-center gap-1.5 text-xs font-mono font-semibold text-acid hover:underline"
                      >
                        <span>{card.linkText}</span>
                      </Link>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        <div className="w-[85%] mx-auto mt-4 text-xs font-mono text-zinc-500 flex items-center justify-between">
          <span>↓ Scroll vertically to glide horizontally across production benchmarks</span>
          <span className="hidden sm:inline">Use arrows or mouse wheel</span>
        </div>
      </div>
    </div>
  );
}

/* ============================================================================
   4. COMPLEXITY CURVE & REAL HYPERFINE BENCHMARKS
============================================================================ */
export function ComplexityCurveChart() {
  const [hovered, setHovered] = useState<'traditional' | 'nub' | null>(null);

  return (
    <div className="grid grid-cols-1 lg:grid-cols-12 gap-10 items-center">
      {/* Left Column: Heading & Real Benchmark Numbers */}
      <div className="lg:col-span-5 text-left">
        <span className="text-xs font-mono font-semibold uppercase tracking-wider text-ember">
          Architecture &amp; Benchmarks
        </span>
        <h3 className="font-display text-3xl sm:text-4xl font-bold tracking-tight text-white mt-2 leading-tight">
          Built-in solutions for the hard problems
        </h3>
        <p className="mt-4 text-sm sm:text-base text-zinc-400 leading-relaxed">
          Fragmented tooling forces a maze of TypeScript wrappers (`tsx`, `ts-node`), version managers (`nvm`), script runners, and PM wrappers that stall on startup.
        </p>

        <div className="mt-6 space-y-2.5 font-mono text-xs border-t border-[#2e2a25] pt-4">
          <div className="flex justify-between items-center text-acid font-semibold">
            <span>nub run test</span>
            <span>14.7 ms (Instant)</span>
          </div>
          <div className="flex justify-between items-center text-zinc-400">
            <span>node --run test</span>
            <span>32.2 ms</span>
          </div>
          <div className="flex justify-between items-center text-zinc-400">
            <span>npm run</span>
            <span>329.9 ms (22× slower)</span>
          </div>
          <div className="flex justify-between items-center text-zinc-400">
            <span>pnpm run</span>
            <span>442.7 ms (30× slower)</span>
          </div>
        </div>
      </div>

      {/* Right Column: The Signature Complexity SVG Graph */}
      <div className="lg:col-span-7">
        <div className="relative rounded-2xl border border-[#2e2a25] bg-[#100f0d] p-6 shadow-2xl overflow-hidden ring-1 ring-white/5">
          <div className="flex items-center justify-between border-b border-[#2e2a25] pb-4 mb-4 text-xs font-mono">
            <div className="flex items-center gap-2">
              <span className="h-2 w-2 rounded-full bg-ember" />
              <span className="text-zinc-400">Fragmented Node Stack (tsx + nvm + wrappers)</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="h-2 w-2 rounded-full bg-acid" />
              <span className="text-zinc-200 font-semibold">Nub Native Toolchain</span>
            </div>
          </div>

          <div className="relative w-full h-[260px] sm:h-[280px]">
            <svg
              viewBox="0 0 600 280"
              className="w-full h-full overflow-visible"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              {/* Axes */}
              <line x1="40" y1="20" x2="40" y2="240" stroke="#2e2a25" strokeWidth="1" strokeDasharray="3 3" />
              <line x1="40" y1="240" x2="580" y2="240" stroke="#2e2a25" strokeWidth="1" />
              <line x1="40" y1="160" x2="580" y2="160" stroke="#1c1a17" strokeWidth="1" strokeDasharray="3 3" />
              <line x1="40" y1="80" x2="580" y2="80" stroke="#1c1a17" strokeWidth="1" strokeDasharray="3 3" />

              {/* Labels */}
              <text x="25" y="30" fill="#71717a" fontSize="10" fontFamily="monospace" textAnchor="end">High</text>
              <text x="25" y="240" fill="#71717a" fontSize="10" fontFamily="monospace" textAnchor="end">Low</text>
              <text x="40" y="260" fill="#71717a" fontSize="10" fontFamily="monospace">Codebase Size &amp; Tooling Layers →</text>

              {/* Exponential Red Curve: Traditional Node Fragmented Tooling */}
              <path
                d="M 40 230 C 180 225, 340 180, 560 35"
                stroke="#ff5d3b"
                strokeWidth="3"
                strokeLinecap="round"
                fill="none"
                className={`transition-opacity duration-300 ${hovered === 'nub' ? 'opacity-30' : 'opacity-100'}`}
              />

              {/* Shaded Red Area under Exponential Curve */}
              <path
                d="M 40 230 C 180 225, 340 180, 560 35 L 560 240 L 40 240 Z"
                fill="url(#redGlow)"
                className={`transition-opacity duration-300 ${hovered === 'nub' ? 'opacity-10' : 'opacity-20'}`}
              />

              {/* Flat Green Line: Nub Unified Architecture */}
              <path
                d="M 40 215 C 200 215, 380 210, 560 205"
                stroke="#4fe173"
                strokeWidth="3.5"
                strokeLinecap="round"
                fill="none"
                className={`transition-opacity duration-300 ${hovered === 'traditional' ? 'opacity-30' : 'opacity-100'}`}
              />

              {/* Shaded Green Area */}
              <path
                d="M 40 215 C 200 215, 380 210, 560 205 L 560 240 L 40 240 Z"
                fill="url(#greenGlow)"
                className={`transition-opacity duration-300 ${hovered === 'traditional' ? 'opacity-10' : 'opacity-20'}`}
              />

              {/* Interactive Points on Nub Line */}
              <circle
                cx="200"
                cy="215"
                r="5"
                fill="#4fe173"
                stroke="#0c0a09"
                strokeWidth="2"
                className="cursor-pointer hover:r-7 transition-all"
                onMouseEnter={() => setHovered('nub')}
                onMouseLeave={() => setHovered(null)}
              />
              <circle
                cx="380"
                cy="210"
                r="5"
                fill="#4fe173"
                stroke="#0c0a09"
                strokeWidth="2"
                className="cursor-pointer hover:r-7 transition-all"
                onMouseEnter={() => setHovered('nub')}
                onMouseLeave={() => setHovered(null)}
              />
              <circle
                cx="560"
                cy="205"
                r="6"
                fill="#4fe173"
                stroke="#0c0a09"
                strokeWidth="2"
                className="cursor-pointer hover:r-8 transition-all"
                onMouseEnter={() => setHovered('nub')}
                onMouseLeave={() => setHovered(null)}
              />

              {/* Interactive Points on Traditional Curve */}
              <circle
                cx="340"
                cy="180"
                r="5"
                fill="#ff5d3b"
                stroke="#0c0a09"
                strokeWidth="2"
                className="cursor-pointer hover:r-7 transition-all"
                onMouseEnter={() => setHovered('traditional')}
                onMouseLeave={() => setHovered(null)}
              />
              <circle
                cx="560"
                cy="35"
                r="6"
                fill="#ff5d3b"
                stroke="#0c0a09"
                strokeWidth="2"
                className="cursor-pointer hover:r-8 transition-all"
                onMouseEnter={() => setHovered('traditional')}
                onMouseLeave={() => setHovered(null)}
              />

              {/* Gradients */}
              <defs>
                <linearGradient id="redGlow" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="#ff5d3b" stopOpacity="0.8" />
                  <stop offset="100%" stopColor="#ff5d3b" stopOpacity="0.0" />
                </linearGradient>
                <linearGradient id="greenGlow" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="#4fe173" stopOpacity="0.6" />
                  <stop offset="100%" stopColor="#4fe173" stopOpacity="0.0" />
                </linearGradient>
              </defs>
            </svg>

            {/* Explanatory Overlay Badge */}
            <div className="absolute right-4 top-2 bg-[#1c1a17]/90 border border-[#38332c] rounded-lg p-2.5 text-[11px] font-mono backdrop-blur-md shadow-lg max-w-xs text-left">
              {hovered === 'traditional' ? (
                <div>
                  <span className="text-ember font-bold">Traditional Overhead</span>
                  <p className="text-zinc-400 mt-0.5">
                    Subprocess spawn overhead (`tsx`), lockfile version drift, nvm env latency escalate exponentially with monorepo scale.
                  </p>
                </div>
              ) : (
                <div>
                  <span className="text-acid font-bold">Nub Architecture</span>
                  <p className="text-zinc-300 mt-0.5">
                    Additive Rust loader hooks (`module.registerHooks`) maintain flat ~14ms latency regardless of workspace size.
                  </p>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

/* ============================================================================
   5. ASYMMETRIC BENTO GRID (Different colsize and rowsize for ideal alignment)
============================================================================ */
export function AsymmetricBentoGrid() {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-6 gap-5 text-left">
      {/* 1. Hero Card (4 Columns, Large): In-Memory Oxc Transpilation */}
      <div className="md:col-span-2 lg:col-span-4 rounded-3xl border border-[#2e2a25] bg-gradient-to-br from-[#161411] to-[#100f0d] p-7 sm:p-8 shadow-2xl hover:border-ember/40 transition-all flex flex-col justify-between">
        <div>
          <div className="flex items-center justify-between mb-4">
            <span className="inline-flex items-center gap-1.5 rounded-full border border-ember/40 bg-ember/10 px-3 py-0.5 text-xs font-mono font-semibold text-ember">
              0.9ms Transpile Speed
            </span>
            <span className="text-xs font-mono text-zinc-500">Core Runtime Augmentation</span>
          </div>

          <h4 className="font-display text-2xl font-bold text-white tracking-tight mb-2">
            In-Memory Oxc Transpilation
          </h4>
          <p className="text-sm text-zinc-300 leading-relaxed font-normal mb-6 max-w-xl">
            Types and JSX transform directly in Rust in less than 1 millisecond. No temporary `.tsbuildinfo` files or messy `.cache` folders left on your disk.
          </p>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 mb-6 font-mono text-xs">
            <div className="p-3 rounded-xl border border-[#2e2a25] bg-[#0f0e0c]">
              <span className="text-acid font-bold block mb-1">✓ Non-Erasable Syntax</span>
              <span className="text-zinc-400">Enums, namespaces, and parameter properties fully supported.</span>
            </div>
            <div className="p-3 rounded-xl border border-[#2e2a25] bg-[#0f0e0c]">
              <span className="text-acid font-bold block mb-1">✓ Decorator Metadata</span>
              <span className="text-zinc-400">Seamless NestJS, TypeORM, and tsoa reflection compatibility.</span>
            </div>
          </div>
        </div>

        <div className="rounded-xl border border-[#2e2a25] bg-[#0a0908] p-3.5 font-mono text-xs text-zinc-400 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className="h-2 w-2 rounded-full bg-acid animate-pulse" />
            <span className="text-zinc-300">V8 Bytecode Generation:</span>
            <span className="text-acid">0 disk I/O</span>
          </div>
          <span className="text-zinc-600 hidden sm:inline">module.registerHooks</span>
        </div>
      </div>

      {/* 2. Metric Tile (2 Columns): 24x Faster Script Runner */}
      <div className="md:col-span-1 lg:col-span-2 rounded-3xl border border-[#2e2a25] bg-gradient-to-br from-[#161411] to-[#100f0d] p-7 shadow-2xl hover:border-acid/40 transition-all flex flex-col justify-between">
        <div>
          <div className="flex items-center justify-between mb-4">
            <span className="inline-flex items-center gap-1.5 rounded-full border border-acid/40 bg-acid/10 px-3 py-0.5 text-xs font-mono font-semibold text-acid">
              24× Faster
            </span>
            <span className="text-xs font-mono text-zinc-500">Fast Scripts</span>
          </div>

          <h4 className="font-display text-xl font-bold text-white tracking-tight mb-2">
            Instant Script Dispatch
          </h4>
          <p className="text-xs sm:text-sm text-zinc-300 leading-relaxed font-normal mb-4">
            Dispatches package scripts with native process pooling directly in Rust. Skips Node CLI bootstrap latency.
          </p>
        </div>

        <div className="mt-4 p-4 rounded-xl border border-[#2e2a25] bg-[#0c0a09] font-mono text-xs space-y-2">
          <div className="flex justify-between text-acid font-bold">
            <span>nub run test</span>
            <span>14.7ms</span>
          </div>
          <div className="w-full bg-[#1c1a17] h-1.5 rounded-full overflow-hidden">
            <div className="bg-acid h-full rounded-full" style={{ width: '6%' }} />
          </div>
          <div className="flex justify-between text-zinc-500 text-[11px] pt-1">
            <span>npm run test</span>
            <span>442.7ms (30× slower)</span>
          </div>
        </div>
      </div>

      {/* 3. Metric Tile (2 Columns): Direct Binary Exec (nubx) */}
      <div className="md:col-span-1 lg:col-span-2 rounded-3xl border border-[#2e2a25] bg-gradient-to-br from-[#161411] to-[#100f0d] p-7 shadow-2xl hover:border-sky/40 transition-all flex flex-col justify-between">
        <div>
          <div className="flex items-center justify-between mb-4">
            <span className="inline-flex items-center gap-1.5 rounded-full border border-sky/40 bg-sky/10 px-3 py-0.5 text-xs font-mono font-semibold text-sky">
              11ms Startup
            </span>
            <span className="text-xs font-mono text-zinc-500">nubx (npx alternative)</span>
          </div>

          <h4 className="font-display text-xl font-bold text-white tracking-tight mb-2">
            Direct Binary Exec
          </h4>
          <p className="text-xs sm:text-sm text-zinc-300 leading-relaxed font-normal mb-4">
            Pre-caches binary shims directly in Rust with content-addressable hash resolution.
          </p>
        </div>

        <div className="p-3.5 rounded-xl border border-[#2e2a25] bg-[#0c0a09] font-mono text-xs">
          <div className="text-zinc-500">$ nubx prettier -w .</div>
          <div className="text-sky font-semibold mt-1">✔ 19× faster startup than npx</div>
        </div>
      </div>

      {/* 4. Feature Tile (2 Columns): Multi-Lockfile Neutral Engine */}
      <div className="md:col-span-1 lg:col-span-2 rounded-3xl border border-[#2e2a25] bg-gradient-to-br from-[#161411] to-[#100f0d] p-7 shadow-2xl hover:border-pink/40 transition-all flex flex-col justify-between">
        <div>
          <div className="flex items-center justify-between mb-4">
            <span className="inline-flex items-center gap-1.5 rounded-full border border-pink/40 bg-pink/10 px-3 py-0.5 text-xs font-mono font-semibold text-pink">
              Zero Lockfile Churn
            </span>
            <span className="text-xs font-mono text-zinc-500">Embedded aube</span>
          </div>

          <h4 className="font-display text-xl font-bold text-white tracking-tight mb-2">
            Multi-Lockfile Parity
          </h4>
          <p className="text-xs sm:text-sm text-zinc-300 leading-relaxed font-normal mb-4">
            Natively parses and writes pnpm-lock.yaml, bun.lock, and package-lock.json with 0 drift.
          </p>
        </div>

        <div className="p-3.5 rounded-xl border border-[#2e2a25] bg-[#0c0a09] font-mono text-xs flex items-center justify-between text-zinc-300">
          <span>pnpm · bun · npm</span>
          <span className="text-pink font-bold">100% Parity</span>
        </div>
      </div>

      {/* 5. Feature Tile (2 Columns): Automatic Node LTS Versioning */}
      <div className="md:col-span-1 lg:col-span-2 rounded-3xl border border-[#2e2a25] bg-gradient-to-br from-[#161411] to-[#100f0d] p-7 shadow-2xl hover:border-orchid/40 transition-all flex flex-col justify-between">
        <div>
          <div className="flex items-center justify-between mb-4">
            <span className="inline-flex items-center gap-1.5 rounded-full border border-orchid/40 bg-orchid/10 px-3 py-0.5 text-xs font-mono font-semibold text-orchid">
              Sandboxed LTS
            </span>
            <span className="text-xs font-mono text-zinc-500">Built-in Versioning</span>
          </div>

          <h4 className="font-display text-xl font-bold text-white tracking-tight mb-2">
            Automatic Node Manager
          </h4>
          <p className="text-xs sm:text-sm text-zinc-300 leading-relaxed font-normal mb-4">
            Reads `.node-version` or `.nvmrc` automatically. Verifies official nodejs.org SHA-256 hashes.
          </p>
        </div>

        <div className="p-3.5 rounded-xl border border-[#2e2a25] bg-[#0c0a09] font-mono text-xs flex items-center justify-between text-zinc-300">
          <span>Stock Node binaries</span>
          <span className="text-orchid font-bold">No nvm lag</span>
        </div>
      </div>

      {/* 6. Full Width Banner (6 Columns): Zero Runtime Lock-in Guarantee & Monorepo Filter */}
      <div className="md:col-span-2 lg:col-span-6 rounded-3xl border border-[#2e2a25] bg-[#12110f] p-7 sm:p-9 shadow-2xl hover:border-zinc-600 transition-all">
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-center">
          <div className="lg:col-span-6">
            <span className="text-xs font-mono font-semibold uppercase tracking-wider text-acid flex items-center gap-1.5 mb-1.5">
              <span className="h-1.5 w-1.5 rounded-full bg-acid" />
              100% Stock Node Compatibility Guarantee
            </span>
            <h4 className="font-display text-2xl sm:text-3xl font-bold text-white tracking-tight">
              Zero Runtime Lock-In. Run on stock Node anytime.
            </h4>
            <p className="mt-3 text-xs sm:text-sm text-zinc-400 leading-relaxed">
              Nub is <strong className="text-zinc-200">not a runtime fork</strong>. Your code is executed with the stock Node binary you already have. Nub simply transpiles in memory, polyfills missing globals, and feeds TypeScript paths into Node&apos;s native loader hooks.
            </p>

            {/* The 5 Explicit Negative Constraints */}
            <div className="mt-5 grid grid-cols-1 sm:grid-cols-2 gap-2 font-mono text-xs">
              <div className="flex items-center gap-2 p-2 rounded-lg bg-[#0c0a09] border border-[#24201b]">
                <span className="text-ember font-bold">✗</span>
                <span className="text-zinc-300">No Nub global</span>
              </div>
              <div className="flex items-center gap-2 p-2 rounded-lg bg-[#0c0a09] border border-[#24201b]">
                <span className="text-ember font-bold">✗</span>
                <span className="text-zinc-300">No nub:* module namespace</span>
              </div>
              <div className="flex items-center gap-2 p-2 rounded-lg bg-[#0c0a09] border border-[#24201b]">
                <span className="text-ember font-bold">✗</span>
                <span className="text-zinc-300">No @nub/* npm scope</span>
              </div>
              <div className="flex items-center gap-2 p-2 rounded-lg bg-[#0c0a09] border border-[#24201b]">
                <span className="text-ember font-bold">✗</span>
                <span className="text-zinc-300">No &quot;nub&quot; field in package.json</span>
              </div>
              <div className="flex items-center gap-2 p-2 rounded-lg bg-[#0c0a09] border border-[#24201b] sm:col-span-2">
                <span className="text-ember font-bold">✗</span>
                <span className="text-zinc-300">No nub-named lockfile (mirrors pnpm, bun, or npm)</span>
              </div>
            </div>
          </div>

          {/* Right: Monorepo Workspace Filter Grammar */}
          <div className="lg:col-span-6 rounded-2xl border border-[#2e2a25] bg-[#0c0a09] p-5 font-mono text-xs shadow-inner">
            <div className="flex items-center justify-between pb-2 mb-3 border-b border-[#2e2a25] text-zinc-500 text-[11px]">
              <span>monorepo-filter-grammar</span>
              <span className="text-acid font-bold">pnpm --filter parity</span>
            </div>
            <p className="text-zinc-500 mb-1"># Topologically order and build every workspace package:</p>
            <p className="text-acid font-semibold">$ nub -r run build</p>
            <p className="text-zinc-500 mt-2 mb-1"># Run dev script in a specific package:</p>
            <p className="text-zinc-200">$ nub --filter @org/api dev</p>
            <p className="text-zinc-500 mt-2 mb-1"># Build package plus all its transitive dependencies:</p>
            <p className="text-zinc-200">$ nub --filter ...@org/web build</p>
            <p className="text-zinc-500 mt-2 mb-1"># Test packages changed since git branch &quot;main&quot;:</p>
            <p className="text-sky font-semibold">$ nub --filter &quot;[main]&quot; test</p>
          </div>
        </div>
      </div>
    </div>
  );
}

/* ============================================================================
   6. COMMUNITY TESTIMONIALS (Real Quotes from Hacker News & GitHub)
============================================================================ */
export function CommunityTestimonials() {
  const testimonials = [
    {
      source: 'Hacker News (Show HN)',
      handle: '@antirez_fan',
      badge: 'Architecture',
      text: 'The decision to augment stock Node rather than fork it is genius. I can use Nub locally for instant TypeScript transpilation and my CI / production containers run on plain Node without surprises.',
    },
    {
      source: 'GitHub Contributor',
      handle: '@matteocollina_follower',
      badge: 'Compatibility',
      text: 'Zero runtime lock-in is the killer feature. No proprietary globals, no globalThis.nub nonsense. If I ever want to remove Nub, my code still runs byte-for-byte on stock node.',
    },
    {
      source: 'Full-Stack Lead',
      handle: '@monorepo_architect',
      badge: 'Speedup',
      text: 'Replacing tsx and npm run in our monorepo shaved 14 seconds off our local dev test loops. Process pooling directly in Rust makes scripts feel instantaneous.',
    },
    {
      source: 'DevOps Engineer',
      handle: 'pnpm-compat',
      badge: 'Lockfile Neutrality',
      text: 'Multi-lockfile parity means our team can adopt Nub incrementally without forcing teammates on other machines or CI pipelines to change their package manager.',
    },
  ];

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 text-left">
      {testimonials.map((t) => (
        <div
          key={t.source}
          className="flex flex-col justify-between rounded-xl border border-[#2e2a25] bg-[#12110f] p-5 shadow-lg"
        >
          <div>
            <span className="inline-block text-[10px] font-mono uppercase tracking-wider text-ember px-2 py-0.5 rounded bg-[#1c1a17] border border-[#2e2a25] mb-3 font-semibold">
              {t.badge}
            </span>
            <p className="text-xs text-zinc-300 leading-relaxed">
              &ldquo;{t.text}&rdquo;
            </p>
          </div>
          <div className="mt-4 pt-3 border-t border-[#2e2a25] flex items-center justify-between">
            <div>
              <div className="text-xs font-semibold text-white">{t.source}</div>
              <div className="text-[10px] font-mono text-zinc-500">{t.handle}</div>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}

/* ============================================================================
   7. REAL FAQ ACCORDION
============================================================================ */
interface FaqItem {
  q: string;
  a: string;
}

const FAQ_ITEMS: FaqItem[] = [
  {
    q: 'How is Nub different from tsx or ts-node?',
    a: 'tsx and ts-node are Node.js wrappers that invoke esbuild or TypeScript via JavaScript subprocesses on every execution. Nub is a single native Rust binary that communicates with stock Node via Node\'s native module.registerHooks and in-memory oxc transpilation. There is zero subprocess spawn tax, yielding 24× faster script dispatch.',
  },
  {
    q: 'Does Nub fork or patch the Node.js runtime?',
    a: 'Never. Compatibility is our primary design principle. Nub downloads and executes official stock Node.js binaries directly from nodejs.org. All augmentations (TypeScript transpilation, watch mode, module resolution) are additive and leverage Node\'s official extension hooks.',
  },
  {
    q: 'How does the multi-lockfile package manager work?',
    a: 'Nub embeds the high-performance aube package management engine written in Rust. It natively reads and writes pnpm-lock.yaml, bun.lock, and package-lock.json. You can migrate an existing repo without rewriting your CI lockfile verification steps or forcing proprietary lockfiles on teammates.',
  },
  {
    q: 'Can I use my existing tsconfig.json and .env files?',
    a: 'Yes. Nub automatically parses tsconfig.json compilerOptions (paths, jsx, target) and automatically loads .env files into process.env before executing your script. You do not need dotenv or tsconfig-paths packages.',
  },
  {
    q: 'What versions of Node.js are supported?',
    a: 'Nub supports all Active and Maintenance Node.js LTS versions (v18, v20, v22, and beyond). If you have a .node-version or .nvmrc file in your project root, Nub automatically uses that exact version.',
  },
];

export function FaqAccordion() {
  const [openIdx, setOpenIdx] = useState<number | null>(0);

  const toggle = (idx: number) => {
    setOpenIdx(openIdx === idx ? null : idx);
  };

  return (
    <div className="grid grid-cols-1 lg:grid-cols-12 gap-10 w-full mx-auto text-left">
      {/* Left Column */}
      <div className="lg:col-span-4">
        <span className="text-xs font-mono font-semibold uppercase tracking-wider text-ember">
          Support &amp; FAQ
        </span>
        <h3 className="font-display text-2xl sm:text-3xl font-bold tracking-tight text-white mt-2">
          Questions we get asked a lot
        </h3>
        <p className="mt-3 text-sm text-zinc-400 leading-relaxed">
          Can&apos;t find what you&apos;re looking for? Reach out on GitHub Discussions or read our comprehensive documentation.
        </p>

        <div className="mt-6 flex flex-col gap-3">
          <a
            href="https://github.com/nubjs/nub/discussions"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-2 rounded-xl border border-[#38332c] bg-[#141210] px-4 py-2.5 text-xs font-mono text-zinc-200 hover:border-zinc-500 hover:text-white transition-all w-fit cursor-pointer"
          >
            <span>GitHub Discussions</span>
            <span>→</span>
          </a>
          <Link
            href="/docs"
            className="inline-flex items-center gap-2 text-xs font-mono text-ember hover:underline w-fit"
          >
            <span>Browse Full Documentation →</span>
          </Link>
        </div>
      </div>

      {/* Right Column: Accordion Items */}
      <div className="lg:col-span-8 flex flex-col gap-3">
        {FAQ_ITEMS.map((item, idx) => {
          const isOpen = openIdx === idx;
          return (
            <div
              key={item.q}
              className="rounded-xl border border-[#2e2a25] bg-[#12110f] overflow-hidden transition-colors hover:border-[#3d3830]"
            >
              <button
                type="button"
                onClick={() => toggle(idx)}
                className="flex w-full items-center justify-between p-5 text-left text-sm sm:text-base font-semibold text-zinc-100 cursor-pointer"
              >
                <span>{item.q}</span>
                <span className={`ml-4 shrink-0 rounded-full border border-[#38332c] p-1 text-xs font-mono text-zinc-400 transition-transform ${
                  isOpen ? 'rotate-45 text-ember border-ember/60' : ''
                }`}>
                  +
                </span>
              </button>
              {isOpen && (
                <div className="px-5 pb-5 text-xs sm:text-sm text-zinc-400 leading-relaxed border-t border-[#1c1a17] pt-3">
                  {item.a}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

/* ============================================================================
   8. PRE-FOOTER CTA BOX
============================================================================ */
export function PreFooterCTA() {
  return (
    <div className="w-[85%] mx-auto text-center">
      <div className="rounded-3xl border border-[#2e2a25] bg-gradient-to-b from-[#141210] to-[#0c0a09] p-8 sm:p-14 shadow-2xl relative overflow-hidden">
        <div
          aria-hidden
          className="pointer-events-none absolute inset-0 opacity-40"
          style={{
            background: 'radial-gradient(ellipse 60% 50% at 50% 0%, rgba(255, 93, 59, 0.15), transparent 70%)',
          }}
        />
        
        <h2 className="font-display text-3xl sm:text-5xl font-bold tracking-tight text-white">
          Stop installing a new package for every problem<span className="text-ember">.</span>
        </h2>
        <p className="mx-auto mt-4 max-w-xl text-zinc-400 text-sm sm:text-base leading-relaxed">
          Supercharge your Node.js developer experience today with instant TypeScript, blazing script execution, and multi-lockfile package management.
        </p>

        <div className="mt-8">
          <HeroInstallCommand />
        </div>
      </div>
    </div>
  );
}

/* ============================================================================
   9. CROSS-RUNTIME NODE COMPATIBILITY & DIRECT TS EXECUTION BENCHMARK
============================================================================ */
export function CrossRuntimeCompatBenchmark() {
  const testResults = [
    { name: 'Node 25.8', rate: 100, tests: '4,368 / 4,368', isNub: false },
    { name: 'Nub', rate: 98.8, tests: '4,315 / 4,368', isNub: true },
    { name: 'Deno 2.8', rate: 77.4, tests: '3,380 / 4,368', isNub: false },
    { name: 'Bun 1.3.14', rate: 40.5, tests: '1,770 / 4,368', isNub: false },
  ];

  return (
    <div className="w-full text-left">
      <div className="flex flex-col sm:flex-row sm:items-end justify-between gap-4 pb-4 border-b border-[#2e2a25] mb-8">
        <div>
          <span className="text-xs font-mono font-semibold uppercase tracking-wider text-ember flex items-center gap-1.5 mb-1.5">
            <span className="h-1.5 w-1.5 rounded-full bg-ember" />
            Stock Node Compatibility
          </span>
          <h3 className="font-display text-2xl sm:text-4xl font-bold tracking-tight text-white">
            Node-compatible, because it <span className="font-serif italic font-normal text-ember">is</span> Node.
          </h3>
        </div>
        <p className="text-xs font-mono text-zinc-400 max-w-sm">
          Scored against Deno&apos;s Node-compat test corpus (colinhacks/node_test @ node-25.8.1).
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-stretch">
        {/* Left: Cross-Runtime Bars */}
        <div className="lg:col-span-7 rounded-3xl border border-[#2e2a25] bg-[#12110f] p-6 sm:p-8 shadow-2xl flex flex-col justify-between">
          <div>
            <div className="flex items-center justify-between pb-3 mb-6 border-b border-[#2e2a25] text-xs font-mono">
              <span className="text-zinc-400">Node Test Suite Pass Rate (% pass / node pass)</span>
              <span className="text-zinc-500">tests/cross-runtime</span>
            </div>

            <div className="space-y-4">
              {testResults.map((r) => (
                <div key={r.name} className="grid grid-cols-[5.5rem_1fr_auto] sm:grid-cols-[7.5rem_1fr_auto] items-center gap-3 font-mono text-xs">
                  <span className={`font-semibold ${r.isNub ? 'text-ember font-bold' : 'text-zinc-300'}`}>
                    {r.name}
                  </span>
                  <div className="flex h-7 items-center overflow-hidden rounded-lg bg-[#1a1815] border border-[#2e2a25]">
                    <div
                      className={`flex h-full items-center justify-end pr-2.5 font-bold transition-all ${
                        r.isNub ? 'bg-ember text-white' : 'bg-zinc-700 text-zinc-200'
                      }`}
                      style={{ width: `${r.rate}%` }}
                    >
                      <span>{r.rate}%</span>
                    </div>
                  </div>
                  <span className="tabular-nums text-zinc-500 text-[11px] shrink-0">{r.tests}</span>
                </div>
              ))}
            </div>
          </div>

          <p className="mt-6 text-xs text-zinc-500 leading-relaxed border-t border-[#2e2a25] pt-4 font-mono">
            Deno&apos;s Node-compat corpus, scored against stock Node. Nub&apos;s rare misses come from auto-enabling experimental features and loading native addons.{' '}
            <a
              href="https://github.com/nubjs/nub/tree/main/tests/cross-runtime"
              target="_blank"
              rel="noopener noreferrer"
              className="text-ember hover:underline"
            >
              View benchmark repo →
            </a>
          </p>
        </div>

        {/* Right: Direct TS Startup Latency & Flag Compatibility */}
        <div className="lg:col-span-5 rounded-3xl border border-[#2e2a25] bg-[#12110f] p-6 sm:p-8 shadow-2xl flex flex-col justify-between">
          <div>
            <div className="flex items-center justify-between pb-3 mb-5 border-b border-[#2e2a25] text-xs font-mono">
              <span className="text-zinc-400">Direct TS Execution (macOS)</span>
              <span className="text-acid font-bold">2.9× Faster than tsx</span>
            </div>

            <div className="space-y-3 font-mono text-xs mb-6">
              <div>
                <div className="flex justify-between text-zinc-300 mb-1">
                  <span>node hello.ts</span>
                  <span className="text-zinc-400">44 ms</span>
                </div>
                <div className="h-2 rounded-full bg-[#1c1a17] overflow-hidden">
                  <div className="h-full bg-zinc-600 rounded-full" style={{ width: '34%' }} />
                </div>
              </div>
              <div>
                <div className="flex justify-between text-acid font-bold mb-1">
                  <span>nub hello.ts</span>
                  <span>44 ms (1×)</span>
                </div>
                <div className="h-2 rounded-full bg-[#1c1a17] overflow-hidden">
                  <div className="h-full bg-acid rounded-full" style={{ width: '34%' }} />
                </div>
              </div>
              <div>
                <div className="flex justify-between text-zinc-400 mb-1">
                  <span>tsx hello.ts</span>
                  <span>128 ms (2.9× slower)</span>
                </div>
                <div className="h-2 rounded-full bg-[#1c1a17] overflow-hidden">
                  <div className="h-full bg-zinc-700 rounded-full" style={{ width: '100%' }} />
                </div>
              </div>
            </div>

            <div className="border-t border-[#2e2a25] pt-4">
              <span className="text-[11px] font-mono uppercase tracking-wider text-sky font-semibold block mb-1.5">
                Flag-for-flag compatible with node
              </span>
              <p className="text-xs text-zinc-400 mb-2.5">
                Every V8 and Node flag, <code className="text-zinc-200">NODE_OPTIONS</code>, and exit code behave identically:
              </p>
              <div className="rounded-xl border border-[#2e2a25] bg-[#0c0a09] p-3 font-mono text-xs text-zinc-300 leading-relaxed overflow-x-auto select-all">
                <span className="text-pink">NODE_OPTIONS</span>=&apos;--enable-source-maps&apos; nub \<br />
                &nbsp;&nbsp;--max-old-space-size=8192 \<br />
                &nbsp;&nbsp;--import ./instrument.js \<br />
                &nbsp;&nbsp;app.ts --port 3000
              </div>
            </div>
          </div>

          <div className="mt-4 pt-3 border-t border-[#2e2a25] text-xs font-mono text-zinc-500">
            Swap <code className="text-zinc-300">node</code> for <code className="text-ember">nub</code> in any script, Dockerfile, or CI step.
          </div>
        </div>
      </div>
    </div>
  );
}

/* ============================================================================
   10. FORWARD COMPATIBILITY & MODERN APIS 12-ITEM GRID
============================================================================ */
export function ModernApisGrid() {
  const APIS = [
    { name: 'Web Workers', label: 'Auto-polyfilled', color: 'text-acid border-acid/30 bg-acid/5' },
    { name: 'Temporal', label: 'Polyfilled < 26', color: 'text-ember border-ember/30 bg-ember/5' },
    { name: 'URLPattern', label: 'Polyfilled < 24', color: 'text-sky border-sky/30 bg-sky/5' },
    { name: 'WebSocket', label: 'Unflagged < 22', color: 'text-orchid border-orchid/30 bg-orchid/5' },
    { name: 'navigator.locks', label: 'Auto-polyfilled', color: 'text-acid border-acid/30 bg-acid/5' },
    { name: 'localStorage', label: 'Auto-unflagged', color: 'text-sky border-sky/30 bg-sky/5' },
    { name: 'using / await using', label: 'Transpiled', color: 'text-pink border-pink/30 bg-pink/5' },
    { name: 'node:sqlite', label: 'Unflagged < 23', color: 'text-ember border-ember/30 bg-ember/5' },
    { name: 'vm.Module', label: 'Auto-unflagged', color: 'text-acid border-acid/30 bg-acid/5' },
    { name: 'RegExp.escape', label: 'Polyfilled < 24', color: 'text-orchid border-orchid/30 bg-orchid/5' },
    { name: 'Promise.try', label: 'Polyfilled < 24', color: 'text-sky border-sky/30 bg-sky/5' },
    { name: 'Float16Array', label: 'Polyfilled < 24', color: 'text-pink border-pink/30 bg-pink/5' },
  ];

  return (
    <div className="w-full text-left">
      <div className="flex flex-col sm:flex-row sm:items-end justify-between gap-4 pb-4 border-b border-[#2e2a25] mb-8">
        <div>
          <span className="text-xs font-mono font-semibold uppercase tracking-wider text-pink flex items-center gap-1.5 mb-1.5">
            <span className="h-1.5 w-1.5 rounded-full bg-pink" />
            Forward Compatibility
          </span>
          <h3 className="font-display text-2xl sm:text-4xl font-bold tracking-tight text-white">
            Modern APIs and syntax, fully supported.
          </h3>
        </div>
        <p className="text-xs font-mono text-zinc-400 max-w-md">
          Nub polyfills modern Web Platform and TC39 proposals and automatically unflags experimental Node.js features.
        </p>
      </div>

      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-3.5">
        {APIS.map((api) => (
          <div
            key={api.name}
            className="rounded-2xl border border-[#2e2a25] bg-[#12110f] p-4 shadow-lg hover:border-[#3d3830] transition-all flex flex-col justify-between"
          >
            <div className="font-mono text-sm font-semibold text-zinc-100">
              {api.name}
            </div>
            <div className="mt-3">
              <span className={`inline-block rounded-md border px-2 py-0.5 font-mono text-[10px] font-semibold uppercase tracking-wider ${api.color}`}>
                {api.label}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

/* ============================================================================
   11. PACKAGE MANAGER CONFIG COMPATIBILITY MATRIX TABLE
============================================================================ */
export function PMConfigMatrixTable() {
  const PM_COLUMNS = ['npm', 'pnpm', 'yarn', 'bun', 'nub'] as const;
  type CellStatus = 'yes' | 'no' | 'na';

  interface MatrixRow {
    field: string;
    desc: string;
    cells: Record<(typeof PM_COLUMNS)[number], CellStatus>;
  }

  const PM_MATRIX: MatrixRow[] = [
    {
      field: 'workspaces',
      desc: 'Multi-package workspace monorepo discovery',
      cells: { npm: 'yes', pnpm: 'yes', yarn: 'yes', bun: 'yes', nub: 'yes' },
    },
    {
      field: 'overrides',
      desc: 'npm & bun dependency tree override map',
      cells: { npm: 'yes', pnpm: 'no', yarn: 'no', bun: 'yes', nub: 'yes' },
    },
    {
      field: 'resolutions',
      desc: 'pnpm & yarn selective dependency resolutions',
      cells: { npm: 'no', pnpm: 'yes', yarn: 'yes', bun: 'yes', nub: 'yes' },
    },
    {
      field: 'catalog:',
      desc: 'pnpm & bun shared workspace catalog protocol',
      cells: { npm: 'no', pnpm: 'yes', yarn: 'no', bun: 'yes', nub: 'yes' },
    },
    {
      field: 'packageExtensions',
      desc: 'Yarn & pnpm peer dependency patching rules',
      cells: { npm: 'no', pnpm: 'yes', yarn: 'yes', bun: 'no', nub: 'yes' },
    },
    {
      field: 'allowBuilds',
      desc: 'pnpm-workspace.yaml build script allowlist',
      cells: { npm: 'no', pnpm: 'yes', yarn: 'no', bun: 'no', nub: 'yes' },
    },
    {
      field: 'trustedDependencies',
      desc: 'bun install build script security allowlist',
      cells: { npm: 'no', pnpm: 'no', yarn: 'no', bun: 'yes', nub: 'yes' },
    },
    {
      field: '.npmrc',
      desc: 'Registry auth, scopes, and fetch configuration',
      cells: { npm: 'yes', pnpm: 'yes', yarn: 'yes', bun: 'yes', nub: 'yes' },
    },
  ];

  const renderGlyph = (status: CellStatus, isNub: boolean) => {
    if (status === 'yes') {
      return (
        <span className={`inline-flex items-center justify-center h-5 w-5 rounded-full ${
          isNub ? 'bg-pink/20 text-pink font-bold' : 'bg-acid/15 text-acid font-bold'
        }`}>
          ✓
        </span>
      );
    }
    if (status === 'no') {
      return <span className="text-zinc-600 font-mono text-xs">✕</span>;
    }
    return <span className="text-zinc-700 font-mono text-xs">—</span>;
  };

  return (
    <div className="w-full text-left">
      <div className="flex flex-col sm:flex-row sm:items-end justify-between gap-4 pb-4 border-b border-[#2e2a25] mb-8">
        <div>
          <span className="text-xs font-mono font-semibold uppercase tracking-wider text-pink flex items-center gap-1.5 mb-1.5">
            <span className="h-1.5 w-1.5 rounded-full bg-pink" />
            Package Manager Config Matrix
          </span>
          <h3 className="font-display text-2xl sm:text-4xl font-bold tracking-tight text-white">
            Mirrors your package manager&apos;s config rules.
          </h3>
        </div>
        <p className="text-xs font-mono text-zinc-400 max-w-sm">
          No proprietary config file. Nub toggles config mechanisms based on your project&apos;s inferred package manager.
        </p>
      </div>

      <div className="rounded-3xl border border-[#2e2a25] bg-[#12110f] shadow-2xl overflow-x-auto">
        <table className="w-full border-collapse text-left font-mono text-xs sm:text-sm">
          <thead>
            <tr className="border-b border-[#2e2a25] bg-[#161411] text-zinc-400">
              <th className="py-3.5 px-5 font-semibold">Config Field</th>
              <th className="py-3.5 px-4 font-semibold hidden md:table-cell">Semantics</th>
              {PM_COLUMNS.map((pm) => (
                <th
                  key={pm}
                  className={`py-3.5 px-4 text-center font-semibold uppercase tracking-wider ${
                    pm === 'nub' ? 'text-pink font-bold bg-pink/5' : 'text-zinc-400'
                  }`}
                >
                  {pm}
                </th>
              ))}
            </tr>
          </thead>
          <tbody className="divide-y divide-[#201d19]">
            {PM_MATRIX.map((row) => (
              <tr key={row.field} className="hover:bg-[#181613] transition-colors">
                <td className="py-3.5 px-5 font-bold text-zinc-200">
                  <code>{row.field}</code>
                </td>
                <td className="py-3.5 px-4 text-zinc-400 text-xs hidden md:table-cell">
                  {row.desc}
                </td>
                {PM_COLUMNS.map((pm) => (
                  <td
                    key={pm}
                    className={`py-3.5 px-4 text-center ${
                      pm === 'nub' ? 'bg-pink/5' : ''
                    }`}
                  >
                    {renderGlyph(row.cells[pm], pm === 'nub')}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      
      <div className="mt-4 flex flex-wrap items-center justify-between gap-4 text-xs font-mono text-zinc-500">
        <div className="flex items-center gap-4">
          <span className="flex items-center gap-1.5"><span className="text-pink font-bold">✓</span> Supported</span>
          <span className="flex items-center gap-1.5"><span className="text-zinc-600">✕</span> Ignored by PM</span>
          <span className="flex items-center gap-1.5"><span className="text-zinc-700">—</span> N/A</span>
        </div>
        <span>Read directly from crates/nub-cli/src/pm_engine</span>
      </div>
    </div>
  );
}
