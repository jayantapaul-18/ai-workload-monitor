import type { ReactNode } from "react";

interface StatCardProps {
  title: string;
  value: string;
  subtitle?: string;
  icon?: ReactNode;
  accent?: string;
  children?: ReactNode;
}

export function StatCard({
  title,
  value,
  subtitle,
  icon,
  accent = "var(--accent)",
  children,
}: StatCardProps) {
  return (
    <div className="stat-card">
      <div className="stat-card__accent" style={{ background: accent }} />
      <div className="stat-card__body">
        <div className="stat-card__header">
          <span className="stat-card__title">{title}</span>
          {icon && <span className="stat-card__icon">{icon}</span>}
        </div>
        <div className="stat-card__value">{value}</div>
        {subtitle && <div className="stat-card__subtitle">{subtitle}</div>}
        {children}
      </div>
    </div>
  );
}
