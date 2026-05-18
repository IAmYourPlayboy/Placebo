import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// HLS-проксирование живёт на axum-бэкенде с M3 (см. crates/placebo-api/
// src/handlers/hls_proxy.rs). Vite-server остаётся минимальным:
// единственная задача – проксировать /api на axum, чтобы фронт ходил
// на относительный URL и в dev, и в Tauri-сборке.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
    proxy: {
      "/api": {
        target: "http://localhost:3001",
        changeOrigin: true,
      },
    },
  },
});
