import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

interface AlertToast {
  id: number;
  title: string;
  message: string;
}

export function AlertToasts() {
  const [toasts, setToasts] = useState<AlertToast[]>([]);

  useEffect(() => {
    const unlisten = listen<{ title: string; message: string }>("alert-triggered", (e) => {
      const id = Date.now();
      setToasts((prev) => [...prev, { id, ...e.payload }].slice(-3));
      setTimeout(() => {
        setToasts((prev) => prev.filter((t) => t.id !== id));
      }, 5000);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  if (toasts.length === 0) return null;

  return (
    <div className="alert-toasts">
      {toasts.map((t) => (
        <div key={t.id} className="alert-toast">
          <strong>{t.title}</strong>
          <span>{t.message}</span>
        </div>
      ))}
    </div>
  );
}
