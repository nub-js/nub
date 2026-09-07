// Emits libuv's configured threadpool size and the parallelism available to the
// process (below the host's core count under a cgroup quota) as JSON so the test
// can assert the sizing rule without text-matching. `size` is null when the
// variable is absent (plain Node: `--node` / `NODE_COMPAT`).
const os = require("node:os");
const out = {
  size: process.env.UV_THREADPOOL_SIZE ?? null,
  cores: os.availableParallelism(),
};
process.stdout.write(JSON.stringify(out) + "\n");
