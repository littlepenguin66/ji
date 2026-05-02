#!/bin/bash
set -euo pipefail

# Mirror CLAUDE.md → AGENTS.md
cp CLAUDE.md AGENTS.md
echo "CLAUDE.md → AGENTS.md mirrored"
