import { useState } from "react";
import { errorText, t } from "../i18n";
import { asBackendError, ipc } from "../ipc";
import type { NetworkStatus } from "../types";
import { Sheet, SheetActions } from "./sheet";
import "./network-sheet.css";

export function NetworkSheet({ status, onClose }: { status: NetworkStatus; onClose: () => void }) {
  const [local, setLocal] = useState(status.localHttpProxyPort !== null);
  const [port, setPort] = useState(status.localHttpProxyPort?.toString() ?? "");
  const [pending, setPending] = useState<"test" | "save" | null>(null);
  const [message, setMessage] = useState<string | null>(
    status.invalid ? t("error.network_configuration") : null,
  );
  const valid = !local || (/^[0-9]{1,5}$/.test(port) && Number(port) >= 1 && Number(port) <= 65535);

  async function apply(action: "test" | "save") {
    if (!valid || pending) return;
    setPending(action);
    setMessage(null);
    const settings = { localHttpProxyPort: local ? Number(port) : null };
    try {
      if (action === "save") {
        await ipc.saveNetwork(settings);
        onClose();
      } else {
        await ipc.testNetwork(settings);
        setMessage(t("network.ok"));
      }
    } catch (error) {
      const { code, message } = asBackendError(error);
      setMessage(errorText(code, message));
    } finally {
      setPending(null);
    }
  }

  return (
    <Sheet onDismiss={pending ? undefined : onClose}>
      <h2>{t("network.title")}</h2>
      <fieldset className="network-fields" disabled={pending !== null}>
        <label>
          <span>{t("network.route")}</span>
          <select
            value={local ? "local" : "system"}
            onChange={(event) => {
              setLocal(event.currentTarget.value === "local");
              setMessage(null);
            }}
          >
            <option value="system">{t("network.system")}</option>
            <option value="local">{t("network.local")}</option>
          </select>
        </label>
        {local && (
          <label>
            <span>{t("network.port")}</span>
            <input
              inputMode="numeric"
              placeholder="8080"
              value={port}
              onChange={(event) => {
                setPort(event.currentTarget.value.trim());
                setMessage(null);
              }}
            />
          </label>
        )}
      </fieldset>
      <p className="sheet-note">{t("network.scope")}</p>
      {message && <p role="status">{message}</p>}
      <SheetActions>
        <button type="button" className="btn" disabled={pending !== null} onClick={onClose}>
          {t("confirm.cancel")}
        </button>
        <button
          type="button"
          className="btn"
          disabled={!valid || pending !== null}
          onClick={() => apply("test")}
        >
          {t(pending === "test" ? "network.testing" : "network.test")}
        </button>
        <button
          type="button"
          className="btn btn-primary"
          disabled={!valid || pending !== null}
          onClick={() => apply("save")}
        >
          {t("network.save")}
        </button>
      </SheetActions>
    </Sheet>
  );
}
