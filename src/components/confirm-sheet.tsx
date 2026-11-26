import { useEffect, useRef } from "react";
import { t } from "../i18n";
import { Sheet, SheetActions } from "./sheet";

interface Props {
  title: string;
  body: string;
  note?: string | null;
  action: string;
  destructive?: boolean;
  focusCancel?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmSheet({
  title,
  body,
  note,
  action,
  destructive,
  focusCancel,
  onConfirm,
  onCancel,
}: Props) {
  const initialFocus = useRef<HTMLButtonElement>(null);
  useEffect(() => initialFocus.current?.focus(), []);
  return (
    <Sheet onDismiss={onCancel} onConfirm={focusCancel ? undefined : onConfirm}>
      <h2>{title}</h2>
      <p>{body}</p>
      {note && <p className="sheet-note">{note}</p>}
      <SheetActions>
        <button type="button" className="btn" onClick={onCancel} ref={focusCancel ? initialFocus : undefined}>
          {t("confirm.cancel")}
        </button>
        <button
          type="button"
          className={destructive ? "btn btn-danger" : "btn btn-primary"}
          onClick={onConfirm}
          ref={focusCancel ? undefined : initialFocus}
        >
          {action}
        </button>
      </SheetActions>
    </Sheet>
  );
}
