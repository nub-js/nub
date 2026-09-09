'use client';

import { useEffect, useRef } from 'react';
import { usePathname } from 'next/navigation';

/* The landing page is FORCED dark (the redesign is a dark-theme spec), while
   docs/blog keep the light/dark switch. The shared RootProvider resolves the
   theme to the user's system preference; this component overrides that on the
   home route by mutating the <html> class + colorScheme DIRECTLY — no
   localStorage write, no next-themes state change — so navigating to docs
   restores their preference untouched.

   First paint is covered by an inline <script> in (home)/layout.tsx (it runs
   during HTML parse, before paint, so system-light users see no flash); this
   effect covers client-side navigation in both directions. */
export function ForceDarkHome() {
  const pathname = usePathname();
  const active = useRef(false);

  useEffect(() => {
    const root = document.documentElement;
    const isHome = pathname === '/';
    if (isHome && !active.current) {
      root.classList.add('dark');
      root.style.colorScheme = 'dark';
      active.current = true;
    } else if (!isHome && active.current) {
      root.classList.remove('dark');
      root.style.colorScheme = '';
      active.current = false;
    }
  }, [pathname]);

  return null;
}
