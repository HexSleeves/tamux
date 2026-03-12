import { useState, type CSSProperties, type ReactNode } from "react";
import { BUILTIN_THEMES } from "../../lib/themes";
import type { AmuxSettings } from "../../lib/types";

export type SettingsUpdater = <K extends keyof AmuxSettings>(key: K, value: AmuxSettings[K]) => void;

export function Section({ title, children }: { title: string; children: ReactNode }) {
    return (
        <div style={{ marginBottom: 20 }}>
            <div style={{
                fontSize: 12, fontWeight: 600, color: "var(--accent)",
                marginBottom: 8, textTransform: "uppercase", letterSpacing: "0.04em",
            }}>{title}</div>
            {children}
        </div>
    );
}

export function SettingRow({ label, children }: { label: string; children: ReactNode }) {
    return (
        <div style={{
            display: "flex", alignItems: "center", justifyContent: "space-between",
            padding: "6px 0", fontSize: 12, gap: 12,
        }}>
            <span style={{ color: "var(--text-secondary)", flexShrink: 0 }}>{label}</span>
            {children}
        </div>
    );
}

export function FontSelector({ value, fonts, onChange }: {
    value: string; fonts: string[]; onChange: (value: string) => void;
}) {
    return (
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <select value={value} onChange={(event) => onChange(event.target.value)}
                style={{ ...inputStyle, width: 200 }}>
                {fonts.map((font) => (
                    <option key={font} value={font} style={{ fontFamily: font }}>{font}</option>
                ))}
                {!fonts.includes(value) ? <option value={value}>{value}</option> : null}
            </select>
            <span style={{ fontFamily: value, fontSize: 12 }}>Abc</span>
        </div>
    );
}

export function ThemePicker({ value, onChange }: { value: string; onChange: (value: string) => void }) {
    return (
        <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 8, marginTop: 4 }}>
            {BUILTIN_THEMES.map((theme) => (
                <button key={theme.name} onClick={() => onChange(theme.name)}
                    style={{
                        padding: 8, borderRadius: 0, cursor: "pointer",
                        border: value === theme.name ? "2px solid var(--accent)" : "2px solid var(--border)",
                        background: theme.colors.background,
                        display: "flex", flexDirection: "column", gap: 4,
                        transition: "border-color 0.15s",
                    }}>
                    <div style={{ display: "flex", gap: 2 }}>
                        {[theme.colors.red, theme.colors.green, theme.colors.yellow,
                        theme.colors.blue, theme.colors.magenta, theme.colors.cyan].map((color, index) => (
                            <div key={index} style={{ width: 8, height: 8, borderRadius: 2, background: color }} />
                        ))}
                    </div>
                    <span style={{
                        fontSize: 9, color: theme.colors.foreground, whiteSpace: "nowrap",
                        overflow: "hidden", textOverflow: "ellipsis",
                    }}>{theme.name}</span>
                </button>
            ))}
        </div>
    );
}

export function ColorInput({ value, onChange, placeholder }: {
    value: string; onChange: (value: string) => void; placeholder?: string;
}) {
    return (
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <input type="color" value={value || placeholder || "#000000"}
                onChange={(event) => onChange(event.target.value)}
                style={{
                    width: 28, height: 22, padding: 0, border: "1px solid var(--border)",
                    borderRadius: 0, cursor: "pointer", background: "none",
                }} />
            <input type="text" value={value} onChange={(event) => onChange(event.target.value)}
                placeholder={placeholder}
                style={{ ...inputStyle, width: 100, fontFamily: "var(--font-mono)", fontSize: 11 }} />
        </div>
    );
}

export function SliderInput({ value, min, max, step, onChange }: {
    value: number; min: number; max: number; step: number;
    onChange: (value: number) => void;
}) {
    return (
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <input type="range" min={min} max={max} step={step} value={value}
                onChange={(event) => onChange(parseFloat(event.target.value))}
                style={{ width: 120, accentColor: "var(--accent)" }} />
            <span style={{ fontSize: 11, color: "var(--text-secondary)", minWidth: 32, textAlign: "right" }}>
                {Number.isInteger(step) ? value : value.toFixed(step < 0.1 ? 2 : 1)}
            </span>
        </div>
    );
}

