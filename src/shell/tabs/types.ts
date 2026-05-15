import type { Router } from "@remix-run/router";

export type TabId = string;

export type Tab = {
  id: TabId;
  title: string;
  /** Path при создании таба; текущий путь узнаётся через router.state.location. */
  initialPath: string;
  /** Per-tab memory router. Управляет per-tab историей. */
  router: Router;
  /** Момент создания (Date.now()) – используется для стабильной сортировки. */
  createdAt: number;
};

export type TabManagerApi = {
  tabs: Tab[];
  activeTabId: TabId;
  openTab(path: string, title?: string): TabId;
  closeTab(id: TabId): void;
  activateTab(id: TabId): void;
  renameTab(id: TabId, title: string): void;
  /** Навигация внутри активного таба по path. Создаёт новую запись в history. */
  navigateInActiveTab(path: string): void;
  /** Кнопки <, >, ↻ */
  goBack(): void;
  goForward(): void;
  reload(): void;
};
