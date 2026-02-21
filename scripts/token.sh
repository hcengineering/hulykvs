#!/bin/bash

CONFIG_PATH="../hulykvs_server/src/config/default.toml"

# Prefer explicit env var to match how the server is started locally.
if [ -n "${HULY_TOKEN_SECRET:-}" ]; then
  SECRET="${HULY_TOKEN_SECRET}"
else
  SECRET=$(sed -nE 's/^[[:space:]]*token_secret[[:space:]]*=[[:space:]]*"([^"]+)".*$/\1/p' "$CONFIG_PATH" | head -n 1)
fi

if [ -z "$SECRET" ]; then
  echo "❌No token_secret in $CONFIG_PATH"
  exit 1
fi

claims=$1 # "claims.json"

if ! command -v jwt >/dev/null 2>&1; then
  echo "❌ jwt CLI is not installed"
  exit 1
fi

if [ -z "$claims" ] || [ ! -f "$claims" ]; then
  echo "❌ Claims file is missing: $claims"
  exit 1
fi

# Try old jwt-cli syntax first.
TOKEN=$(echo -n "${SECRET}" | jwt -alg HS256 -key - -sign "${claims}" 2>/dev/null || true)

# Fallback to new jwt-cli syntax.
if [ -z "$TOKEN" ]; then
  payload=$(tr -d '\n' < "${claims}")
  TOKEN=$(jwt encode --alg HS256 --secret "${SECRET}" "${payload}" 2>/dev/null || true)
fi

if [ -z "$TOKEN" ]; then
  echo "❌ Failed to create JWT token with available jwt CLI syntax"
  exit 1
fi

echo "$TOKEN"
