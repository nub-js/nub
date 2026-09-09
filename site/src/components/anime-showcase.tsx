'use client';

import React, { useState } from 'react';
import Link from 'next/link';

/* ----------------------------------------------------------------------------
   Anime.js-inspired Interactive UI Components for Nub
   Lightweight, 0 runtime dependencies, 120fps smooth CSS animations
---------------------------------------------------------------------------- */

/* --- 1. Hero Command Bar with Instant Copy --- */
export function AnimeHeroCommand() {
  const [activeTab, setActiveTab] = useState<'curl' | 'npm' | 'pnpm' | 'brew'>('curl');
  const [copied, setCopied] = useState(false);

  const commands = {
    curl: 'curl -fsSL https://nub.sh | sh',
    npm: 'npm install -g @nub/cli',
    pnpm: 'pnpm add -g @nub/cli',
    brew: 'brew install nub',
  };

  const currentCmd = commands[activeTab];

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(currentCmd);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // ignore clipboard failure
    }
  };

  return (
    <div className="w-full max-w-xl mx-auto">
      {/* Tabs */}
      <div className="flex items-center gap-1 mb-2">
        {(['curl', 'npm', 'pnpm', 'brew'] as const).map((tab) => (
          <button
            key={tab}
            type="button"
            onClick={() => setActiveTab(tab)}
            className={`px-3 py-1 text-xs font-mono rounded-md transition-colors ${
              activeTab === tab
                ? 'bg-fd-foreground/10 text-fd-foreground font-semibold'
                : 'text-fd-muted-foreground hover:text-fd-foreground'
            }`}
          >
            {tab}
          </button>
        ))}
      </div>

      {/* Command Pill */}
      <div
        onClick={handleCopy}
        className="group relative flex items-center justify-between gap-3 px-5 py-3.5 rounded-xl border border-fd-border/80 bg-fd-card/70 hover:border-ember/60 hover:bg-fd-card transition-all cursor-pointer shadow-sm hover:shadow-md"
      >
        <div className="flex items-center gap-3 min-w-0 font-mono text-sm sm:text-base">
          <span className="text-ember font-bold select-none">$</span>
          <span className="truncate text-fd-foreground">{currentCmd}</span>
        </div>
        <button
          type="button"
          aria-label="Copy install command"
          className="shrink-0 flex items-center gap-1.5 px-3 py-1 rounded-lg text-xs font-mono font-medium border border-fd-border bg-fd-muted/50 group-hover:border-ember/40 group-hover:text-ember transition-colors"
        >
          {copied ? (
            <>
              <svg className="w-3.5 h-3.5 text-acid" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
                <path d="M20 6 9 17l-5-5" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
              <span>Copied</span>
            </>
          ) : (
            <>
              <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <rect x="9" y="9" width="13" height="13" rx="2" strokeLinecap="round" strokeLinejoin="round" />
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
              <span>Copy</span>
            </>
          )}
        </button>
      </div>
    </div>
  );
}

