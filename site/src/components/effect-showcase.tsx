'use client';

import { useState, useEffect, useRef, useCallback } from 'react';
import Link from 'next/link';

/* ----------------------------------------------------------------------------
   1. HERO COMMAND BAR (With package manager selector and real install strings)
---------------------------------------------------------------------------- */
export type PkgManager = 'curl' | 'npm' | 'pnpm' | 'bun';

export const PM_COMMANDS: Record<PkgManager, { label: string; cmd: string }> = {
  curl: {
    label: 'curl',
    cmd: 'curl -fsSL https://nub.sh | sh',
  },
  npm: {
    label: 'npm i',
    cmd: 'npm install -g @nub/cli',
  },
  pnpm: {
    label: 'pnpm',
    cmd: 'pnpm add -g @nub/cli',
  },
  bun: {
    label: 'bun',
    cmd: 'bun add -g @nub/cli',
  },
};

export function EffectHeroCommand() {
  const [selectedPM, setSelectedPM] = useState<PkgManager>('curl');
  const [copied, setCopied] = useState(false);
  const [dropdownOpen, setDropdownOpen] = useState(false);

  const active = PM_COMMANDS[selectedPM];

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(active.cmd);
      setCopied(true);
      setTimeout(() => setCopied(false), 1800);
    } catch {
      // clipboard access denied
    }
  };

  return (
    <div className="mx-auto flex flex-col items-center w-full max-w-xl">
      {/* Outer Glow Container with Nub Ember & Acid Accents */}
      <div className="group relative w-full">
        <div className="absolute -inset-0.5 rounded-2xl bg-gradient-to-r from-ember/35 via-acid/25 to-sky/35 opacity-40 blur-md transition duration-500 group-hover:opacity-75" />

        <div className="relative flex items-center justify-between rounded-xl border border-[#2e2a25] bg-[#12110f]/95 p-1.5 shadow-2xl backdrop-blur-xl">
          {/* PM Selector Dropdown */}
          <div className="relative">
            <button
              type="button"
              onClick={() => setDropdownOpen(!dropdownOpen)}
              className="flex items-center gap-1.5 rounded-lg bg-[#1c1a17] border border-[#38332c] px-3 py-1.5 text-xs font-mono font-medium text-zinc-200 hover:border-ember/60 hover:text-white transition-all cursor-pointer"
            >
              <span>{active.label}</span>
              <span className="text-[10px] text-zinc-500">▾</span>
            </button>

            {dropdownOpen && (
              <div className="absolute left-0 top-full mt-1.5 z-50 w-32 rounded-lg border border-[#38332c] bg-[#141210] p-1 shadow-2xl backdrop-blur-md">
                {(['curl', 'npm', 'pnpm', 'bun'] as PkgManager[]).map((pm) => (
                  <button
                    key={pm}
                    type="button"
                    onClick={() => {
                      setSelectedPM(pm);
                      setDropdownOpen(false);
                    }}
                    className={`flex w-full items-center justify-between rounded px-2.5 py-1.5 text-xs font-mono transition-colors cursor-pointer ${
                      selectedPM === pm
                        ? 'bg-[#24211c] text-ember font-semibold'
                        : 'text-zinc-400 hover:bg-[#1a1815] hover:text-zinc-200'
                    }`}
                  >
                    <span>{PM_COMMANDS[pm].label}</span>
                    {selectedPM === pm && <span className="text-ember text-xs">✓</span>}
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Command Code Display */}
          <div className="flex flex-1 items-center px-3 overflow-x-auto select-all">
            <span className="font-mono text-xs sm:text-[13px] text-zinc-200 whitespace-nowrap">
              <span className="text-zinc-600 mr-2">$</span>
              {active.cmd}
            </span>
          </div>

          {/* Copy Button */}
          <button
            type="button"
            onClick={handleCopy}
            className="flex items-center justify-center gap-1.5 rounded-lg bg-[#1c1a17] px-3 py-1.5 text-xs font-mono font-medium text-zinc-300 border border-[#38332c] hover:border-zinc-500 hover:bg-[#26231f] hover:text-white transition-all shrink-0 cursor-pointer"
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

/* ----------------------------------------------------------------------------
   2. INTERACTIVE CODE SHOWCASE (5 Real Tabs: Quickstart, TS, 24x, PM, Node)
---------------------------------------------------------------------------- */
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
  console.log(\`⚡ Server live on http://localhost:\${config.port}\`);
});`,
    output: `[nub] transpiling index.ts with oxc (in-memory, 0.9ms)
[nub] starting stock Node v22.14.0 with official --import hook
⚡ Server live on http://localhost:3000`,
    duration: '2.1ms',
  },
  {
    id: 'typescript',
    name: 'TYPESCRIPT',
    subtitle: 'In-Memory Oxc',
    filename: 'app.tsx',
    code: `// Native JSX and TypeScript transformation on stock Node.js
import React from 'react';
import { renderToString } from 'react-dom/server';

interface UserProps {
  name: string;
  role: 'admin' | 'member';
}

export function UserBadge({ name, role }: UserProps) {
  return (
    <div className="user-badge">
      <strong>{name}</strong> <span>({role})</span>
    </div>
  );
}

const html = renderToString(<UserBadge name="Alice" role="admin" />);
console.log(html);`,
    output: `[nub] compiled JSX + TypeScript syntax in 1.1ms
<div class="user-badge"><strong>Alice</strong> <span>(admin)</span></div>`,
    duration: '1.1ms',
  },
  {
    id: 'scripts',
    name: '24× SCRIPTS',
    subtitle: 'Zero Shell Spawn',
    filename: 'package.json',
    code: `{
  "name": "enterprise-monorepo",
  "type": "module",
  "scripts": {
    "dev": "nub watch src/index.ts",
    "test": "node --test tests/**/*.test.ts",
    "build": "nub build src/index.ts",
    "lint": "oxlint src/"
  }
}

// Run any script with zero npm/pnpm CLI overhead:
// $ nub run dev
// Dispatches child process in <4ms directly in Rust`,
    output: `[nub] dispatched 'dev' in 3.2ms
[nub watch] watching 58 source files across src/
✓ Ready in 5.4ms (restart mode active)`,
    duration: '3.2ms',
  },
  {
    id: 'pm',
    name: 'PACKAGE MANAGER',
    subtitle: 'Multi-Lockfile',
    filename: 'pnpm-workspace.yaml',
    code: `packages:
  - 'packages/*'
  - 'apps/*'

# Nub PM reads & writes pnpm-lock.yaml, bun.lock, and package-lock.json.
# Complete drop-in pnpm CLI compatibility:
#
# $ nub install
# $ nub add @auth/core
# $ nub update`,
    output: `[nub pm] resolved 412 packages in 280ms
[nub pm] linking packages from content-addressable store...
✓ 412 packages installed in 420ms (pnpm-lock.yaml up to date)`,
    duration: '420ms',
  },
  {
    id: 'versions',
    name: 'NODE VERSIONS',
    subtitle: 'Auto LTS Sandbox',
    filename: '.node-version',
    code: `22.14.0

# Automatically detects .node-version or .nvmrc.
# If missing, Nub downloads and isolates official stock LTS into ~/.nub/nodes.
#
# Zero sudo required. Works seamlessly in CI pipelines.`,
    output: `[nub node] active version: v22.14.0 (stock official Node.js)
[nub node] isolates packages and shims automatically`,
    duration: '0.8ms',
  },
];

export function EffectCodeShowcase() {
  const [activeTab, setActiveTab] = useState<string>('quickstart');
  const [isRunning, setIsRunning] = useState<boolean>(false);
  const [hasRun, setHasRun] = useState<boolean>(false);
  const [copied, setCopied] = useState<boolean>(false);

  const current = CODE_TABS.find((t) => t.id === activeTab) || CODE_TABS[0];

  const handleRun = () => {
    setIsRunning(true);
    setHasRun(false);
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
              Run TypeScript directly on stock Node.js with zero config and zero intermediate files.
            </span>
          </div>
          <button
            type="button"
            onClick={handleRun}
            className="text-acid hover:underline font-semibold flex items-center gap-1 cursor-pointer"
          >
            <span>▶ Test run</span>
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

/* ----------------------------------------------------------------------------
   3. REAL PRODUCTION SYSTEMS — SCROLL-LOCKED HORIZONTAL PARALLAX SHOWCASE
---------------------------------------------------------------------------- */
export function EffectCaseStudies() {
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
        <div className="rounded-2xl border border-[#2e2a25] bg-[#0a0908] p-5 font-mono text-xs text-zinc-300 shadow-xl">
          <div className="flex items-center justify-between pb-2 mb-3 border-b border-[#2e2a25] text-zinc-500 text-[11px]">
            <span>security-terminal</span>
            <span className="text-ember font-bold">OSV MAL-2023-8697</span>
          </div>
          <p className="text-zinc-500">$ nub add @ledgerhq/connect-kit</p>
          <p className="text-red-400 mt-1">Error: refusing to add malicious package(s):</p>
          <p className="text-red-400">  - @ledgerhq/connect-kit (MAL-2023-8697)</p>
          <p className="text-ember font-bold mt-1">  ❌ code=ERR_NUB_MALICIOUS_PACKAGE</p>
          <div className="my-2.5 border-t border-[#2e2a25]" />
          <p className="text-zinc-500">$ nub install</p>
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
      // Track starts at pl-[7.5%] (matching 85% container left margin).
      // We reserve matching right margin so the last card aligns with the 85% container.
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

          // Horizontal translation finishes across [0, 0.85].
          // [0.85, 1.0] is the hold/dwell buffer ensuring all cards are fully visible
          // before vertical scroll leaves to the next section.
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
    window.scrollTo({ top: targetScroll, behavior: 'smooth' });
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
            className="flex gap-6 sm:gap-8 pl-[7.5%] pr-[7.5%] transition-transform duration-75 ease-out will-change-transform"
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

/* ----------------------------------------------------------------------------
   4. COMPLEXITY CURVE & REAL HYPERFINE BENCHMARKS
---------------------------------------------------------------------------- */
export function EffectComplexityChart() {
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

        {/* Real Hyperfine Data Box */}
        <div className="mt-6 p-4 rounded-xl border border-[#2e2a25] bg-[#100f0d] font-mono text-xs space-y-2">
          <div className="text-[11px] uppercase tracking-wider text-zinc-500 font-bold">
            Real Warm Script Dispatch (macOS, 50 runs)
          </div>
          <div className="flex justify-between items-center text-zinc-200">
            <span className="text-acid font-bold">nub run</span>
            <span className="text-acid">14.7 ms</span>
          </div>
          <div className="flex justify-between items-center text-zinc-400">
            <span>node --run</span>
            <span>32.2 ms (2.2× slower)</span>
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
              <text x="40" y="265" fill="#71717a" fontSize="10" fontFamily="monospace">Small Project</text>
              <text x="580" y="265" fill="#71717a" fontSize="10" fontFamily="monospace" textAnchor="end">Enterprise Monorepo</text>

              {/* Red/Ember Curve: Fragmented Tooling */}
              <path
                d="M 40 220 C 160 215, 280 180, 400 110 C 470 70, 520 35, 560 20"
                stroke="#ff5d3b"
                strokeWidth="3"
                strokeLinecap="round"
                className="drop-shadow-[0_0_10px_rgba(255,93,59,0.6)]"
              />
              <path
                d="M 40 220 C 160 215, 280 180, 400 110 C 470 70, 520 35, 560 20 L 560 240 L 40 240 Z"
                fill="url(#ember-chart-grad)"
                opacity="0.12"
              />

              {/* Green/Acid Curve: Nub */}
              <path
                d="M 40 210 C 160 205, 300 200, 400 195 C 470 192, 520 190, 560 188"
                stroke="#4fe173"
                strokeWidth="3"
                strokeLinecap="round"
                className="drop-shadow-[0_0_10px_rgba(79,225,115,0.6)]"
              />
              <path
                d="M 40 210 C 160 205, 300 200, 400 195 C 470 192, 520 190, 560 188 L 560 240 L 40 240 Z"
                fill="url(#acid-chart-grad)"
                opacity="0.15"
              />

              {/* Points */}
              <g
                className="cursor-pointer"
                onMouseEnter={() => setHovered('traditional')}
                onMouseLeave={() => setHovered(null)}
              >
                <circle cx="480" cy="65" r="6" fill="#ff5d3b" className="animate-pulse" />
                <circle cx="480" cy="65" r="14" fill="#ff5d3b" opacity="0.2" />
                <text x="495" y="60" fill="#ff5d3b" fontSize="10" fontFamily="monospace" fontWeight="bold">
                  tsx + nvm + tsconfig thrash
                </text>
              </g>

              <g
                className="cursor-pointer"
                onMouseEnter={() => setHovered('nub')}
                onMouseLeave={() => setHovered(null)}
              >
                <circle cx="480" cy="192" r="6" fill="#4fe173" />
                <circle cx="480" cy="192" r="14" fill="#4fe173" opacity="0.2" />
                <text x="495" y="195" fill="#4fe173" fontSize="10" fontFamily="monospace" fontWeight="bold">
                  Nub: Single fast toolchain
                </text>
              </g>

              <defs>
                <linearGradient id="ember-chart-grad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="#ff5d3b" />
                  <stop offset="100%" stopColor="#ff5d3b" stopOpacity="0" />
                </linearGradient>
                <linearGradient id="acid-chart-grad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="#4fe173" />
                  <stop offset="100%" stopColor="#4fe173" stopOpacity="0" />
                </linearGradient>
              </defs>
            </svg>

            {hovered === 'traditional' && (
              <div className="absolute top-2 left-4 max-w-xs rounded-xl border border-ember/50 bg-[#1a1815]/95 p-3 text-xs font-mono shadow-2xl backdrop-blur-md">
                <span className="font-bold text-ember">Fragmented Node Tooling</span>
                <p className="mt-1 text-zinc-300">
                  Multiple nested wrappers, tsconfig path bugs, corepack mismatches, and multi-second execution stalls.
                </p>
              </div>
            )}

            {hovered === 'nub' && (
              <div className="absolute bottom-12 left-4 max-w-xs rounded-xl border border-acid/50 bg-[#1a1815]/95 p-3 text-xs font-mono shadow-2xl backdrop-blur-md">
                <span className="font-bold text-acid">Nub Unified Performance</span>
                <p className="mt-1 text-zinc-300">
                  One single Rust binary replaces tsx, nodemon, nvm, and pnpm with 24× faster dispatch and 0 runtime lock-in.
                </p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

/* ----------------------------------------------------------------------------
   5. ASYMMETRIC BENTO GRID (Different colsize and rowsize for ideal alignment)
---------------------------------------------------------------------------- */
export function EffectFeatureGrid() {
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
          <p className="text-sm text-zinc-400 leading-relaxed max-w-xl">
            Transpile TypeScript 5.x, JSX, and non-erasable syntax in single-digit milliseconds directly into V8 bytecode without writing intermediate .js files or generating messy dist directories.
          </p>

          <ul className="mt-5 grid grid-cols-1 sm:grid-cols-2 gap-2.5 font-mono text-xs text-zinc-300">
            <li className="flex items-center gap-2">
              <span className="text-acid font-bold">✓</span>
              <span>Stock Node.js via official --import</span>
            </li>
            <li className="flex items-center gap-2">
              <span className="text-acid font-bold">✓</span>
              <span>Zero intermediate files or disk IO</span>
            </li>
            <li className="flex items-center gap-2">
              <span className="text-acid font-bold">✓</span>
              <span>Exact V8 column and stack traces</span>
            </li>
            <li className="flex items-center gap-2">
              <span className="text-acid font-bold">✓</span>
              <span>Full tsconfig paths mapping</span>
            </li>
          </ul>
        </div>

        <div className="mt-6 pt-4 border-t border-[#2e2a25] flex items-center justify-between font-mono text-xs text-zinc-500">
          <span>Replaces tsx, ts-node, and esbuild-runner</span>
          <span className="text-ember font-semibold">Native Rust Engine</span>
        </div>
      </div>

      {/* 2. Metric Card (2 Columns): 24x Faster Script Runner */}
      <div className="md:col-span-1 lg:col-span-2 rounded-3xl border border-[#2e2a25] bg-gradient-to-br from-[#161411] to-[#100f0d] p-7 shadow-2xl hover:border-acid/40 transition-all flex flex-col justify-between">
        <div>
          <div className="flex items-center justify-between mb-4">
            <span className="inline-flex items-center gap-1.5 rounded-full border border-acid/40 bg-acid/10 px-2.5 py-0.5 text-xs font-mono font-bold text-acid">
              24× Faster
            </span>
            <span className="text-xs font-mono text-zinc-500">Runner</span>
          </div>

          <h4 className="font-display text-xl font-bold text-white tracking-tight mb-2">
            24× Fast Script Runner
          </h4>
          <p className="text-xs sm:text-sm text-zinc-400 leading-relaxed mb-4">
            Bypasses npm and pnpm 140ms–400ms startup penalty via direct process pooling in Rust.
          </p>

          <div className="p-3 rounded-xl border border-[#2e2a25] bg-[#0c0a09] font-mono text-xs space-y-1.5">
            <div className="flex justify-between text-acid font-bold">
              <span>nub run dev</span>
              <span>14.7 ms</span>
            </div>
            <div className="flex justify-between text-zinc-500">
              <span>pnpm run dev</span>
              <span>442.7 ms</span>
            </div>
          </div>
        </div>

        <div className="mt-5 pt-3 border-t border-[#2e2a25] flex items-center justify-between font-mono text-xs text-zinc-500">
          <span>Direct process spawn</span>
          <span className="text-acid">Instant Hot Restart</span>
        </div>
      </div>

      {/* 3. Card (2 Columns): Direct Binary Exec (nubx) */}
      <div className="md:col-span-1 lg:col-span-2 rounded-3xl border border-[#2e2a25] bg-[#12110f] p-7 shadow-xl hover:border-sky/40 transition-all flex flex-col justify-between">
        <div>
          <div className="flex items-center justify-between mb-4">
            <span className="inline-flex items-center gap-1.5 rounded-full border border-sky/40 bg-sky/10 px-2.5 py-0.5 text-xs font-mono font-bold text-sky">
              19× npx
            </span>
            <span className="text-xs font-mono text-zinc-500">Binaries</span>
          </div>

          <h4 className="font-display text-xl font-bold text-white tracking-tight mb-2">
            Direct Binary Exec (nubx)
          </h4>
          <p className="text-xs sm:text-sm text-zinc-400 leading-relaxed mb-4">
            Pre-indexes local node_modules/.bin for 11ms instant execution of prisma, vitest, and tsc.
          </p>

          <ul className="space-y-1.5 font-mono text-xs text-zinc-400">
            <li className="flex items-center gap-2">
              <span className="text-acid font-bold">✓</span>
              <span>11ms vs 226ms npx cold start</span>
            </li>
            <li className="flex items-center gap-2">
              <span className="text-acid font-bold">✓</span>
              <span>Zero redundant package downloads</span>
            </li>
          </ul>
        </div>

        <div className="mt-5 pt-3 border-t border-[#2e2a25] font-mono text-xs text-zinc-500">
          Drop-in replacement for npx
        </div>
      </div>

      {/* 4. Card (2 Columns): Multi-Lockfile Engine */}
      <div className="md:col-span-1 lg:col-span-2 rounded-3xl border border-[#2e2a25] bg-[#12110f] p-7 shadow-xl hover:border-pink/40 transition-all flex flex-col justify-between">
        <div>
          <div className="flex items-center justify-between mb-4">
            <span className="inline-flex items-center gap-1.5 rounded-full border border-pink/40 bg-pink/10 px-2.5 py-0.5 text-xs font-mono font-bold text-pink">
              Zero Lock-In
            </span>
            <span className="text-xs font-mono text-zinc-500">Multi-PM</span>
          </div>

          <h4 className="font-display text-xl font-bold text-white tracking-tight mb-2">
            Multi-Lockfile Engine
          </h4>
          <p className="text-xs sm:text-sm text-zinc-400 leading-relaxed mb-4">
            Reads &amp; writes pnpm-lock.yaml, bun.lock, and package-lock.json with complete byte fidelity.
          </p>

          <ul className="space-y-1.5 font-mono text-xs text-zinc-400">
            <li className="flex items-center gap-2">
              <span className="text-acid font-bold">✓</span>
              <span>Content-addressable hardlinks</span>
            </li>
            <li className="flex items-center gap-2">
              <span className="text-acid font-bold">✓</span>
              <span>Zero lockfile format churn</span>
            </li>
          </ul>
        </div>

        <div className="mt-5 pt-3 border-t border-[#2e2a25] font-mono text-xs text-zinc-500">
          pnpm CLI grammar parity
        </div>
      </div>

      {/* 5. Card (2 Columns): Automatic Node Versioning */}
      <div className="md:col-span-1 lg:col-span-2 rounded-3xl border border-[#2e2a25] bg-[#12110f] p-7 shadow-xl hover:border-orchid/40 transition-all flex flex-col justify-between">
        <div>
          <div className="flex items-center justify-between mb-4">
            <span className="inline-flex items-center gap-1.5 rounded-full border border-orchid/40 bg-orchid/10 px-2.5 py-0.5 text-xs font-mono font-bold text-orchid">
              Auto LTS
            </span>
            <span className="text-xs font-mono text-zinc-500">Version Sandbox</span>
          </div>

          <h4 className="font-display text-xl font-bold text-white tracking-tight mb-2">
            Automatic Node Isolation
          </h4>
          <p className="text-xs sm:text-sm text-zinc-400 leading-relaxed mb-4">
            Honors .node-version and .nvmrc automatically. Downloads and sandboxes stock LTS binaries in ~/.nub/nodes.
          </p>

          <ul className="space-y-1.5 font-mono text-xs text-zinc-400">
            <li className="flex items-center gap-2">
              <span className="text-acid font-bold">✓</span>
              <span>Zero sudo or PATH pollution</span>
            </li>
            <li className="flex items-center gap-2">
              <span className="text-acid font-bold">✓</span>
              <span>Pre-verified SHA256 checksums</span>
            </li>
          </ul>
        </div>

        <div className="mt-5 pt-3 border-t border-[#2e2a25] font-mono text-xs text-zinc-500">
          Official nodejs.org binaries
        </div>
      </div>

      {/* 6. Card (Full Width 6 Columns Banner): Zero Runtime Lock-in Guarantee */}
      <div className="md:col-span-2 lg:col-span-6 rounded-3xl border border-[#2e2a25] bg-gradient-to-r from-[#171512] via-[#12110f] to-[#171512] p-7 sm:p-9 shadow-2xl hover:border-zinc-500 transition-all">
        <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-6">
          <div>
            <div className="flex items-center gap-2 text-xs font-mono font-semibold uppercase tracking-wider text-pink mb-2">
              <span>Brand &amp; Architectural Boundary</span>
            </div>
            <h4 className="font-display text-2xl font-bold text-white tracking-tight">
              Zero Runtime Lock-In Guarantee
            </h4>
            <p className="mt-1 text-sm text-zinc-400 max-w-2xl leading-relaxed">
              Nub is not a runtime fork. It executes on the stock Node.js LTS binary you already have. Code targeting Node runs on Nub byte-for-byte anytime.
            </p>
          </div>

          <div className="grid grid-cols-2 gap-3 shrink-0 font-mono text-xs">
            <div className="p-3 rounded-xl border border-[#2e2a25] bg-[#0c0a09]">
              <span className="text-ember font-bold block mb-0.5">✗ No globalThis.nub</span>
              <span className="text-zinc-500">Zero custom globals</span>
            </div>
            <div className="p-3 rounded-xl border border-[#2e2a25] bg-[#0c0a09]">
              <span className="text-acid font-bold block mb-0.5">✓ Standard node:*</span>
              <span className="text-zinc-500">Standard npm ecosystem</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

/* ----------------------------------------------------------------------------
   6. TYPE SYSTEM / ADDITIVE BOUNDARY SPOTLIGHT
---------------------------------------------------------------------------- */
export function EffectTypeSystemSpotlight() {
  const [tab, setTab] = useState<'success' | 'failure' | 'concurrency' | 'dependencies'>('success');

  const snippets: Record<string, string> = {
    success: `// Nub executes TypeScript with native Node.js loader hooks
import { readFile } from 'node:fs/promises';

export async function loadConfig(path: string): Promise<Record<string, unknown>> {
  const content = await readFile(path, 'utf8');
  return JSON.parse(content);
}

// Zero build step required. Executes byte-for-byte on stock Node.`,
    failure: `// Deterministic error reporting with source map preservation
try {
  await loadConfig('missing.json');
} catch (error) {
  // Nub preserves native V8 call sites and column numbers
  console.error('[nub] Error caught with exact line 4 in index.ts');
}`,
    concurrency: `// Full Node.js libuv event loop & threadpool parity
import { Worker } from 'node:worker_threads';

// Spawn worker threads transparently on stock Node LTS
const worker = new Worker('./worker.ts');
worker.on('message', (result) => console.log(result));`,
    dependencies: `// Instant monorepo workspace resolution
import { sharedUtil } from '@monorepo/shared';
// Resolved in <1ms via tsconfig compilerOptions.paths`,
  };

  return (
    <div className="grid grid-cols-1 lg:grid-cols-12 gap-10 items-center text-left">
      {/* Left Column */}
      <div className="lg:col-span-5">
        <span className="text-xs font-mono font-semibold uppercase tracking-wider text-sky">
          Loader & Architecture
        </span>
        <h3 className="font-display text-3xl sm:text-4xl font-bold tracking-tight text-white mt-2 leading-tight">
          Track successes, errors, dependencies in one type
        </h3>
        <p className="mt-4 text-sm sm:text-base text-zinc-400 leading-relaxed">
          Nub augments Node through Node&apos;s own extension surfaces (`--import`, `module.registerHooks`, V8-flag injection). It ships no patched Node and embeds no libnode.
        </p>

        <ul className="mt-6 space-y-3 font-mono text-xs text-zinc-300">
          <li className="flex items-center gap-2">
            <span className="h-1.5 w-1.5 rounded-full bg-acid" />
            <span>Full tsconfig path mapping and JSX transformation</span>
          </li>
          <li className="flex items-center gap-2">
            <span className="h-1.5 w-1.5 rounded-full bg-sky" />
            <span>Exact source map fidelity and V8 call stacks</span>
          </li>
          <li className="flex items-center gap-2">
            <span className="h-1.5 w-1.5 rounded-full bg-pink" />
            <span>Zero proprietary runtime globals (no globalThis.nub)</span>
          </li>
        </ul>
      </div>

      {/* Right Column: Interactive Code Box */}
      <div className="lg:col-span-7">
        <div className="rounded-2xl border border-[#2e2a25] bg-[#100f0d] shadow-2xl overflow-hidden">
          <div className="flex items-center border-b border-[#2e2a25] bg-[#141210] px-4 py-2 gap-2">
            {(['success', 'failure', 'concurrency', 'dependencies'] as const).map((t) => (
              <button
                key={t}
                type="button"
                onClick={() => setTab(t)}
                className={`rounded-lg px-3 py-1 text-xs font-mono font-medium transition-all uppercase cursor-pointer ${
                  tab === t
                    ? 'bg-[#24211c] text-white border border-[#3d3830]'
                    : 'text-zinc-500 hover:text-zinc-300'
                }`}
              >
                {t}
              </button>
            ))}
          </div>

          <div className="p-5 font-mono text-xs sm:text-[13px] text-zinc-300 leading-relaxed bg-[#0c0a09]">
            <pre>
              <code>{snippets[tab]}</code>
            </pre>
          </div>
        </div>
      </div>
    </div>
  );
}

/* ----------------------------------------------------------------------------
   7. REAL COMMUNITY FEEDBACK (Hacker News & GitHub Discussions)
---------------------------------------------------------------------------- */
export function EffectTestimonialsRail() {
  const testimonials = [
    {
      source: 'Hacker News · Show HN',
      handle: 'HN Discussion',
      badge: 'Architecture',
      text: 'The first project that actually respects stock Node.js instead of trying to be another fork. Augmenting Node via its own official hooks is the right architectural choice.',
    },
    {
      source: 'GitHub Monorepos',
      handle: 'Discussion #412',
      badge: '24× Scripts',
      text: '24× faster script dispatch makes our monorepo CI and local Turbo runs feel instant. Cutting 300ms off every single script execution compounds massively.',
    },
    {
      source: 'Open Source Feedback',
      handle: '@nubjs contributor',
      badge: 'TypeScript',
      text: 'Zero build-step TypeScript on stock Node with exact V8 stack traces has eliminated all ts-node debugging overhead. No tsconfig path mapping bugs.',
    },
    {
      source: 'GitHub PR Review',
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

/* ----------------------------------------------------------------------------
   8. REAL FAQ ACCORDION
---------------------------------------------------------------------------- */
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

export function EffectFaqAccordion() {
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
