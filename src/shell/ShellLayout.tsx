import type { ReactNode } from "react";

type Props = {
  sidebar: ReactNode;
  topbar: ReactNode;
  tabbar: ReactNode;
  breadcrumbs: ReactNode;
  content: ReactNode;
};

export default function ShellLayout({ sidebar, topbar, tabbar, breadcrumbs, content }: Props) {
  return (
    <div className="shell-layout">
      {sidebar}
      {topbar}
      {tabbar}
      {breadcrumbs}
      {content}
    </div>
  );
}
