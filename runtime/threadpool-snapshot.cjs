// The compat tier (Node 18.19–22.14, 23.0–23.4) loads nub's preload with `--import`,
// and Node's ESM loader reads that file through libuv's threadpool, so the pool
// already exists when preload.mjs runs. libuv before 1.50 (Node ≤ 22.21) gives its
// workers no thread name, which leaves the thread ids that appear when the pool is
// built as the only way to tell the workers apart — measured on Node 18.19, 20.11 and
// 22.13: 7 threads when a `--require` preload runs, 15 once the pool exists, and 15
// already when an `--import` preload's body runs. Node runs every `--require`
// before it loads any `--import`, so this records the thread ids that precede the
// pool for preload-common.cjs (installThreadpoolPolicy) to diff against. The fast
// tier's `--require preload.cjs` takes the same snapshot itself. The launcher adds
// this token only on Linux, and only when it sized the pool.
const SNAPSHOT = Symbol.for("nub.threadpool.snapshot");
if (process.platform === "linux" && process[SNAPSHOT] === undefined) {
  try {
    const { readdirSync } = require("node:fs");
    process[SNAPSHOT] = new Set(readdirSync("/proc/self/task").map(Number));
  } catch {}
}