/* --- 2. Interactive Feature Gallery Playground --- */
export function AnimeFeaturePlayground() {
  const [activeFeature, setActiveFeature] = useState<'ts' | 'run' | 'nubx' | 'pm' | 'node'>('ts');
  const [benchRunning, setBenchRunning] = useState(false);
  const [benchProgress, setBenchProgress] = useState(1);

  const runBenchmark = () => {
    setBenchRunning(true);
    setBenchProgress(0);
    const start = performance.now();
    const duration = 1000;
    const tick = (now: number) => {
      const elapsed = now - start;
      const p = Math.min(1, elapsed / duration);
      setBenchProgress(p);
      if (p < 1) {
        requestAnimationFrame(tick);
      } else {
        setBenchRunning(false);
      }
    };
    requestAnimationFrame(tick);
  };

  return (
    <div className="w-full">
      {/* Feature Selector Tabs (Anime.js sub-nav style) */}
      <div className="flex flex-wrap items-center justify-center gap-2 mb-10">
        {[
          { id: 'ts', label: '01. TypeScript & JSX', dot: 'bg-ember' },
          { id: 'run', label: '02. 24× Script Runner', dot: 'bg-acid' },
          { id: 'nubx', label: '03. 19× Binary Exec', dot: 'bg-sky' },
          { id: 'pm', label: '04. Fast Package Manager', dot: 'bg-pink' },
          { id: 'node', label: '05. Node Versioning', dot: 'bg-orchid' },
        ].map((item) => (
          <button
            key={item.id}
            type="button"
            onClick={() => setActiveFeature(item.id as any)}
            className={`inline-flex items-center gap-2 px-4 py-2 rounded-full text-xs font-mono transition-all border ${
              activeFeature === item.id
                ? 'border-fd-foreground/30 bg-fd-card text-fd-foreground font-semibold shadow-sm scale-105'
                : 'border-transparent text-fd-muted-foreground hover:text-fd-foreground hover:bg-fd-card/50'
            }`}
          >
            <span className={`w-2 h-2 rounded-full ${item.dot}`} />
            <span>{item.label}</span>
          </button>
        ))}
      </div>

      {/* Feature Content Panel */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-stretch">
        {/* Left column: Feature Details */}
        <div className="lg:col-span-6 flex flex-col justify-between rounded-2xl border border-fd-border/70 bg-fd-card/50 p-6 sm:p-8">
          <div>
            {activeFeature === 'ts' && (
              <>
                <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-mono text-ember border border-ember/30 bg-ember/10 mb-4">
                  <span>nub &lt;file&gt;</span>
                </div>
                <h3 className="font-display text-2xl sm:text-3xl font-semibold tracking-tight text-fd-foreground">
                  TypeScript without the build step.
                </h3>
                <p className="mt-3 text-fd-muted-foreground leading-relaxed">
                  Nub transpiles your code in memory using <strong className="text-fd-foreground">oxc</strong> and runs it directly on stock Node. Full support for TS enums, decorators, parameter properties, JSX, and extensionless imports that stock Node rejects.
                </p>
                <div className="mt-6 flex flex-col gap-2.5">
                  <div className="flex items-center gap-2 text-sm text-fd-foreground/90">
                    <svg className="w-4 h-4 text-ember shrink-0" viewBox="0 0 24 24" fill="currentColor">
                      <polygon points="17.737 11.987 12.5 17.225 11.263 15.987 14.388 12.862 6.5 12.862 6.5 11.112 14.388 11.112 11.263 7.987 12.5 6.75"/>
                    </svg>
                    <span>Auto-loads <code className="text-xs bg-fd-muted px-1.5 py-0.5 rounded font-mono">.env</code> &amp; <code className="text-xs bg-fd-muted px-1.5 py-0.5 rounded font-mono">.env.local</code> without dotenv</span>
                  </div>
                  <div className="flex items-center gap-2 text-sm text-fd-foreground/90">
                    <svg className="w-4 h-4 text-ember shrink-0" viewBox="0 0 24 24" fill="currentColor">
                      <polygon points="17.737 11.987 12.5 17.225 11.263 15.987 14.388 12.862 6.5 12.862 6.5 11.112 14.388 11.112 11.263 7.987 12.5 6.75"/>
                    </svg>
                    <span>Respects <code className="text-xs bg-fd-muted px-1.5 py-0.5 rounded font-mono">tsconfig.json</code> paths via resolve hooks</span>
                  </div>
                  <div className="flex items-center gap-2 text-sm text-fd-foreground/90">
                    <svg className="w-4 h-4 text-ember shrink-0" viewBox="0 0 24 24" fill="currentColor">
                      <polygon points="17.737 11.987 12.5 17.225 11.263 15.987 14.388 12.862 6.5 12.862 6.5 11.112 14.388 11.112 11.263 7.987 12.5 6.75"/>
                    </svg>
                    <span>Direct imports of JSON, YAML, TOML, and JSONC</span>
                  </div>
                </div>
              </>
            )}

            {activeFeature === 'run' && (
              <>
                <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-mono text-acid border border-acid/30 bg-acid/10 mb-4">
                  <span>nub run &lt;script&gt;</span>
                </div>
                <h3 className="font-display text-2xl sm:text-3xl font-semibold tracking-tight text-fd-foreground">
                  A 24× faster script runner.
                </h3>
                <p className="mt-3 text-fd-muted-foreground leading-relaxed">
                  Whereas <code className="font-mono text-xs text-fd-foreground">npm run</code> and <code className="font-mono text-xs text-fd-foreground">pnpm run</code> cold-boot heavy JavaScript runtimes paying 300–450ms of overhead before starting your command, Nub is written in Rust and dispatches scripts in <strong>14.7ms</strong>.
                </p>
                <div className="mt-6 flex flex-col gap-2.5">
                  <div className="flex items-center gap-2 text-sm text-fd-foreground/90">
                    <svg className="w-4 h-4 text-acid shrink-0" viewBox="0 0 24 24" fill="currentColor">
                      <polygon points="17.737 11.987 12.5 17.225 11.263 15.987 14.388 12.862 6.5 12.862 6.5 11.112 14.388 11.112 11.263 7.987 12.5 6.75"/>
                    </svg>
                    <span>Full <code className="text-xs bg-fd-muted px-1.5 py-0.5 rounded font-mono">--filter</code> and <code className="text-xs bg-fd-muted px-1.5 py-0.5 rounded font-mono">-r</code> monorepo workspaces</span>
                  </div>
                  <div className="flex items-center gap-2 text-sm text-fd-foreground/90">
                    <svg className="w-4 h-4 text-acid shrink-0" viewBox="0 0 24 24" fill="currentColor">
                      <polygon points="17.737 11.987 12.5 17.225 11.263 15.987 14.388 12.862 6.5 12.862 6.5 11.112 14.388 11.112 11.263 7.987 12.5 6.75"/>
                    </svg>
                    <span>Automatic lifecycle hooks (<code className="text-xs font-mono">prebuild</code>, <code className="text-xs font-mono">postbuild</code>)</span>
                  </div>
                  <div className="flex items-center gap-2 text-sm text-fd-foreground/90">
                    <svg className="w-4 h-4 text-acid shrink-0" viewBox="0 0 24 24" fill="currentColor">
                      <polygon points="17.737 11.987 12.5 17.225 11.263 15.987 14.388 12.862 6.5 12.862 6.5 11.112 14.388 11.112 11.263 7.987 12.5 6.75"/>
                    </svg>
                    <span>Full argument forwarding with <code className="text-xs font-mono">--</code> delimiter</span>
                  </div>
                </div>
              </>
            )}

            {activeFeature === 'nubx' && (
              <>
                <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-mono text-sky border border-sky/30 bg-sky/10 mb-4">
                  <span>nubx / nub dlx</span>
                </div>
                <h3 className="font-display text-2xl sm:text-3xl font-semibold tracking-tight text-fd-foreground">
                  A 19× faster npx &amp; CLI runner.
                </h3>
                <p className="mt-3 text-fd-muted-foreground leading-relaxed">
                  Runs local binaries from <code className="font-mono text-xs text-fd-foreground">node_modules/.bin</code> in <strong>11ms</strong> without wrapping in an intermediate Node.js process. Drop-in replacement for <code className="font-mono text-xs">npx</code> and <code className="font-mono text-xs">pnpm exec</code>.
                </p>
                <div className="mt-6 flex flex-col gap-2.5">
                  <div className="flex items-center gap-2 text-sm text-fd-foreground/90">
                    <svg className="w-4 h-4 text-sky shrink-0" viewBox="0 0 24 24" fill="currentColor">
                      <polygon points="17.737 11.987 12.5 17.225 11.263 15.987 14.388 12.862 6.5 12.862 6.5 11.112 14.388 11.112 11.263 7.987 12.5 6.75"/>
                    </svg>
                    <span>Walks ancestor directories to resolve monorepo bins</span>
                  </div>
                  <div className="flex items-center gap-2 text-sm text-fd-foreground/90">
                    <svg className="w-4 h-4 text-sky shrink-0" viewBox="0 0 24 24" fill="currentColor">
                      <polygon points="17.737 11.987 12.5 17.225 11.263 15.987 14.388 12.862 6.5 12.862 6.5 11.112 14.388 11.112 11.263 7.987 12.5 6.75"/>
                    </svg>
                    <span>Instantly downloads uninstalled packages via dlx</span>
                  </div>
                  <div className="flex items-center gap-2 text-sm text-fd-foreground/90">
                    <svg className="w-4 h-4 text-sky shrink-0" viewBox="0 0 24 24" fill="currentColor">
                      <polygon points="17.737 11.987 12.5 17.225 11.263 15.987 14.388 12.862 6.5 12.862 6.5 11.112 14.388 11.112 11.263 7.987 12.5 6.75"/>
                    </svg>
                    <span>Works with any package manager (npm, pnpm, bun)</span>
                  </div>
                </div>
              </>
            )}

            {activeFeature === 'pm' && (
              <>
                <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-mono text-pink border border-pink/30 bg-pink/10 mb-4">
                  <span>nub install</span>
                </div>
                <h3 className="font-display text-2xl sm:text-3xl font-semibold tracking-tight text-fd-foreground">
                  A 5× faster pnpm, built in.
                </h3>
                <p className="mt-3 text-fd-muted-foreground leading-relaxed">
                  Embedded <strong className="text-fd-foreground">aube</strong> Rust engine reads whatever lockfile your project already has (<code className="font-mono text-xs">pnpm-lock.yaml</code>, <code className="font-mono text-xs">package-lock.json</code>, or <code className="font-mono text-xs">bun.lock</code>) and writes the exact same format back.
                </p>
                <div className="mt-6 flex flex-col gap-2.5">
                  <div className="flex items-center gap-2 text-sm text-fd-foreground/90">
                    <svg className="w-4 h-4 text-pink shrink-0" viewBox="0 0 24 24" fill="currentColor">
                      <polygon points="17.737 11.987 12.5 17.225 11.263 15.987 14.388 12.862 6.5 12.862 6.5 11.112 14.388 11.112 11.263 7.987 12.5 6.75"/>
                    </svg>
                    <span>Zero lock-in: switch freely without migration headaches</span>
                  </div>
                  <div className="flex items-center gap-2 text-sm text-fd-foreground/90">
                    <svg className="w-4 h-4 text-pink shrink-0" viewBox="0 0 24 24" fill="currentColor">
                      <polygon points="17.737 11.987 12.5 17.225 11.263 15.987 14.388 12.862 6.5 12.862 6.5 11.112 14.388 11.112 11.263 7.987 12.5 6.75"/>
                    </svg>
                    <span>Deny-by-default dependency postinstall scripts</span>
                  </div>
                  <div className="flex items-center gap-2 text-sm text-fd-foreground/90">
                    <svg className="w-4 h-4 text-pink shrink-0" viewBox="0 0 24 24" fill="currentColor">
                      <polygon points="17.737 11.987 12.5 17.225 11.263 15.987 14.388 12.862 6.5 12.862 6.5 11.112 14.388 11.112 11.263 7.987 12.5 6.75"/>
                    </svg>
                    <span>Content-addressed global store with O(packages) linking</span>
                  </div>
                </div>
              </>
            )}

            {activeFeature === 'node' && (
              <>
                <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-mono text-orchid border border-orchid/30 bg-orchid/10 mb-4">
                  <span>nub node</span>
                </div>
                <h3 className="font-display text-2xl sm:text-3xl font-semibold tracking-tight text-fd-foreground">
                  Built-in Node version manager.
                </h3>
                <p className="mt-3 text-fd-muted-foreground leading-relaxed">
                  Reads your <code className="font-mono text-xs text-fd-foreground">.node-version</code> or <code className="font-mono text-xs text-fd-foreground">.nvmrc</code> pin and executes on that exact version. Automatically downloads and installs Node from nodejs.org if missing.
                </p>
                <div className="mt-6 flex flex-col gap-2.5">
                  <div className="flex items-center gap-2 text-sm text-fd-foreground/90">
                    <svg className="w-4 h-4 text-orchid shrink-0" viewBox="0 0 24 24" fill="currentColor">
                      <polygon points="17.737 11.987 12.5 17.225 11.263 15.987 14.388 12.862 6.5 12.862 6.5 11.112 14.388 11.112 11.263 7.987 12.5 6.75"/>
                    </svg>
                    <span>Replaces nvm, fnm, and volta</span>
                  </div>
                  <div className="flex items-center gap-2 text-sm text-fd-foreground/90">
                    <svg className="w-4 h-4 text-orchid shrink-0" viewBox="0 0 24 24" fill="currentColor">
                      <polygon points="17.737 11.987 12.5 17.225 11.263 15.987 14.388 12.862 6.5 12.862 6.5 11.112 14.388 11.112 11.263 7.987 12.5 6.75"/>
                    </svg>
                    <span>Verifies SHA256 checksums automatically</span>
                  </div>
                  <div className="flex items-center gap-2 text-sm text-fd-foreground/90">
                    <svg className="w-4 h-4 text-orchid shrink-0" viewBox="0 0 24 24" fill="currentColor">
                      <polygon points="17.737 11.987 12.5 17.225 11.263 15.987 14.388 12.862 6.5 11.112 14.388 11.112 11.263 7.987 12.5 6.75"/>
                    </svg>
                    <span>Zero config, instant invocation</span>
                  </div>
                </div>
              </>
            )}
          </div>

          <div className="mt-8 pt-6 border-t border-fd-border/60 flex items-center justify-between">
            <span className="text-xs font-mono text-fd-muted-foreground uppercase tracking-wider">
              {activeFeature === 'ts' && 'oxc transpile · native addon'}
              {activeFeature === 'run' && '14.7ms dispatch · 50 runs'}
              {activeFeature === 'nubx' && '11ms direct binary exec'}
              {activeFeature === 'pm' && '346ms warm frozen install'}
              {activeFeature === 'node' && 'on-demand node provisioner'}
            </span>
            <Link
              href={
                activeFeature === 'ts'
                  ? '/docs/runtime'
                  : activeFeature === 'run'
                  ? '/docs/run'
                  : activeFeature === 'nubx'
                  ? '/docs/nubx'
                  : activeFeature === 'pm'
                  ? '/docs/pm'
                  : '/docs/node'
              }
              className="text-xs font-mono font-semibold text-fd-foreground hover:text-ember transition-colors flex items-center gap-1"
            >
              <span>Explore docs</span>
              <span>→</span>
            </Link>
          </div>
        </div>

        {/* Right column: Interactive Visual Simulation / Terminal */}
        <div className="lg:col-span-6 flex flex-col rounded-2xl border border-fd-border/70 bg-[#0b0a08] p-6 text-[#ece6d8] shadow-xl overflow-hidden font-mono text-xs sm:text-sm">
          {/* Terminal Window Chrome */}
          <div className="flex items-center justify-between pb-4 mb-4 border-b border-white/10">
            <div className="flex items-center gap-2">
              <span className="w-3 h-3 rounded-full bg-ember/80" />
              <span className="w-3 h-3 rounded-full bg-acid/70" />
              <span className="w-3 h-3 rounded-full bg-sky/70" />
            </div>
            <span className="text-[11px] uppercase tracking-wider text-white/40">
              {activeFeature === 'run' ? 'Benchmark Visualizer' : 'Interactive Playground'}
            </span>
          </div>

          {/* Interactive Visual content */}
          <div className="flex-1 flex flex-col justify-center">
            {activeFeature === 'ts' && (
              <div className="space-y-3 leading-relaxed">
                <div className="text-white/40">// src/server.ts (zero compile step)</div>
                <div>
                  <span className="text-pink">import</span> {'{ createServer }'} <span className="text-pink">from</span> <span className="text-acid">&quot;node:http&quot;</span>;
                </div>
                <div>
                  <span className="text-pink">import</span> config <span className="text-pink">from</span> <span className="text-acid">&quot;./config.yaml&quot;</span>;
                </div>
                <div className="pt-2">
                  <span className="text-sky">enum</span> <span className="text-ember">Status</span> {'{ Active, Idle }'}
                </div>
                <div>
                  <span className="text-sky">class</span> <span className="text-ember">Server</span> {'{'}
                  <div className="pl-4">
                    <span className="text-sky">constructor</span>(<span className="text-sky">public</span> status = <span className="text-ember">Status</span>.Active) {'{}'}
                  </div>
                  {'}'}
                </div>
                <div className="pt-4 border-t border-white/10">
                  <div className="text-white/40">$ nub src/server.ts</div>
                  <div className="text-acid font-semibold mt-1">✓ oxc transpiled in 3.4ms → running on Node v26</div>
                  <div className="text-white/70">Listening on http://localhost:3000</div>
                </div>
              </div>
            )}

            {activeFeature === 'run' && (
              <div className="space-y-4">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-xs uppercase tracking-wider text-white/50">Script Dispatch Latency (macOS)</span>
                  <button
                    type="button"
                    onClick={runBenchmark}
                    disabled={benchRunning}
                    className="px-2.5 py-1 rounded bg-acid/20 text-acid border border-acid/40 text-xs hover:bg-acid/30 transition-all cursor-pointer font-semibold"
                  >
                    {benchRunning ? 'Running...' : 'Run Benchmark'}
                  </button>
                </div>

                <div className="space-y-3 pt-2">
                  <div>
                    <div className="flex justify-between text-xs mb-1">
                      <span className="font-bold text-acid">nub run</span>
                      <span className="text-acid font-semibold">14.7 ms (fastest)</span>
                    </div>
                    <div className="h-4 bg-white/10 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-acid rounded-full transition-all duration-300"
                        style={{ width: `${Math.max(4, (14.7 / 442.7) * 100 * benchProgress)}%` }}
                      />
                    </div>
                  </div>

                  <div>
                    <div className="flex justify-between text-xs mb-1">
                      <span className="text-white/80">node --run</span>
                      <span className="text-white/50">32.2 ms (2.2×)</span>
                    </div>
                    <div className="h-4 bg-white/10 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-white/30 rounded-full transition-all duration-300"
                        style={{ width: `${(32.2 / 442.7) * 100 * benchProgress}%` }}
                      />
                    </div>
                  </div>

                  <div>
                    <div className="flex justify-between text-xs mb-1">
                      <span className="text-white/80">npm run</span>
                      <span className="text-white/50">329.9 ms (22× slower)</span>
                    </div>
                    <div className="h-4 bg-white/10 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-white/20 rounded-full transition-all duration-300"
                        style={{ width: `${(329.9 / 442.7) * 100 * benchProgress}%` }}
                      />
                    </div>
                  </div>

                  <div>
                    <div className="flex justify-between text-xs mb-1">
                      <span className="text-white/80">pnpm run</span>
                      <span className="text-white/50">442.7 ms (30× slower)</span>
                    </div>
                    <div className="h-4 bg-white/10 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-ember/60 rounded-full transition-all duration-300"
                        style={{ width: `${(442.7 / 442.7) * 100 * benchProgress}%` }}
                      />
                    </div>
                  </div>
                </div>

                <div className="text-[11px] text-white/40 pt-2 border-t border-white/10">
                  hyperfine 50 runs · warm package.json execution
                </div>
              </div>
            )}

            {activeFeature === 'nubx' && (
              <div className="space-y-3 leading-relaxed">
                <div className="text-white/40">// Direct resolution from node_modules/.bin</div>
                <div className="flex items-center gap-2">
                  <span className="text-ember">$</span>
                  <span className="text-white font-semibold">nubx prisma generate</span>
                </div>
                <div className="text-acid">✓ Resolved node_modules/.bin/prisma in 0.8ms</div>
                <div className="text-white/60">✔ Generated Prisma Client (v6.0.0) in 28ms</div>

                <div className="pt-3 border-t border-white/10 text-white/40">// dlx fallback for uninstalled packages</div>
                <div className="flex items-center gap-2">
                  <span className="text-ember">$</span>
                  <span className="text-white font-semibold">nubx cowsay &quot;Hello from Rust!&quot;</span>
                </div>
                <div className="p-3 bg-white/5 rounded border border-white/10 text-acid font-mono text-xs">
                  {' ____________________'}<br />
                  {'< Hello from Rust! >'}<br />
                  {' --------------------'}<br />
                  {'        \\   ^__^'}<br />
                  {'         \\  (oo)\\_______'}<br />
                  {'            (__)\\       )\\/\\'}<br />
                  {'                ||----w |'}<br />
                  {'                ||     ||'}
                </div>
              </div>
            )}

            {activeFeature === 'pm' && (
              <div className="space-y-4">
                <div className="text-xs uppercase tracking-wider text-white/50">Multi-Lockfile In-Place Roundtrip</div>
                <div className="grid grid-cols-3 gap-2">
                  <div className="p-3 rounded-lg border border-pink/40 bg-pink/10 text-center">
                    <div className="font-bold text-pink">pnpm</div>
                    <div className="text-[11px] text-white/60 mt-1">pnpm-lock.yaml</div>
                    <div className="text-[10px] text-acid mt-1">✓ Roundtrip</div>
                  </div>
                  <div className="p-3 rounded-lg border border-white/20 bg-white/5 text-center">
                    <div className="font-bold text-sky">npm</div>
                    <div className="text-[11px] text-white/60 mt-1">package-lock.json</div>
                    <div className="text-[10px] text-acid mt-1">✓ Roundtrip</div>
                  </div>
                  <div className="p-3 rounded-lg border border-white/20 bg-white/5 text-center">
                    <div className="font-bold text-amber-400">bun</div>
                    <div className="text-[11px] text-white/60 mt-1">bun.lock</div>
                    <div className="text-[10px] text-acid mt-1">✓ Roundtrip</div>
                  </div>
                </div>

                <div className="space-y-2 pt-2 border-t border-white/10">
                  <div className="flex justify-between text-xs">
                    <span className="text-white/70">1168 packages warm install:</span>
                    <span className="text-acid font-bold">nub: 346 ms</span>
                  </div>
                  <div className="flex justify-between text-xs text-white/40">
                    <span>bun: 1,896 ms (5.5×)</span>
                    <span>pnpm: 3,453 ms (10×)</span>
                  </div>
                </div>
              </div>
            )}

            {activeFeature === 'node' && (
              <div className="space-y-3 leading-relaxed">
                <div className="text-white/40">// Auto-reads .node-version or .nvmrc</div>
                <div>
                  <span className="text-ember">$</span> <span className="text-white font-semibold">echo 22.12.0 &gt; .node-version</span>
                </div>
                <div>
                  <span className="text-ember">$</span> <span className="text-white font-semibold">nub app.ts</span>
                </div>
                <div className="text-acid">
                  → Node v22.12.0 not found locally. Downloading from nodejs.org...
                </div>
                <div className="text-acid">
                  ✓ Checksum verified (sha256: 8a93ef...)
                </div>
                <div className="text-acid">
                  ✓ Installed in 1.4s
                </div>
                <div className="text-white font-semibold pt-2">
                  Server listening on http://localhost:8080 (running on Node v22.12.0)
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

/* --- 3. Anime.js-style Performance Latency Breakdown Chart --- */
export function AnimePerformanceChart() {
  const [activeSegment, setActiveSegment] = useState<number | null>(null);

  const segments = [
    { label: 'Rust Init', time: 0.8, color: 'bg-ember', textColor: 'text-ember', pct: '2%' },
    { label: 'Oxc Transpile', time: 2.4, color: 'bg-acid', textColor: 'text-acid', pct: '6%' },
    { label: 'Hook Resolver', time: 1.6, color: 'bg-sky', textColor: 'text-sky', pct: '4%' },
    { label: 'Node V8 Engine', time: 38.0, color: 'bg-pink', textColor: 'text-pink', pct: '88%' },
  ];

  const totalTime = 42.8;

  return (
    <div className="w-full max-w-4xl mx-auto rounded-2xl border border-fd-border/70 bg-fd-card/40 p-6 sm:p-8 backdrop-blur">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6">
        <div>
          <span className="text-xs font-mono uppercase tracking-widest text-ember font-semibold">
            Execution Latency
          </span>
          <h3 className="font-display text-xl sm:text-2xl font-semibold tracking-tight text-fd-foreground mt-1">
            Total startup overhead: ~4.8 ms
          </h3>
        </div>
        <div className="flex items-baseline gap-2 font-mono">
          <span className="text-3xl font-bold text-fd-foreground">{totalTime}</span>
          <span className="text-sm text-fd-muted-foreground">ms total startup</span>
        </div>
      </div>

      {/* Stacked Bar Chart (Anime.js signature style) */}
      <div className="h-6 w-full rounded-full overflow-hidden flex bg-fd-muted p-0.5 gap-0.5 shadow-inner">
        {segments.map((seg, i) => (
          <div
            key={seg.label}
            onMouseEnter={() => setActiveSegment(i)}
            onMouseLeave={() => setActiveSegment(null)}
            style={{ width: `${(seg.time / totalTime) * 100}%` }}
            className={`h-full ${seg.color} rounded-sm transition-all cursor-pointer ${
              activeSegment === i ? 'brightness-125 scale-y-110' : 'opacity-90 hover:opacity-100'
            }`}
          />
        ))}
      </div>

      {/* Segment Legend */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 mt-6">
        {segments.map((seg, i) => (
          <div
            key={seg.label}
            onMouseEnter={() => setActiveSegment(i)}
            onMouseLeave={() => setActiveSegment(null)}
            className={`p-3 rounded-xl border transition-all cursor-pointer ${
              activeSegment === i
                ? 'border-fd-foreground/40 bg-fd-card shadow-sm'
                : 'border-transparent hover:bg-fd-muted/30'
            }`}
          >
            <div className="flex items-center gap-2">
              <span className={`w-2.5 h-2.5 rounded-full ${seg.color}`} />
              <span className="text-xs font-mono font-medium text-fd-foreground">{seg.label}</span>
            </div>
            <div className="mt-1.5 font-mono text-sm font-bold text-fd-foreground flex items-baseline justify-between">
              <span>{seg.time} ms</span>
              <span className="text-[11px] font-normal text-fd-muted-foreground">{seg.pct}</span>
            </div>
          </div>
        ))}
      </div>

      <div className="mt-6 pt-4 border-t border-fd-border/60 flex flex-wrap items-center justify-between gap-2 text-xs font-mono text-fd-muted-foreground">
        <span>Runs on stock Node 18+ · Zero patched Node binary</span>
        <span>Compare: tsx 128ms · ts-node 480ms</span>
      </div>
    </div>
  );
}

/* --- 4. Anime.js Quick Links Grid ("Start Animating / Start Building") --- */
export function AnimeQuickLinks() {
  const links = [
    { title: 'TypeScript Runner', href: '/docs/runtime', dot: 'bg-ember', desc: 'TS, JSX, decorators, .env' },
    { title: 'Script Dispatcher', href: '/docs/run', dot: 'bg-acid', desc: '24× faster pnpm run' },
    { title: 'Binary Runner (nubx)', href: '/docs/nubx', dot: 'bg-sky', desc: '19× faster npx execution' },
    { title: 'Package Manager', href: '/docs/pm', dot: 'bg-pink', desc: '5× faster multi-lockfile PM' },
    { title: 'Node Versioning', href: '/docs/node', dot: 'bg-orchid', desc: 'On-demand version manager' },
    { title: 'Watch Mode', href: '/docs/watch', dot: 'bg-ember', desc: 'Dependency-aware restart' },
    { title: 'Benchmarks', href: 'https://github.com/nubjs/nub/blob/main/benchmarks/results.md', dot: 'bg-acid', desc: 'Independent CI measurements' },
    { title: 'Complete Docs', href: '/docs', dot: 'bg-sky', desc: 'Full guides & API reference' },
  ];

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 w-full">
      {links.map((link) => (
        <Link
          key={link.title}
          href={link.href}
          className="group flex flex-col justify-between p-5 rounded-xl border border-fd-border/70 bg-fd-card/50 hover:border-ember/50 hover:bg-fd-card transition-all shadow-sm hover:shadow-md"
        >
          <div>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <span className={`w-2 h-2 rounded-full ${link.dot}`} />
                <h4 className="font-mono text-sm font-semibold text-fd-foreground group-hover:text-ember transition-colors">
                  {link.title}
                </h4>
              </div>
              <svg
                className="w-4 h-4 text-fd-muted-foreground group-hover:text-ember group-hover:translate-x-0.5 transition-all"
                viewBox="0 0 24 24"
                fill="currentColor"
              >
                <polygon points="17.737 11.987 12.5 17.225 11.263 15.987 14.388 12.862 6.5 12.862 6.5 11.112 14.388 11.112 11.263 7.987 12.5 6.75"/>
              </svg>
            </div>
            <p className="mt-2 text-xs text-fd-muted-foreground">
              {link.desc}
            </p>
          </div>
        </Link>
      ))}
    </div>
  );
}

/* --- 5. Anime.js Toolbox 2-Sided Interactive Component --- */
export function AnimeToolboxInteractive() {
  const [selectedTool, setSelectedTool] = useState<number>(0);

  const tools = [
    {
      cmd: 'nub <file>',
      label: 'TypeScript Runner',
      side: 'left',
      accent: 'text-ember',
      borderAccent: 'border-ember/50',
      bgAccent: 'bg-ember/10',
      dot: 'bg-ember',
      desc: 'Transpiles TS, JSX, and decorators in-memory with oxc and executes directly on stock Node. Auto-loads .env and resolves tsconfig paths without external packages.',
      replaces: ['tsx', 'ts-node', 'dotenv', 'tsconfig-paths'],
      snippet: '$ nub src/index.ts\n# oxc transpile in 3ms → runs on Node v26\nServer running at http://localhost:3000',
    },
    {
      cmd: 'nub run',
      label: 'Script Dispatcher',
      side: 'left',
      accent: 'text-acid',
      borderAccent: 'border-acid/50',
      bgAccent: 'bg-acid/10',
      dot: 'bg-acid',
      desc: 'Dispatches package.json scripts in 14.7ms from Rust. Avoids 400ms JS cold-start latency. Full support for --filter and -r monorepo workspaces.',
      replaces: ['npm run', 'pnpm run'],
      snippet: '$ nub run build\n# executes in 14.7ms from Rust\n$ nub -r --filter @pkg/* test',
    },
    {
      cmd: 'nubx',
      label: 'Binary Runner',
      side: 'left',
      accent: 'text-sky',
      borderAccent: 'border-sky/50',
      bgAccent: 'bg-sky/10',
      dot: 'bg-sky',
      desc: 'Walks node_modules/.bin in Rust and execs binaries directly in 11ms. Bypasses intermediate Node process wrapper. Seamless fallback to remote dlx.',
      replaces: ['npx', 'pnpm exec', 'pnpm dlx'],
      snippet: '$ nubx prisma generate\n# resolved local bin in 11ms\n$ nubx eslint .',
    },
    {
      cmd: 'nub install',
      label: 'Package Manager',
      side: 'right',
      accent: 'text-pink',
      borderAccent: 'border-pink/50',
      bgAccent: 'bg-pink/10',
      dot: 'bg-pink',
      desc: 'Embedded aube Rust engine. Autodetects and roundtrips pnpm-lock.yaml, package-lock.json, and bun.lock in place. 5× faster warm installs with GVS content store.',
      replaces: ['pnpm', 'npm', 'yarn'],
      snippet: '$ nub install\n# reads existing pnpm-lock.yaml\n# 1,168 packages linked in 346ms',
    },
    {
      cmd: 'nub node',
      label: 'Version Manager',
      side: 'right',
      accent: 'text-orchid',
      borderAccent: 'border-orchid/50',
      bgAccent: 'bg-orchid/10',
      dot: 'bg-orchid',
      desc: 'Reads .node-version or .nvmrc and downloads verified genuine Node builds from nodejs.org on the fly. Replaces nvm and fnm with zero friction.',
      replaces: ['nvm', 'fnm', 'volta'],
      snippet: '$ echo 22.12.0 > .node-version\n$ nub app.ts\n# installed Node v22.12.0 in 1.4s',
    },
    {
      cmd: 'nub watch',
      label: 'Watch Mode',
      side: 'right',
      accent: 'text-ember',
      borderAccent: 'border-ember/50',
      bgAccent: 'bg-ember/10',
      dot: 'bg-ember',
      desc: 'Dependency-aware auto-restart. Watches entrypoint, transitive imports, .env, and tsconfig changes with automatic sourcemap support.',
      replaces: ['nodemon', 'tsx watch'],
      snippet: '$ nub watch src/server.ts\n# ↺ src/db.ts changed — restarting\nListening on http://localhost:3000',
    },
  ];

  const current = tools[selectedTool];

  return (
    <div className="w-full">
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-center">
        {/* Left labels column (Anime.js toolbox-labels-left) */}
        <div className="lg:col-span-3 flex flex-col gap-2.5">
          {tools.slice(0, 3).map((tool, index) => {
            const isSel = selectedTool === index;
            return (
              <button
                key={tool.cmd}
                type="button"
                onClick={() => setSelectedTool(index)}
                onMouseEnter={() => setSelectedTool(index)}
                className={`w-full text-left p-3.5 rounded-xl border transition-all cursor-pointer ${
                  isSel
                    ? `${tool.borderAccent} ${tool.bgAccent} shadow-sm scale-[1.02]`
                    : 'border-fd-border/70 bg-fd-card/40 hover:bg-fd-card hover:border-fd-foreground/20'
                }`}
              >
                <div className="flex items-center gap-2">
                  <span className={`w-2 h-2 rounded-full ${tool.dot}`} />
                  <span className={`font-mono text-xs font-bold ${isSel ? tool.accent : 'text-fd-foreground'}`}>
                    {tool.cmd}
                  </span>
                </div>
                <div className="text-xs text-fd-muted-foreground mt-1 pl-4">
                  {tool.label}
                </div>
              </button>
            );
          })}
        </div>

        {/* Center Active Detail Showcase Panel */}
        <div className="lg:col-span-6 rounded-2xl border border-fd-border/80 bg-[#0b0a08] p-6 sm:p-7 text-[#ece6d8] shadow-2xl flex flex-col justify-between min-h-[340px]">
          <div>
            <div className="flex items-center justify-between pb-4 mb-4 border-b border-white/10">
              <span className={`inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-mono font-semibold ${current.bgAccent} ${current.accent} border ${current.borderAccent}`}>
                <span>{current.cmd}</span>
              </span>
              <span className="text-xs font-mono text-white/50">
                Replaces: {current.replaces.join(', ')}
              </span>
            </div>

            <h4 className="text-xl sm:text-2xl font-display font-semibold text-white tracking-tight">
              {current.label}
            </h4>
            <p className="mt-3 text-sm text-white/70 leading-relaxed font-sans">
              {current.desc}
            </p>
          </div>

          <div className="mt-6 pt-4 border-t border-white/10 bg-black/40 p-3.5 rounded-xl font-mono text-xs text-white/90">
            <pre className="whitespace-pre-wrap leading-relaxed">{current.snippet}</pre>
          </div>
        </div>

        {/* Right labels column (Anime.js toolbox-labels-right) */}
        <div className="lg:col-span-3 flex flex-col gap-2.5">
          {tools.slice(3, 6).map((tool, i) => {
            const index = i + 3;
            const isSel = selectedTool === index;
            return (
              <button
                key={tool.cmd}
                type="button"
                onClick={() => setSelectedTool(index)}
                onMouseEnter={() => setSelectedTool(index)}
                className={`w-full text-left p-3.5 rounded-xl border transition-all cursor-pointer ${
                  isSel
                    ? `${tool.borderAccent} ${tool.bgAccent} shadow-sm scale-[1.02]`
                    : 'border-fd-border/70 bg-fd-card/40 hover:bg-fd-card hover:border-fd-foreground/20'
                }`}
              >
                <div className="flex items-center gap-2">
                  <span className={`w-2 h-2 rounded-full ${tool.dot}`} />
                  <span className={`font-mono text-xs font-bold ${isSel ? tool.accent : 'text-fd-foreground'}`}>
                    {tool.cmd}
                  </span>
                </div>
                <div className="text-xs text-fd-muted-foreground mt-1 pl-4">
                  {tool.label}
                </div>
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}

