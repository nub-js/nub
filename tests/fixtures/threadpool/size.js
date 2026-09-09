// Emits the threadpool size this process runs with, and the parallelism available
// to it (below the host's core count under a cgroup quota), as JSON so the test
// can assert the sizing rule without text-matching.
//
// `size` is the value Node read at startup: the variable itself when the user set
// it, else nub's ownership marker, because the preload deletes nub's own value
// from `process.env` so children get Node's default (`env` shows what a child
// would inherit). Null under plain Node (`--node` / `NODE_COMPAT`). On Linux,
// `workers` counts libuv's worker threads after one pool use and `nices` lists
// their nice values in creation order, which is the pool the process really has.
const fs = require("node:fs");
const os = require("node:os");
const out = {
  size: process.env.UV_THREADPOOL_SIZE ?? process.env.__NUB_AUGMENTED_UV_THREADPOOL_SIZE ?? null,
  env: process.env.UV_THREADPOOL_SIZE ?? null,
  cores: os.availableParallelism(),
};
if (process.platform === "linux") {
  fs.stat("/", () => {});
  const nices = [];
  for (const d of fs.readdirSync("/proc/self/task").map(Number).filter(Boolean).sort((a, b) => a - b)) {
    let comm = "";
    try {
      comm = fs.readFileSync(`/proc/self/task/${d}/comm`, "latin1").trim();
    } catch {}
    if (comm !== "libuv-worker") continue;
    const stat = fs.readFileSync(`/proc/self/task/${d}/stat`, "latin1");
    nices.push(Number(stat.slice(stat.lastIndexOf(")") + 2).split(" ")[16]));
  }
  out.workers = nices.length;
  out.nices = nices;
}
process.stdout.write(JSON.stringify(out) + "\n");
