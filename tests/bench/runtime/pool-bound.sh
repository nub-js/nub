#!/bin/bash
# Workloads that queue CPU-heavy work on libuv's threadpool, where Node's fixed 4 threads cap
# throughput at 4 concurrent tasks whatever the core count: password hashing (bcrypt, scrypt,
# pbkdf2 at the OWASP iteration count), sharp thumbnails and gzip of a 1 MB response. Fastify 5 endpoints under autocannon,
# plain `node` (4 threads) against `nub` (the core count). Meant for a box with many cores; on 4 or
# fewer nub sets nothing different (max(4, cores)), so there is nothing to measure there.
#
# Runs at the repo root with NUB_BIN set, which is the `remote-build --job adhoc` contract:
#   nub scripts/remote-build.ts --job adhoc --script tests/bench/runtime/pool-bound.sh --machine c3d-standard-16 --detach
# Every measurement is also printed as one machine-readable `ROW {...}` line.
set -u
echo "NUB_BIN=$NUB_BIN"; "$NUB_BIN" --version
ARCH=$(uname -m); case "$ARCH" in x86_64) NA=x64;; aarch64|arm64) NA=arm64;; *) echo "unknown arch $ARCH"; exit 1;; esac
NV=${NODE_VERSION:-v26.8.1}
ROUNDS=${ROUNDS:-3}
W=$(mktemp -d /tmp/pb.XXXX); cd "$W" || exit 1; pwd; nproc; uptime
curl -fsSL "https://nodejs.org/dist/$NV/node-$NV-linux-$NA.tar.xz" -o node.tar.xz || exit 1
mkdir n && tar -xJf node.tar.xz -C n --strip-components=1 || exit 1
N="$W/n/bin"; "$N/node" --version; NP=$(nproc)
export NODE_NO_WARNINGS=1
cat > package.json <<'EOF'
{ "name": "pool-bound", "private": true, "type": "module" }
EOF
PATH="$N:$PATH" "$N/npm" install --silent --no-audit --no-fund fastify@5 @fastify/compress bcrypt sharp autocannon > npm.log 2>&1 || { echo "npm install failed"; tail -20 npm.log; exit 1; }

cat > server.mjs <<'EOF'
import Fastify from "fastify";
import bcrypt from "bcrypt";
import sharp from "sharp";
import compress from "@fastify/compress";
import { scrypt, pbkdf2 } from "node:crypto";
import { gzip } from "node:zlib";
import { promisify } from "node:util";
const gzipAsync = promisify(gzip);
const app = Fastify({ logger: false });
await app.register(compress, { global: false, threshold: 0 });
// a ~1 MB JSON body, built once and gzipped per request (zlib runs on the threadpool). Serialized
// ahead of time: at 270 KB, serializing on the main thread cost more than the compression, so the
// route measured the event loop and not the pool (16 vCPU: 881 vs 844 req/s).
const payload = Buffer.from(JSON.stringify({ rows: Array.from({ length: 11000 }, (_, i) => ({ id: i, name: "user" + i, email: "user" + i + "@example.com", tags: ["a", "b", "c"], score: i * 1.5 })) }));
app.get("/gzip", { compress: { threshold: 0 } }, async (req, reply) => { reply.header("content-type", "application/json"); return payload; });
// the same body gzipped in one call: one pool task per request, where the streaming compressor
// above queues one task per 16 KB chunk with an event-loop round trip between them
app.get("/gzip-once", async (req, reply) => { reply.header("content-type", "application/json").header("content-encoding", "gzip"); return gzipAsync(payload); });
const hash = await bcrypt.hash("correct horse battery staple", 12);
app.get("/bcrypt", async () => ({ ok: await bcrypt.compare("correct horse battery staple", hash) }));
app.get("/scrypt", async () => new Promise((res, rej) => scrypt("correct horse battery staple", "salt", 64, { N: 2 ** 15, r: 8, p: 1, maxmem: 64 * 1024 * 1024 }, (e, k) => e ? rej(e) : res({ k: k.length }))));
app.get("/pbkdf2", async () => new Promise((res, rej) => pbkdf2("correct horse battery staple", "salt", 600000, 32, "sha256", (e, k) => e ? rej(e) : res({ k: k.length }))));
// a 4000x3000 photo-like source, generated once; every request resizes it to a 320px JPEG
const source = await sharp({ create: { width: 4000, height: 3000, channels: 3, noise: { type: "gaussian", mean: 128, sigma: 40 } } }).jpeg({ quality: 90 }).toBuffer();
app.get("/thumb", async (req, reply) => { reply.type("image/jpeg"); return sharp(source).resize(320).jpeg({ quality: 80 }).toBuffer(); });
await app.listen({ port: 0, host: "127.0.0.1" });
console.log("PORT=" + app.server.address().port + " UV_THREADPOOL_SIZE=" + (process.env.UV_THREADPOOL_SIZE ?? "unset") + " sharpConcurrency=" + sharp.concurrency());
EOF
bench() { # bench <label> <route> <conns> <cmd...>
  local label=$1 route=$2 conns=$3; shift 3
  "$@" server.mjs > server.out 2>&1 &
  local pid=$!
  for i in $(seq 1 300); do grep -q PORT= server.out && break; sleep 0.2; done
  local port; port=$(sed -n 's/PORT=\([0-9]*\).*/\1/p' server.out | head -1)
  if [ -z "$port" ]; then echo "$label $route: server failed"; cat server.out; kill $pid 2>/dev/null; return; fi
  local pool; pool=$(sed -n 's/.*UV_THREADPOOL_SIZE=\([^ ]*\).*/\1/p' server.out | head -1)
  # nub must have sized the pool, or this run compares node with node: the binary under test
  # has to come from a tree that carries the threadpool augmentation.
  if [ "$label" = nub ] && [ "$pool" = unset ]; then echo "FATAL: nub did not set UV_THREADPOOL_SIZE (built from a tree without the augmentation?)"; kill $pid; exit 1; fi
  local res; res=$(PATH="$N:$PATH" ./node_modules/.bin/autocannon -c "$conns" -d 15 -H "accept-encoding: gzip" --json "http://127.0.0.1:$port$route" 2>/dev/null | "$N/node" -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{const j=JSON.parse(s);console.log(JSON.stringify({rps:Math.round(j.requests.average*10)/10,p50:j.latency.p50,p99:j.latency.p99,errors:j.errors,non2xx:j.non2xx}))})')
  echo "$label pool=$pool $route: $res"
  echo "ROW {\"bench\":\"pool-bound\",\"label\":\"$label\",\"pool\":\"$pool\",\"route\":\"$route\",\"result\":$res}"
  kill $pid; wait $pid 2>/dev/null
}
echo "=== pool-bound routes, node (4 threads) vs nub ($NP cores), Node $NV, autocannon -d 15, $ROUNDS interleaved rounds ==="
for r in $(seq 1 "$ROUNDS"); do echo "--- round $r ---"; for route in /bcrypt /scrypt /pbkdf2 /thumb /gzip /gzip-once; do
  PATH="$N:$PATH" bench node "$route" 64 "$N/node"
  PATH="$N:$PATH" bench nub "$route" 64 "$NUB_BIN"
done; done
echo "DONE"
