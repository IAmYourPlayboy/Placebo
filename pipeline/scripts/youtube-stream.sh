#!/bin/sh
# Извлекает YouTube Live stream URL через yt-dlp и передаёт через ffmpeg как чистый MPEG-TS
# Использование: youtube-stream.sh <youtube-url>
URL=$(yt-dlp -f 'best[vcodec^=avc1]' --no-warnings -g "$1" 2>/dev/null)
if [ -z "$URL" ]; then
  # Fallback: попробовать без фильтра кодека
  URL=$(yt-dlp -f best --no-warnings -g "$1" 2>/dev/null)
fi
if [ -z "$URL" ]; then
  echo "ERROR: failed to extract URL from $1" >&2
  exit 1
fi
exec ffmpeg -hide_banner -loglevel error -i "$URL" -c:v copy -c:a aac -f mpegts pipe:1
