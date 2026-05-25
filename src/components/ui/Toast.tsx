import {
  createContext,
  ReactNode,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import "./toast.css";

type ToastApi = {
  show(message: string): void;
};

const ToastContext = createContext<ToastApi | null>(null);
const DEFAULT_DURATION_MS = 2200;

/**
 * Single-slot toast: showing a new message replaces any in-flight one.
 * That's all the alpha needs ("Coming soon" feedback for disabled tiles);
 * we will swap this for a real queue if we ever stack toasts.
 */
export function ToastProvider({ children }: { children: ReactNode }) {
  const [message, setMessage] = useState<string | null>(null);
  const timer = useRef<number | null>(null);

  const show = useCallback((msg: string) => {
    setMessage(msg);
    if (timer.current !== null) {
      window.clearTimeout(timer.current);
    }
    timer.current = window.setTimeout(() => {
      setMessage(null);
      timer.current = null;
    }, DEFAULT_DURATION_MS);
  }, []);

  useEffect(() => {
    return () => {
      if (timer.current !== null) {
        window.clearTimeout(timer.current);
      }
    };
  }, []);

  return (
    <ToastContext.Provider value={{ show }}>
      {children}
      {message !== null && (
        <div className="toast" role="status" aria-live="polite">
          {message}
        </div>
      )}
    </ToastContext.Provider>
  );
}

export function useToast(): ToastApi {
  const ctx = useContext(ToastContext);
  if (!ctx) throw new Error("useToast must be used within <ToastProvider>");
  return ctx;
}
