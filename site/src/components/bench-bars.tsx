'use client';

import { useEffect, useRef, useState } from 'react';

/* Horizontal benchmark bars, redesigned as the page's chart primitive: the
   fastest row fills in the single accent color, competitors in muted gray,
   right-aligned time + multiplier labels, and the fill animates 0 → width on
   scroll into view (respects prefers-reduced-motion). Kept signature-compatible
   with the old server BenchBars so blog MDX (<Bench>) keeps working; the accent
   prop is accepted but every row resolves to the one accent color. */

export type BenchRow = {
  cmd: string;
  ms: number;
  ratio?: number | null;
  label?: string;
  us?: boolean;
};

export function BenchBars({
  rows,
  max,
  unit = 'ms',
}: {
  rows: BenchRow[];
  max: number;
  /* @deprecated — the redesign uses a single accent everywhere. */
  accent?: 'ember' | 'acid' | 'sky' | 'pink';
  unit?: string;
}) {
  const ref = useRef<HTMLDivElement | null>(null);
  const [on, setOn] = useState(false);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
      setOn(true);
      return;
    }
    const io = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setOn(true);
          io.disconnect();
        }
      },
      { threshold: 0.3 },
    );
    io.observe(el);
    return () => io.disconnect();
  }, []);

  return (
    <div ref={ref} className="space-y-4">
      {rows.map((r, i) => (
        <div key={r.cmd}>
          <div className="mb-1.5 flex items-baseline justify-between gap-4">
            <span
              className={`font-mono text-sm ${r.us ? 'font-semibold text-ember' : 'nub-code-fg'}`}
            >
              {r.cmd}
            </span>
            <span className="nub-code-muted shrink-0 font-mono text-xs tabular-nums">
              {r.ms} {unit}
              {r.label
                ? `  ·  ${r.label}`
                : r.ratio
                  ? `  ·  ${r.ratio}× slower`
                  : ''}
            </span>
          </div>
          <div className="nub-code-track h-2.5 overflow-hidden rounded-full">
            <div
              className={`nub-bench-bar h-full rounded-full ${r.us ? 'bg-ember' : 'nub-code-bar-muted'}`}
              style={{
                width: on ? `${Math.max((r.ms / max) * 100, 3)}%` : '0%',
                transitionDelay: `${i * 90}ms`,
              }}
            />
          </div>
        </div>
      ))}
    </div>
  );
}
