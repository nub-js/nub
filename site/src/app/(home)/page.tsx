import Link from 'next/link';
import { getLatestNode } from '@/lib/node-version';
import { START_PROMPT } from '@/lib/start-prompt';
import { MigrationPrompt, ViewRepoLink } from '@/components/migration-prompt';
import { StarNudge } from '@/components/star-nudge';
import { SmoothScroll } from '@/components/smooth-scroll';
import {
  HeroInstallCommand,
  HeroQuickCommands,
  HeroInteractiveStudio,
  RealWorldProductionParallax,
  CrossRuntimeCompatBenchmark,
  ModernApisGrid,
  ComplexityCurveChart,
  AsymmetricBentoGrid,
  PMConfigMatrixTable,
  CommunityTestimonials,
  FaqAccordion,
  PreFooterCTA,
} from '@/components/landing-showcase';

export default async function HomePage() {
  const node = await getLatestNode();

  return (
    <div className="relative w-full overflow-x-clip bg-[#0c0a09] text-zinc-100 selection:bg-ember selection:text-white font-sans">
      {/* Silky smooth momentum scrolling powered by Lenis */}
      <SmoothScroll />

      {/* Handwritten "Leave a star" annotation that swoops up to the nav's GitHub star pill */}
      <StarNudge />

      {/* Top Ambient Hero Radial Glow & Subtle Grid Pattern with Nub Ember & Acid Accents */}
      <div
        aria-hidden
        className="pointer-events-none absolute inset-x-0 top-0 h-[750px] opacity-60 z-0"
        style={{
          background: 'radial-gradient(ellipse 60% 50% at 50% 0%, rgba(255, 93, 59, 0.16), rgba(79, 225, 115, 0.05) 40%, transparent 70%)',
        }}
      />
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 effect-grid-bg opacity-20 z-0"
      />

      {/* ----------------------------------------------------------- 1. MONUMENTAL HERO */}
      <section className="relative z-10 pt-16 pb-20 md:pt-24 md:pb-28 border-b border-[#2e2a25]">
        <div className="w-[85%] mx-auto text-center">
          {/* Eyebrow / Release Pill */}
          <Link
            href="/blog/introducing-nub"
            className="group inline-flex items-center gap-2 rounded-full border border-[#38332c] bg-[#141210] py-1 pl-3 pr-4 text-[11px] font-mono tracking-wider uppercase text-zinc-400 hover:border-ember/60 hover:text-white transition-all shadow-sm mb-8"
          >
            <span className="inline-block h-2 w-2 rounded-full bg-ember animate-pulse" />
            <span className="text-zinc-200 font-semibold">Nub 0.7</span>
            <span className="text-zinc-600">—</span>
            <span className="truncate">Release Notes</span>
            <span aria-hidden className="text-zinc-500 group-hover:translate-x-0.5 transition-transform">
              →
            </span>
          </Link>

          {/* Sharp & Curvy Headline: Bricolage Grotesque + Newsreader Italic */}
          <h1 className="font-display text-4xl sm:text-6xl md:text-[68px] font-bold tracking-tight leading-[1.06] text-white text-balance max-w-5xl mx-auto">
            The all-in-one toolkit that{' '}
            <span className="font-serif italic font-normal text-ember">augments</span>{' '}
            Node.js
            <span className="text-ember">.</span>
          </h1>

          {/* Subtitle */}
          <p className="mx-auto mt-6 max-w-3xl text-pretty text-sm sm:text-base md:text-lg text-zinc-400 leading-relaxed font-normal">
            A TypeScript-first toolchain for Node.js. Run TypeScript files, package.json scripts, and local CLIs on stock Node with zero config, zero lock-in, and 24× faster performance.
          </p>

          {/* Install Command Bar */}
          <div className="mt-10">
            <HeroInstallCommand />
          </div>

          {/* Agent migration & GitHub Links */}
          <div className="mt-6 flex flex-wrap items-center justify-center gap-x-6 gap-y-2 text-xs font-mono text-zinc-500">
            <MigrationPrompt prompt={START_PROMPT} />
            <span className="text-zinc-800 hidden sm:inline">|</span>
            <ViewRepoLink />
          </div>

          {/* Quick 7-Command Overview Deck */}
          <HeroQuickCommands />

          {/* Tabbed Interactive Studio */}
          <HeroInteractiveStudio />
        </div>
      </section>

      {/* ----------------------------------------------------------- 2. REAL PRODUCTION SYSTEMS (Scroll-locked Horizontal Parallax) */}
      <section className="relative z-10 border-b border-[#2e2a25] bg-[#0e0c0a]/50">
        <RealWorldProductionParallax />
      </section>

      {/* ----------------------------------------------------------- 3. CROSS-RUNTIME NODE COMPATIBILITY & DIRECT TS */}
      <section className="relative z-10 py-20 md:py-28 border-b border-[#2e2a25]">
        <div className="w-[85%] mx-auto">
          <CrossRuntimeCompatBenchmark />
        </div>
      </section>

      {/* ----------------------------------------------------------- 4. FORWARD COMPATIBILITY & MODERN APIS */}
      <section className="relative z-10 py-20 md:py-28 border-b border-[#2e2a25] bg-[#0e0c0a]/50">
        <div className="w-[85%] mx-auto">
          <ModernApisGrid />
        </div>
      </section>

      {/* ----------------------------------------------------------- 5. COMPLEXITY VS SCALE & ASYMMETRIC BENTO GRID */}
      <section className="relative z-10 py-20 md:py-28 border-b border-[#2e2a25]">
        <div className="w-[85%] mx-auto">
          {/* Complexity Graph & Hyperfine Data */}
          <ComplexityCurveChart />

          {/* Asymmetric Bento Grid (Different Colsize & Rowsize) */}
          <div className="mt-16">
            <div className="mb-8 text-left">
              <span className="text-xs font-mono font-semibold uppercase tracking-wider text-ember flex items-center gap-1.5 mb-1.5">
                <span className="h-1.5 w-1.5 rounded-full bg-ember" />
                Asymmetric Architecture
              </span>
              <h3 className="font-display text-2xl sm:text-3xl font-bold tracking-tight text-white">
                Engineered for maximum developer velocity
              </h3>
            </div>
            <AsymmetricBentoGrid />
          </div>
        </div>
      </section>

      {/* ----------------------------------------------------------- 6. PACKAGE MANAGER CONFIG COMPATIBILITY MATRIX */}
      <section className="relative z-10 py-20 md:py-28 border-b border-[#2e2a25] bg-[#0e0c0a]/50">
        <div className="w-[85%] mx-auto">
          <PMConfigMatrixTable />
        </div>
      </section>

      {/* ----------------------------------------------------------- 7. COMMUNITY & TESTIMONIALS */}
      <section className="relative z-10 py-20 md:py-28 border-b border-[#2e2a25]">
        <div className="w-[85%] mx-auto">
          <div className="mb-10 text-left">
            <span className="text-xs font-mono font-semibold uppercase tracking-wider text-pink flex items-center gap-1.5 mb-2">
              <span className="h-1.5 w-1.5 rounded-full bg-pink" />
              Open Source Community
            </span>
            <h2 className="font-display text-3xl sm:text-4xl font-bold tracking-tight text-white">
              What developers are saying...
            </h2>
            <p className="mt-2 text-sm sm:text-base text-zinc-400">
              Verified feedback from Hacker News, GitHub discussions, and open-source contributors.
            </p>
          </div>

          <CommunityTestimonials />
        </div>
      </section>

      {/* ----------------------------------------------------------- 8. FAQ ACCORDION */}
      <section className="relative z-10 py-20 md:py-28 border-b border-[#2e2a25] bg-[#0e0c0a]/50">
        <div className="w-[85%] mx-auto">
          <FaqAccordion />
        </div>
      </section>

      {/* ----------------------------------------------------------- 9. PRE-FOOTER CTA */}
      <section className="relative z-10 py-20 md:py-28 border-b border-[#2e2a25]">
        <PreFooterCTA />
      </section>

      {/* ----------------------------------------------------------- 7. 5-COLUMN FOOTER */}
      <footer className="relative z-10 py-16 bg-[#0c0a09]">
        <div className="w-[85%] mx-auto">
          <div className="grid grid-cols-2 md:grid-cols-5 gap-8 pb-12 border-b border-[#2e2a25]">
            {/* Brand column */}
            <div className="col-span-2 text-left">
              <div className="flex items-center gap-1.5 font-bold text-2xl tracking-tight text-white">
                <span>nub</span>
                <span className="text-ember">.</span>
                <span className="ml-2 rounded-full border border-[#38332c] bg-[#141210] px-2 py-0.5 text-[10px] font-mono text-zinc-400 font-normal">
                  v0.7 · Node v{node.major}
                </span>
              </div>
              <p className="mt-3 text-xs text-zinc-400 leading-relaxed max-w-sm">
                A TypeScript-first developer supertool for Node.js. In-memory oxc transpilation, 24× script dispatch, and lockfile-neutral package management.
              </p>
              <div className="mt-4 text-xs font-mono text-zinc-500">
                Released under the MIT License.
              </div>
            </div>

            {/* Documentation */}
            <div className="flex flex-col gap-2.5 text-xs font-mono text-left">
              <span className="font-semibold text-zinc-200 mb-1">Documentation</span>
              <Link href="/docs" className="text-zinc-400 hover:text-white transition-colors">Getting Started</Link>
              <Link href="/docs/commands/run" className="text-zinc-400 hover:text-white transition-colors">TypeScript Runner</Link>
              <Link href="/docs/commands/watch" className="text-zinc-400 hover:text-white transition-colors">Watch Mode</Link>
              <Link href="/docs/commands/install" className="text-zinc-400 hover:text-white transition-colors">Package Manager</Link>
              <Link href="/docs/configuration" className="text-zinc-400 hover:text-white transition-colors">Configuration</Link>
            </div>

            {/* Ecosystem */}
            <div className="flex flex-col gap-2.5 text-xs font-mono text-left">
              <span className="font-semibold text-zinc-200 mb-1">Ecosystem</span>
              <Link href="/docs/commands/nubx" className="text-zinc-400 hover:text-white transition-colors">nubx (19× npx)</Link>
              <Link href="/docs/commands/node-versions" className="text-zinc-400 hover:text-white transition-colors">Node Versioning</Link>
              <Link href="/docs/benchmarks" className="text-zinc-400 hover:text-white transition-colors">Benchmarks</Link>
              <Link href="/blog" className="text-zinc-400 hover:text-white transition-colors">Blog &amp; Releases</Link>
            </div>

            {/* Community */}
            <div className="flex flex-col gap-2.5 text-xs font-mono text-left">
              <span className="font-semibold text-zinc-200 mb-1">Community</span>
              <a href="https://github.com/nubjs/nub" target="_blank" rel="noopener noreferrer" className="text-zinc-400 hover:text-white transition-colors">GitHub</a>
              <a href="https://github.com/nubjs/nub/discussions" target="_blank" rel="noopener noreferrer" className="text-zinc-400 hover:text-white transition-colors">Discussions</a>
              <a href="https://github.com/nubjs/nub/issues" target="_blank" rel="noopener noreferrer" className="text-zinc-400 hover:text-white transition-colors">Issues</a>
              <a href="https://github.com/nubjs/nub/blob/main/LICENSE" target="_blank" rel="noopener noreferrer" className="text-zinc-400 hover:text-white transition-colors">MIT License</a>
            </div>
          </div>

          <div className="mt-8 flex flex-col sm:flex-row items-center justify-between gap-4 text-xs font-mono text-zinc-500">
            <div>
              © {new Date().getFullYear()} Nub. Built for the modern Node.js ecosystem.
            </div>
            <div className="flex items-center gap-4">
              <span>TypeScript 5.x</span>
              <span>•</span>
              <span>Node.js v{node.major} LTS</span>
              <span>•</span>
              <span>Zero Lock-In</span>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}
