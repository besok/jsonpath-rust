#!/usr/bin/env bash

# Use this script to download the RFC9535 compliance suite and prepare it for
# use in tests.

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

url="https://raw.githubusercontent.com/jsonpath-standard/jsonpath-compliance-test-suite/refs/heads/main/cts.json"

curl -s $url | jq -r '.tests' > "$script_dir/rfc9535-cts.json"