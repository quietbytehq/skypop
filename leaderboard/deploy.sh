#!/usr/bin/env bash
# The user runs wrangler login themselves. Never enable shell tracing here.
set +x
set -euo pipefail
umask 077
cd "$(dirname "$0")"
export WRANGLER_SEND_METRICS=false
npm ci --no-audit --no-fund >/dev/null
python3 tools_deploy.py
