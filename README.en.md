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

<br><br>

<img alt="Rust" src="https://img.shields.io/badge/Rust-47.4%25-black?style=flat-square&logo=rust">
<img alt="TypeScript" src="https://img.shields.io/badge/TypeScript-37.3%25-3178c6?style=flat-square&logo=typescript&logoColor=white">
<img alt="CSS" src="https://img.shields.io/badge/CSS-8.9%25-663399?style=flat-square&logo=css">
<img alt="JavaScript" src="https://img.shields.io/badge/JavaScript-2.6%25-f7df1e?style=flat-square&logo=javascript&logoColor=black">

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
