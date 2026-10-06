import type { ReactNode } from "react";

interface ToggleProps {
  checked: boolean;
  onChange: (v: boolean) => void;
  disabled?: boolean;
}

export function Toggle({ checked, onChange, disabled }: ToggleProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      className={`toggle ${checked ? "toggle--on" : ""}`}
      onClick={() => onChange(!checked)}
      disabled={disabled}
    >
      <span className="toggle__knob" />
    </button>
  );
}

interface SettingRowProps {
  label: string;
  description?: string;
  children: ReactNode;
}

export function SettingRow({ label, description, children }: SettingRowProps) {
  return (
    <div className="setting-row">
      <div className="setting-row__text">
        <div className="setting-row__label">{label}</div>
        {description && <div className="setting-row__desc">{description}</div>}
      </div>
      <div className="setting-row__control">{children}</div>
    </div>
  );
}

interface SettingSectionProps {
  title: string;
  description?: string;
  children: ReactNode;
}

export function SettingSection({ title, description, children }: SettingSectionProps) {
  return (
    <section className="setting-section">
      <div className="setting-section__header">
        <h3>{title}</h3>
        {description && <p>{description}</p>}
      </div>
      <div className="setting-section__body">{children}</div>
    </section>
  );
}

interface SelectProps {
  value: string | number;
  options: { value: string | number; label: string }[];
  onChange: (v: string) => void;
}

export function Select({ value, options, onChange }: SelectProps) {
  return (
    <select className="setting-select" value={value} onChange={(e) => onChange(e.target.value)}>
      {options.map((o) => (
        <option key={o.value} value={o.value}>
          {o.label}
        </option>
      ))}
    </select>
  );
}

interface SliderProps {
  value: number;
  min: number;
  max: number;
  step?: number;
  suffix?: string;
  onChange: (v: number) => void;
}

export function Slider({ value, min, max, step = 1, suffix = "", onChange }: SliderProps) {
  return (
    <div className="setting-slider">
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
      />
      <span className="setting-slider__value">
        {value}
        {suffix}
      </span>
    </div>
  );
}
