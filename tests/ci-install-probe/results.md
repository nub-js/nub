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

_pending_
