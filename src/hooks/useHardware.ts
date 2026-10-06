import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import type { HardwareProfile } from "../types/hardware";

export function useHardware() {
  const [profile, setProfile] = useState<HardwareProfile | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    invoke<HardwareProfile>("get_hardware_profile_cmd")
      .then((data) => {
        if (!cancelled) {
          setProfile(data);
          setError(null);
          setLoading(false);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(String(err));
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return { profile, loading, error };
}
