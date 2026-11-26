import { type RefObject, useEffect } from "react";
import { ipc, onShown } from "../ipc";

/** Keeps the macOS popover exactly as tall as its content. */
export function useAutosize(ref: RefObject<HTMLElement | null>, enabled: boolean) {
  useEffect(() => {
    const element = ref.current;
    if (!enabled || !element) return;
    let frame = 0;
    let last = 0;
    const measure = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const height = Math.ceil(element.getBoundingClientRect().height);
        if (height !== last) {
          last = height;
          void ipc.resizePanel(height).catch(() => {
            if (last === height) last = 0;
          });
        }
      });
    };
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    const unlisten = onShown(() => {
      // A size requested while the webview was hidden may still need to be applied.
      last = 0;
      measure();
    });
    return () => {
      cancelAnimationFrame(frame);
      observer.disconnect();
      void unlisten.then((stop) => stop());
    };
  }, [ref, enabled]);
}
