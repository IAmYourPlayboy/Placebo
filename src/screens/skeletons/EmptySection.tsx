import { ReactNode } from "react";

type Props = {
  title: string;
  hint?: ReactNode;
};

export default function EmptySection({ title, hint }: Props) {
  return (
    <div className="empty-section">
      <div className="empty-section__title">{title}</div>
      {hint && <div className="empty-section__hint">{hint}</div>}
    </div>
  );
}
