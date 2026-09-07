#!/usr/bin/env bash
# Print every step's wall-clock for one ci-install-probe run, plus the timing
# lines each install step echoed. Usage: harvest.sh <run-id> [owner/repo]
set -euo pipefail
run="${1:?run id}"; repo="${2:-nubjs/nub}"
gh api "repos/$repo/actions/runs/$run/jobs?per_page=100" --jq '
  .jobs[] | . as $j | .steps[] |
  select(.completed_at != null and .started_at != null) |
  "\($j.name)\t\(.name)\t\(((.completed_at | fromdate) - (.started_at | fromdate)))s\t\(.conclusion)"' \
  | column -t -s $'\t'
echo
echo "# echoed timings"
for jid in $(gh api "repos/$repo/actions/runs/$run/jobs?per_page=100" --jq '.jobs[].id'); do
  name=$(gh api "repos/$repo/actions/jobs/$jid" --jq .name)
  gh api "repos/$repo/actions/jobs/$jid/logs" 2>/dev/null \
    | grep -oE '(TARBALL_INSTALL_MS|INSTALL_MS|SECOND_INSTALL_MS)=[0-9]+|Cache (restored from key|not found for input keys|saved with key)[^\r]*|Cache Size: [^\r]*|downloaded [0-9]+ \([^)]*\) in [0-9.]+s|added [0-9]+ packages in [0-9a-z.]+' \
    | sed "s/^/$name\t/"
done | column -t -s $'\t'
