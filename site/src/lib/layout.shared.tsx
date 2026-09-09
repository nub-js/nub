import type { ReactNode } from 'react';
import type {
  BaseLayoutProps,
  LinkItemType,
} from 'fumadocs-ui/layouts/shared';
import { GitHubStarPill } from '@/components/github-star-pill';

/* The wordmark — stylized with a trailing period as a logo.
   It's text-lg against the text-sm nav links, and the nav row shares a baseline,
   so the larger glyphs put the wordmark's optical center ~0.75px ABOVE the nav's
   — it reads as riding high even though the baselines are mathematically aligned.
   Nudge it down that 0.75px (relative, no layout shift) to optically center it
   with Docs/Blog. (Pure optics; the baselines were already identical.) */
export function Wordmark() {
  return (
    <span className="relative top-[0.75px] font-display text-lg font-medium tracking-tight text-fd-foreground">
      nub<span className="text-ember">.</span>
    </span>
  );
}

/* The GitHub entry, rendered as a star-button pill pinned to the nav's secondary
   (top-right) slot. Replaces fumadocs' default `githubUrl` icon so there's no
   duplicate GitHub control. Exported so the docs/guides layouts — which drop the
   primary nav links — can still surface it on their own headers.

   The top bar stays CLEAN: the optional "Leave a star" nudge does NOT live here.
   It is an absolutely-positioned annotation overlaid on the HOME hero (see
   `StarNudge`, mounted in the hero), so the bar is never widened by it. */
export function githubPillLink(): LinkItemType {
  return {
    type: 'custom',
    secondary: true,
    children: <GitHubStarPill repo="nubjs/nub" />,
  };
}

/* The nav links (Docs/Blog) are 14px; the wordmark is 18px. Their line boxes
   center identically, but the ink CENTER-OF-MASS does not — the larger wordmark's
   glyph mass sits lower, so flush-centered links read as riding high next to it.
   Measured via an alpha-weighted-centroid rasterization of each label: wordmark
   COM 34.42px vs unnudged links 32.10px. Wrapping the label in this relative span
   is itself worth ~1.5px of the correction (it re-boxes the flex child); the
   remaining top:0.8px lands the links' COM at 34.40 — one optical center. */
function NavLabel({ children }: { children: ReactNode }) {
  return <span className="relative top-[0.8px]">{children}</span>;
}

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: <Wordmark />,
    },
    links: [
      { text: <NavLabel>Docs</NavLabel>, url: '/docs', active: 'nested-url' },
      { text: <NavLabel>Toolbox</NavLabel>, url: '/#toolbox' },
      { text: <NavLabel>Features</NavLabel>, url: '/#features' },
      { text: <NavLabel>Benchmarks</NavLabel>, url: '/#latency' },
      { text: <NavLabel>Blog</NavLabel>, url: '/blog', active: 'nested-url' },
      githubPillLink(),
    ],
    themeSwitch: { enabled: true },
  };
}