export function TextInput({ value, onChange, placeholder }: {
    value: string; onChange: (value: string) => void; placeholder?: string;
}) {
    return (
        <input type="text" value={value} onChange={(event) => onChange(event.target.value)}
            placeholder={placeholder} style={inputStyle} />
    );
}

export function PasswordInput({ value, onChange, placeholder }: {
    value: string; onChange: (value: string) => void; placeholder?: string;
}) {
    const [visible, setVisible] = useState(false);
    return (
        <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
            <input type={visible ? "text" : "password"} value={value}
                onChange={(event) => onChange(event.target.value)}
                placeholder={placeholder}
                style={{ ...inputStyle, flex: 1 }} />
            <button type="button" onClick={() => setVisible(!visible)}
                style={{
                    background: "rgba(255,255,255,0.04)",
                    border: "1px solid rgba(255,255,255,0.08)",
                    color: "var(--text-muted)",
                    cursor: "pointer",
                    fontSize: 11,
                    padding: "4px 8px",
                    borderRadius: 0,
                    lineHeight: 1,
                }}
                title={visible ? "Hide" : "Show"}>
                {visible ? "\u25C9" : "\u25CB"}
            </button>
        </div>
    );
}

export function NumberInput({ value, min, max, step, onChange }: {
    value: number; min?: number; max?: number; step?: number;
    onChange: (value: number) => void;
}) {
    return (
        <input type="number" value={value} min={min} max={max} step={step ?? 1}
            onChange={(event) => {
                const nextValue = parseFloat(event.target.value);
                if (!isNaN(nextValue)) onChange(nextValue);
            }}
            style={{ ...inputStyle, width: 80 }} />
    );
}

export function SelectInput({ value, options, onChange }: {
    value: string; options: string[]; onChange: (value: string) => void;
}) {
    return (
        <select value={value} onChange={(event) => onChange(event.target.value)} style={inputStyle}>
            {options.map((option) => (<option key={option} value={option}>{option}</option>))}
        </select>
    );
}

export function Toggle({ value, onChange }: { value: boolean; onChange: (value: boolean) => void }) {
    return (
        <button onClick={() => onChange(!value)} style={{
            width: 36, height: 20, borderRadius: 0, border: "none",
            background: value ? "var(--accent)" : "var(--bg-surface)",
            cursor: "pointer", position: "relative", transition: "background 0.2s",
        }}>
            <div style={{
                width: 14, height: 14, borderRadius: "50%", background: "var(--text-primary)",
                position: "absolute", top: 3, left: value ? 19 : 3, transition: "left 0.2s",
            }} />
        </button>
    );
}

export const inputStyle: CSSProperties = {
    background: "var(--bg-surface)", border: "1px solid var(--border)",
    borderRadius: 0, color: "var(--text-primary)", fontSize: 12,
    padding: "3px 8px", fontFamily: "inherit", outline: "none", width: 200,
};

export const headerBtnStyle: CSSProperties = {
    background: "none", border: "none", color: "var(--text-secondary)",
    cursor: "pointer", fontSize: 12, padding: "2px 6px",
};

export const addBtnStyle: CSSProperties = {
    background: "var(--bg-surface)", border: "1px solid var(--border)",
    color: "var(--text-primary)", cursor: "pointer", fontSize: 11,
    padding: "4px 10px", borderRadius: 0, marginTop: 8,
};

export const kbdStyle: CSSProperties = {
    background: "var(--bg-surface)", padding: "2px 6px", borderRadius: 0,
    fontSize: 10, fontFamily: "var(--font-mono)",
};

export const rebindBtnStyle: CSSProperties = {
    background: "var(--bg-surface)",
    border: "1px solid var(--border)",
    borderRadius: 0,
    color: "var(--text-primary)",
    cursor: "pointer",
    fontSize: 11,
    padding: "4px 8px",
};

export const smallBtnStyle: CSSProperties = {
    background: "var(--bg-surface)",
    border: "1px solid var(--border)",
    borderRadius: 0,
    color: "var(--text-primary)",
    cursor: "pointer",
    fontSize: 11,
    padding: "4px 8px",
};