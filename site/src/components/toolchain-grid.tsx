import Link from 'next/link';
import type { ReactNode } from 'react';

/* The new "Toolchain Overview" section: a scannable icon-card grid, one card
   per tool, each anchor-linking down to its detailed band (the reference's
   fast-comprehension grid pattern). Icons are simple 24px line glyphs — no
   illustration, stays developer-tool credible. */

type Tool = {
  href: string;
  icon: ReactNode;
  title: string;
  command: string;
  blurb: string;
  replaces: string[];
};

const stroke = {
  fill: 'none',
  stroke: 'currentColor',
  strokeWidth: 1.75,
  strokeLinecap: 'round',
  strokeLinejoin: 'round',
} as const;

const TOOLS: Tool[] = [
  {
    href: '#file-runner',
    title: 'File runner',
    command: 'nub index.ts',
    blurb: 'TypeScript, JSX, decorators on stock Node — no build step.',
    replaces: ['tsx', 'ts-node'],
    icon: (
      <svg viewBox="0 0 24 24" width="22" height="22" {...stroke} aria-hidden>
        <path d="M4 3h10l6 6v12H4z" />
        <path d="M14 3v6h6" />
        <path d="M9 13h6M9 17h4" />
      </svg>
    ),
  },
  {
    href: '#script-runner',
    title: 'Script runner',
    command: 'nub run build',
    blurb: 'A drop-in npm run / pnpm run with zero JS startup.',
    replaces: ['npm run', 'pnpm run'],
    icon: (
      <svg viewBox="0 0 24 24" width="22" height="22" {...stroke} aria-hidden>
        <path d="M8 3v18" />
        <path d="M8 5h9a2 2 0 0 1 0 4H8" />
        <path d="M8 13h6a2 2 0 0 1 0 4H8" />
      </svg>
    ),
  },
  {
    href: '#package-runner',
    title: 'Package runner',
    command: 'nubx eslint .',
    blurb: 'Local .bin resolution in Rust — a 19× faster npx.',
    replaces: ['npx', 'pnpm exec'],
    icon: (
      <svg viewBox="0 0 24 24" width="22" height="22" {...stroke} aria-hidden>
        <path d="M4 14v-3a8 8 0 0 1 16 0v3" />
        <path d="M2 14h4v6H2zM18 14h4v6h-4z" />
        <path d="M12 9v3" />
      </svg>
    ),
  },
  {
    href: '#package-manager',
    title: 'Package manager',
    command: 'nub install',
    blurb: 'pnpm-compatible installs against your existing lockfile.',
    replaces: ['pnpm install', 'npm install', 'corepack'],
    icon: (
      <svg viewBox="0 0 24 24" width="22" height="22" {...stroke} aria-hidden>
        <path d="m12 3 8 4.5v9L12 21l-8-4.5v-9z" />
        <path d="M12 12 4 7.5M12 12l8-4.5M12 12v9" />
      </svg>
    ),
  },
  {
    href: '#version-manager',
    title: 'Node version manager',
    command: 'nub node install',
    blurb: 'Auto-installs the pinned Node from .node-version or .nvmrc.',
    replaces: ['nvm', 'fnm'],
    icon: (
      <svg viewBox="0 0 24 24" width="22" height="22" {...stroke} aria-hidden>
        <path d="M12 3a6 6 0 0 0-6 6c0 2.4 1.5 4 3.5 5.5S12.8 16.4 12 18c-.8 1.6-1 3-1 3" />
        <path d="M12 3a6 6 0 0 1 6 6c0 2.4-1.5 4-3.5 5.5S11.2 16.4 12 18" />
        <path d="M10 9h4M10.5 12h3" />
      </svg>
    ),
  },
];

export function ToolchainGrid() {
  return (
    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5">
      {TOOLS.map((tool) => (
        <Link
          key={tool.href}
          href={tool.href}
          className="group flex flex-col rounded-2xl border border-fd-border bg-fd-card/60 p-6 transition-colors hover:border-ember/50 hover:bg-fd-card"
        >
          <span className="inline-flex h-11 w-11 items-center justify-center rounded-xl border border-fd-border bg-fd-muted/40 text-fd-muted-foreground transition-colors group-hover:border-ember/40 group-hover:bg-ember/10 group-hover:text-ember">
            {tool.icon}
          </span>
          <h3 className="mt-4 text-base font-semibold tracking-tight text-fd-foreground">
            {tool.title}
          </h3>
          <p className="mt-2 text-sm leading-relaxed text-fd-muted-foreground">
            {tool.blurb}
          </p>
          <code className="mt-4 inline-flex w-fit items-center gap-1.5 rounded-md bg-fd-muted/60 px-2 py-1 font-mono text-xs text-fd-foreground">
            <span aria-hidden className="text-ember">
              $
            </span>
            {tool.command}
          </code>
          <span className="mt-auto flex flex-wrap gap-1.5 pt-4 text-[0.7rem] text-fd-muted-foreground">
            <span className="pt-px uppercase tracking-wider opacity-70">Replaces</span>
            {tool.replaces.map((r) => (
              <span
                key={r}
                className="rounded border border-fd-border bg-fd-card px-1.5 py-0.5 font-mono text-[0.65rem]"
              >
                {r}
              </span>
            ))}
          </span>
        </Link>
      ))}
    </div>
  );
}
