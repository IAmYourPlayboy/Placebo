# Demo loop_mp4 assets

The alpha seed (migration 010) references 5 looped MP4 cameras whose `slug`
starts with `demo-`. The HLS proxy resolves each one to a static asset at:

```
static/demo/<asset>/index.m3u8
static/demo/<asset>/segment-*.ts
```

Mapping (slug → asset folder, see `010_seed_alpha_cameras.sql`):

| slug                 | asset folder    | duration |
|----------------------|-----------------|----------|
| demo-tokyo-alley     | tokyo-alley     | ~92s     |
| demo-cafe-street     | cafe-street     | ~120s    |
| demo-beach-sunset    | beach-sunset    | ~180s    |
| demo-rainy-window    | rainy-window    | ~240s    |
| demo-mountain-pass   | mountain-pass   | ~150s    |

Generate from a source MP4 with FFmpeg:

```bash
mkdir -p static/demo/tokyo-alley
ffmpeg -i source.mp4 \
    -c:v libx264 -preset veryfast -crf 23 \
    -g 48 -hls_time 4 -hls_playlist_type vod \
    static/demo/tokyo-alley/index.m3u8
```

The assets themselves are NOT committed (large binaries). `.gitignore` keeps
only this README; ship the actual segments out-of-band (e.g. R2) for prod.
