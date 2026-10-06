import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { AppSettings, DEFAULT_SETTINGS } from "../types/settings";

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [settingsPath, setSettingsPath] = useState("");
  const [loaded, setLoaded] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    let cancelled = false;

    Promise.all([
      invoke<AppSettings>("get_settings"),
      invoke<string>("get_settings_path"),
    ])
      .then(([s, path]) => {
        if (!cancelled) {
          setSettings(s);
          setSettingsPath(path);
          setLoaded(true);
          applyTheme(s);
        }
      })
      .catch(() => {
        if (!cancelled) setLoaded(true);
      });

    const unlisten = listen<AppSettings>("settings-changed", (e) => {
      setSettings(e.payload);
      applyTheme(e.payload);
    });

    const unlistenOpen = listen("open-settings", () => {
      window.dispatchEvent(new CustomEvent("navigate-settings"));
    });

    return () => {
      cancelled = true;
      unlisten.then((fn) => fn());
      unlistenOpen.then((fn) => fn());
    };
  }, []);

  const update = useCallback(async (patch: Partial<AppSettings>) => {
    const next = { ...settings, ...patch };
    setSettings(next);
    setSaving(true);
    try {
      await invoke("save_settings", { settings: next });
      applyTheme(next);
    } finally {
      setSaving(false);
    }
  }, [settings]);

  const reset = useCallback(async () => {
    setSaving(true);
    try {
      const defaults = await invoke<AppSettings>("reset_settings");
      setSettings(defaults);
      applyTheme(defaults);
    } finally {
      setSaving(false);
    }
  }, []);

  return { settings, settingsPath, loaded, saving, update, reset };
}

export function applyTheme(settings: AppSettings) {
  document.documentElement.dataset.theme = settings.colorTheme;
  document.documentElement.dataset.accent = settings.accentTheme;
  document.documentElement.dataset.density = settings.compactDensity ? "compact" : "comfortable";
  document.documentElement.dataset.sidebar = settings.compactSidebar ? "compact" : "full";
  document.documentElement.dataset.animate = settings.animateGauges ? "on" : "off";
}
