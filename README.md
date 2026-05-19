<div align="center">

# Placebo

**Десктопное social-приложение на Tauri, React и Rust**

Пространство для комнат, профилей, избранного контента и пользовательского взаимодействия.

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

## О проекте

**Placebo** — это десктопное приложение, построенное вокруг идеи пользовательских комнат, профилей и персонального контента.

Проект сочетает легкий frontend на **React + TypeScript** с backend-частью на **Rust** через **Tauri**. Такой стек позволяет сохранить современный пользовательский интерфейс, высокую производительность и компактную десктопную сборку.

## Основные возможности

- Главный экран с избранными элементами и популярными комнатами.
- Профиль пользователя с аватаром, подпиской и сеткой публикаций.
- Навигация между основными разделами приложения.
- Rust-команды для работы с пользовательской сессией и комнатами.
- Основа для расширения: каталог, создание комнат, друзья и дальнейшая социальная логика.

## Технологический стек

| Layer | Technologies |
|---|---|
| Desktop runtime | Tauri |
| Backend | Rust |
| Frontend | React, TypeScript |
| Styling | CSS |
| Build tooling | Vite, npm |
| Database logic | PLpgSQL |

## Быстрый старт

```bash
# 1. Установить зависимости
npm install

# 2. Запустить приложение в режиме разработки
npm run tauri dev

# 3. Собрать production-версию
npm run tauri build
