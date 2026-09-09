/* Single source of truth for the Node version shown in the homepage mockups.
   Fetches the live release index from nodejs.org and returns the newest release,
   so the "Using Node.js X" / "node vX" / "install <major>" strings track reality
   instead of drifting. Next ISR (`revalidate`) refreshes it daily with no redeploy;
   the fetch is memoized within a render, so calling this from several components
   hits the network once. Falls back to a pinned recent version if the fetch fails
   (offline build, nodejs.org hiccup) so the build never breaks. */

export type NodeVersion = { full: string; major: string };

const FALLBACK: NodeVersion = { full: '26.3.0', major: '26' };

let cached: NodeVersion | null = null;

export async function getLatestNode(): Promise<NodeVersion> {
  if (cached) return cached;
  if (process.env.NODE_ENV === 'development') {
    cached = FALLBACK;
    return FALLBACK;
  }
  try {
    const res = await fetch('https://nodejs.org/dist/index.json', {
      next: { revalidate: 86400 },
      signal: AbortSignal.timeout(1500),
    });
    if (!res.ok) return FALLBACK;
    const all = (await res.json()) as { version: string }[];
    const full = all[0]?.version?.replace(/^v/, '');
    if (!full) return FALLBACK;
    cached = { full, major: full.split('.')[0] };
    return cached;
  } catch {
    return FALLBACK;
  }
}
