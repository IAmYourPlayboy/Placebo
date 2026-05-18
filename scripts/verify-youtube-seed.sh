#!/usr/bin/env bash
# Sanity-check that every YouTube videoId baked into migration 010
# (alpha camera seed) still resolves via yt-dlp. Prints OK/FAIL per id
# and exits non-zero if any failed.
#
# Run before pushing M3 work or whenever a YouTube live stream is suspected
# to have ended. The list here MUST stay in sync with
# crates/placebo-api/migrations/010_seed_alpha_cameras.sql.

set -uo pipefail

declare -a IDS=(
    dfVK7ld38Ys u4UZ4UvZXrg FQWkgr0aHlI AdUw5RdyZxI h1wly909BYw
    7Bl5p4VTXzQ 2L4yhCmGRWg dyWHmEQAVUI qMksIqJv3pI wNmMr_ATI2E
    hSbkw-F7bzY 2Te5EvOXNZw SkdGPWUUkEw
)

if ! command -v yt-dlp >/dev/null 2>&1; then
    echo "yt-dlp is not on PATH – install it first (winget install yt-dlp \
/ pip install yt-dlp / brew install yt-dlp)." >&2
    exit 2
fi

failed=0
for id in "${IDS[@]}"; do
    printf '%-12s ... ' "$id"
    if yt-dlp -f 'best[vcodec^=avc1]' --no-warnings -g \
        "https://www.youtube.com/watch?v=${id}" >/dev/null 2>&1; then
        echo OK
    else
        echo FAIL
        failed=$((failed + 1))
    fi
done

if [[ $failed -gt 0 ]]; then
    echo ""
    echo "$failed/${#IDS[@]} ids failed to resolve. Update migration 010 \
with fresh live IDs and rerun."
    exit 1
fi
echo ""
echo "All ${#IDS[@]} ids resolve OK."
