# ci-install-probe

Measures the dependency-install cost on a GitHub-hosted `ubuntu-latest` runner for `npm ci`, `pnpm install --frozen-lockfile`, `bun install --frozen-lockfile`, and `nub install --frozen-lockfile` on one fixture, [`tests/bench/install/fixtures/large`](../bench/install/fixtures/large) (1,168 packages, all four lockfiles committed), with and without the `actions/cache` round trip that `setup-node` / `setup-nub` perform.

Branch-scoped: pushing to `ci-install-probe` runs [`.github/workflows/ci-install-probe.yml`](../../.github/workflows/ci-install-probe.yml). No pull request.

## Protocol

1. Push once. Every `*-cache` row misses its cache and saves it in the post-step, so this run measures the cold path plus the save cost.
2. Push an empty commit. The `*-cache` rows now restore, so this run measures the restore-and-warm path.
3. `harvest.sh <run-id>` prints every step's wall-clock from the jobs API (cache save/restore live in steps the workflow cannot time from inside) and the `INSTALL_MS` / `SECOND_INSTALL_MS` lines each install step echoed.

Each row also wipes `node_modules` and installs a second time against the now-warm store, which is the number a runner with a persistent disk would see, and the `nub` rows time a release-tarball install of `nub` beside the `npm install -g` the action performs.

`nub-nocache-c64` and `nub-nocache-c128` override the tarball-fetch fan-out (`NUB_CONCURRENCY`; the default on a 4-vCPU runner is 16) to find where the runner's registry throughput tops out.

Findings: `results.md`.
