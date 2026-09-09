'use client';

import React, { useEffect, useState } from 'react';

type Chapter = {
  id: string;
  label: string;
  color: string;
};

const CHAPTERS: Chapter[] = [
  { id: 'intro', label: 'INTRO', color: 'bg-ember' },
  { id: 'toolbox', label: 'TOOLBOX', color: 'bg-acid' },
  { id: 'features', label: 'FEATURES', color: 'bg-sky' },
  { id: 'latency', label: 'LATENCY', color: 'bg-pink' },
  { id: 'guarantee', label: 'LOCK-IN', color: 'bg-orchid' },
  { id: 'get-started', label: 'START', color: 'bg-ember' },
];

export function AnimeChapterNav() {
  const [activeChapter, setActiveChapter] = useState<string>('intro');
  const [scrollProgress, setScrollProgress] = useState(0);

  useEffect(() => {
    const handleScroll = () => {
      const scrollY = window.scrollY;
      const totalHeight = document.documentElement.scrollHeight - window.innerHeight;
      if (totalHeight > 0) {
        setScrollProgress(Math.min(1, Math.max(0, scrollY / totalHeight)));
      }

      // Check which section is in viewport
      for (let i = CHAPTERS.length - 1; i >= 0; i--) {
        const el = document.getElementById(CHAPTERS[i].id);
        if (el) {
          const rect = el.getBoundingClientRect();
          if (rect.top <= window.innerHeight * 0.45) {
            setActiveChapter(CHAPTERS[i].id);
            break;
          }
        }
      }
    };

    window.addEventListener('scroll', handleScroll, { passive: true });
    handleScroll();

    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  const scrollTo = (id: string) => {
    const el = document.getElementById(id);
    if (el) {
      el.scrollIntoView({ behavior: 'smooth' });
    }
  };

  const activeIndex = CHAPTERS.findIndex((c) => c.id === activeChapter);

  return (
    <aside
      aria-label="Chapter navigation"
      className="fixed right-6 top-1/2 -translate-y-1/2 z-40 hidden xl:flex flex-col items-end gap-3 pointer-events-auto select-none"
    >
      <div className="relative flex flex-col items-end gap-2 pr-4 border-r border-fd-border/70 py-2">
        {/* Animated Scroll Track Cursor (Anime.js signature style) */}
        <div
          className="absolute -right-[2.5px] top-0 w-[4px] bg-ember rounded-full transition-all duration-200 pointer-events-none"
          style={{
            height: '24px',
            transform: `translateY(${Math.max(0, activeIndex * 30)}px)`,
          }}
        />

        {CHAPTERS.map((ch, idx) => {
          const isActive = activeChapter === ch.id;
          return (
            <button
              key={ch.id}
              type="button"
              onClick={() => scrollTo(ch.id)}
              className="group flex items-center gap-2.5 py-1 text-right cursor-pointer"
            >
              <span
                className={`font-mono text-[10px] tracking-[0.2em] font-medium transition-all duration-150 ${
                  isActive
                    ? 'text-fd-foreground font-bold translate-x-0 opacity-100'
                    : 'text-fd-muted-foreground/60 hover:text-fd-foreground translate-x-1 opacity-60 hover:opacity-100'
                }`}
              >
                {ch.label}
              </span>
              <span
                className={`w-1.5 h-1.5 rounded-full transition-all duration-200 ${
                  isActive ? `${ch.color} scale-125` : 'bg-fd-border group-hover:bg-fd-foreground/40'
                }`}
              />
            </button>
          );
        })}
      </div>
    </aside>
  );
}
