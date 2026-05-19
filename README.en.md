<div align="center">

# Placebo

**A desktop social app built with Tauri, React, and Rust**

A space for rooms, profiles, favorite content, and user interaction.

<a href="./README.md">
  <img alt="RU" src="https://img.shields.io/badge/README-RU-2f80ed?style=for-the-badge">
</a>
<a href="./README.en.md">
  <img alt="EN" src="https://img.shields.io/badge/README-EN-111827?style=for-the-badge">
</a>

</div>

## About

**Placebo** is a desktop application centered around user rooms, profiles, and personal content.

The project combines a modern **React + TypeScript** frontend with a **Rust** backend powered by **Tauri**. This stack keeps the interface flexible while providing strong performance, native desktop capabilities, and a compact application bundle.

## Features

- Home screen with favorites and popular rooms.
- User profile with avatar, follow action, and posts grid.
- Bottom navigation between core app sections.
- Rust commands for session and room management.
- Expandable foundation for explore, room creation, friends, and social features.

## Tech Stack

| Layer | Technologies |
|---|---|
| Desktop runtime | Tauri |
| Backend | Rust |
| Frontend | React, TypeScript |
| Styling | CSS |
| Build tooling | Vite, npm |
| Database logic | PLpgSQL |

## Getting Started

```bash
# 1. Install dependencies
npm install

# 2. Run in development mode
npm run tauri dev

# 3. Build for production
npm run tauri build
