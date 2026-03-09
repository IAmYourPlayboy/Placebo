# Placebo

Co-watching desktop app built with Tauri 2 + React + TypeScript + Rust.

## Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) (stable)
- [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS

### Windows
```
winget install Microsoft.VisualStudio.2022.BuildTools
winget install Microsoft.WebView2Runtime
```

### macOS
```
xcode-select --install
```

## Setup

```bash
# 1. Install JS deps
npm install

# 2. Run in dev mode (opens the app window)
npm run tauri dev

# 3. Build for production
npm run tauri build
```

## Project Structure

```
placebo/
├── src/                      ← React frontend
│   ├── main.tsx              ← Entry point
│   ├── App.tsx               ← Root + screen router
│   ├── App.css               ← Design system + all styles
│   ├── components/
│   │   ├── BottomNav.tsx     ← Navigation bar (5 tabs)
│   │   └── Icons.tsx         ← All SVG icons
│   └── screens/
│       ├── HomeScreen.tsx    ← Главная (favorites + popular rooms)
│       ├── ProfileScreen.tsx ← Профиль (avatar, posts grid)
│       ├── ExploreScreen.tsx ← Каталог (stub)
│       ├── CreateScreen.tsx  ← Создать комнату (stub)
│       └── FriendsScreen.tsx ← Друзья (stub)
│
├── src-tauri/                ← Rust backend
│   ├── src/
│   │   ├── main.rs           ← Binary entry point
│   │   └── lib.rs            ← Tauri commands + state
│   ├── capabilities/
│   │   └── default.json      ← App permissions
│   ├── Cargo.toml
│   ├── build.rs
│   └── tauri.conf.json       ← Window config, bundle settings
│
├── index.html
├── vite.config.ts
├── tsconfig.json
└── package.json
```

## Screens Status

| Screen   | Status      | Notes                         |
|----------|-------------|-------------------------------|
| Главная  | ✅ Ready    | Favorites grid + room cards   |
| Профиль  | ✅ Ready    | Avatar, follow, posts grid    |
| Каталог  | 🔲 Stub     | Next to implement             |
| Создать  | 🔲 Stub     | Form for room creation        |
| Друзья   | 🔲 Stub     | Friends list                  |

## Rust Commands Available

| Command           | Description                        |
|-------------------|------------------------------------|
| `greet`           | Test hello                         |
| `get_user_id`     | Returns current user session ID    |
| `get_public_rooms`| Returns list of public rooms       |
| `create_room`     | Creates a new room (returns Room)  |
