# ci-install-probe — results

Fixture: [`tests/bench/install/fixtures/large`](../bench/install/fixtures/large), 1,168 packages (the lockfiles resolve 1,152–1,156 tarballs, 58.5–58.9 MB). Runner: GitHub-hosted `ubuntu-latest` (ubuntu-24.04, 4 vCPU). Node 24. Tool versions: npm on Node 24, pnpm 10 (`pnpm/action-setup@v5`), bun (`oven-sh/setup-bun@v2`, latest), nub 0.8.3 (`nubjs/setup-nub@v0`). One job per row; wall-clock in milliseconds from `date +%s%3N` around the install command, step durations from the jobs API (1s resolution).

## Run 1 — cache miss (every `*-cache` row saves in its post-step)

Run [34141195502](https://github.com/nubjs/nub/actions/runs/34141195502).

| row | tool setup step | first install (cold `node_modules`) | second install (store warm, `node_modules` wiped) | cache restore | cache save (post-step) |
| --- | --- | --- | --- | --- | --- |
| npm-nocache | 1s | 80,537 ms | 14,463 ms | — | — |
| npm-cache | 6s (includes the miss) | 70,845 ms | 13,070 ms | miss | 2s |
| pnpm-nocache | 2s + 3s | 8,586 ms | 3,774 ms | — | — |
| pnpm-cache | 2s + 5s (includes the miss) | 8,314 ms | 4,033 ms | miss | 5s |
| bun-nocache | 1s | 10,062 ms | 1,996 ms | — | — |
| nub-nocache | 5s | 7,036 ms | 1,631 ms | — | — |
| nub-cache | 5s | 6,742 ms | 1,672 ms | miss | 6s |
| nub-npmlock-nocache (reads `package-lock.json`) | 4s | 10,232 ms | 1,576 ms | — | — |
| nub-pnpmlock-nocache (reads `pnpm-lock.yaml`) | 4s | 7,705 ms | 1,972 ms | — | — |
| nub-nocache-c64 (`NUB_CONCURRENCY=64`) | 9s | 6,954 ms | 1,669 ms | — | — |
| nub-nocache-c128 (`NUB_CONCURRENCY=128`) | 6s | 7,777 ms | 1,341 ms | — | — |

Notes on run 1:

- The `Setup nub` step is `npm install -g @nubjs/nub@0.8.3` plus a Node 24 provision; it ranged 4–9s across rows. A release-tarball install of the same version (`curl | tar` of `nub-linux-x64.tar.gz`) took 715–992 ms in every nub row.
- Nub's fetch phase reported `downloaded 1152 (58.9 MB) in 6.5–7.4s` at the default fan-out (16 on a 4-vCPU runner) and did not change at 64 or 128, so the fan-out is not the limit on this runner.
- The `second install` column is a relink against a warm store on the same runner; `npm ci` re-extracts every tarball from `~/.npm`, which is why its warm number stays above 13s.

## Run 2 — cache hit

Run [34141508678](https://github.com/nubjs/nub/actions/runs/34141508678). Same matrix; the `*-cache` rows restored the cache run 1 saved.

| row | tool setup step (includes the restore for cache rows) | first install | second install | cache restore | cache save |
| --- | --- | --- | --- | --- | --- |
| npm-nocache | 3s | 81,562 ms | 14,561 ms | — | — |
| npm-cache | 9s (restore hit) | 20,996 ms | 18,919 ms | hit | 1s (no-op) |
| pnpm-nocache | 1s + 1s | 8,813 ms | 4,119 ms | — | — |
| pnpm-cache | 1s + 4s (restore hit) | 4,410 ms | 3,853 ms | hit | 0s |
| bun-nocache | 2s | 3,538 ms | 872 ms | — | — |
| nub-nocache | 5s | 8,167 ms | 1,894 ms | — | — |
| nub-cache | 10s (restore hit) | 1,524 ms | 1,589 ms | hit | 1s (no-op) |
| nub-npmlock-nocache | 5s | 8,705 ms | 1,934 ms | — | — |
| nub-pnpmlock-nocache | 4s | 6,886 ms | 1,681 ms | — | — |
| nub-nocache-c64 | 7s | 8,385 ms | 1,547 ms | — | — |
| nub-nocache-c128 | 4s | 8,883 ms | 1,637 ms | — | — |

Notes on run 2:

- A restored npm cache takes `npm ci` from 81.6 s to 21.0 s, at the price of a 9 s restore inside the setup step; the warm-cache `npm ci` still re-extracts every tarball (18.9 s on the second install).
- A restored nub store takes the install to 1.5 s, at the price of a ~5 s restore (the setup step went from 5 s to 10 s). Net against the no-cache row: 11.5 s versus 13.2 s for setup plus install, with a 6 s save on every miss.
- bun's cold install measured 10.1 s in run 1 and 3.5 s here. Nub's ten cold samples across both runs (all lockfile and concurrency variants) span 6.7–10.2 s with a median of 7.8 s; pnpm's three span 8.3–8.8 s. The bun spread is why the matrix gained three extra cold rows for each of bun and nub (run 3).
- Nub's fetch phase again reported 6.6–8.6 s for 58.4–58.9 MB regardless of `NUB_CONCURRENCY`.

## Run 3 — extra cold samples

_pending_
