#!/usr/bin/env bash
set -euo pipefail

command -v jq >/dev/null 2>&1 || exit 0

edited=$(jq -r '.tool_input.file_path // empty')
case "$edited" in
*.ir.yaml | *.ir.yml) ;;
*) exit 0 ;;
esac

[ -f "$edited" ] || exit 0

command -v erdbt >/dev/null 2>&1 || exit 0

erdbt ir check --help >/dev/null 2>&1 || exit 0

erdbt ir check "$edited"
