import { getCurrentWindow } from "@tauri-apps/api/window";
import { type MouseEvent, useCallback, useEffect, useRef, useState } from "react";
import { antigravity } from "./client";
import { AccountCard } from "./components/account-card";
import { ActivitySheet } from "./components/activity-sheet";
import { ConfirmSheet } from "./components/confirm-sheet";
import { EmptyState } from "./components/empty-state";
import { Footer } from "./components/footer";
import { Header } from "./components/header";
import { NetworkSheet } from "./components/network-sheet";
import { ResultSheet } from "./components/result-sheet";
import { displayName } from "./format";
import { useAutosize } from "./hooks/use-autosize";
import { useSnapshot } from "./hooks/use-snapshot";
import { useTicker } from "./hooks/use-ticker";
import { errorText, t } from "./i18n";
import { asBackendError, ipc, onShown } from "./ipc";
import { showAccountMenu, showAppMenu } from "./menus";
import { bestAccountId } from "./quota";
import type { Account, ProcessKind, RelatedProcess } from "./types";
import "./app.css";

type Dialog =
  | { kind: "network" }
  | { kind: "enableUsage" }
  | { kind: "switch"; account: Account; processes: RelatedProcess[] }
  | { kind: "remove"; account: Account }
  | { kind: "stop" | "restart"; processes: RelatedProcess[] }
  | { kind: "result"; tone: "success" | "error"; message: string };

const PROCESS_ORDER: ProcessKind[] = ["app", "languageServer", "helper"];

function describeProcesses(processes: RelatedProcess[]): string | null {
  const kinds = PROCESS_ORDER.filter((kind) => processes.some((p) => p.kind === kind));
  if (!kinds.length) return null;
  const list = kinds.map((kind) => t(`process.${kind}`, antigravity));
  return t("confirm.processes", { list: list.join(" · ") });
}

export function App() {
  const snapshot = useSnapshot();
  const panel = useRef<HTMLDivElement>(null);
  const [dialog, setDialog] = useState<Dialog | null>(null);
  useTicker();
  useAutosize(panel, snapshot?.platform === "macos");

  const closeDialog = useCallback(() => setDialog(null), []);
  const fail = useCallback((error: unknown) => {
    const { code, message } = asBackendError(error);
    if (code !== "oauth_cancelled")
      setDialog({ kind: "result", tone: "error", message: errorText(code, message) });
  }, []);

  useEffect(() => {
    const unlisten = onShown(() => {
      ipc.refresh(false).catch(() => {});
      setDialog((current) => (current?.kind === "result" && current.tone === "success" ? null : current));
    });
    return () => {
      unlisten.then((stop) => stop());
    };
  }, []);

  useEffect(() => {
    if (snapshot?.platform !== "macos" || dialog) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") getCurrentWindow().hide();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [snapshot?.platform, dialog]);

  if (!snapshot) return null;

  const busy = snapshot.operation.kind !== "idle";
  const status = snapshot.client;
  const best = status.identityStale ? null : bestAccountId(snapshot.accounts, status.activeId);
  // The account in use leads the list; the rest keep the order they were added in.
  const accounts = [...snapshot.accounts].sort(
    (a, b) => Number(b.id === status.activeId) - Number(a.id === status.activeId),
  );

  const addAccount = () => ipc.addAccount().catch(fail);
  const askSwitch = async (account: Account) =>
    setDialog({ kind: "switch", account, processes: await ipc.runningProcesses() });
  const askClient = async (kind: "stop" | "restart") => {
    const processes = await ipc.runningProcesses();
    if (kind === "restart" && !processes.length) {
      ipc.restartClient().catch(fail);
      return;
    }
    setDialog({ kind, processes });
  };

  const confirmDialog = () => {
    if (!dialog || dialog.kind === "result" || dialog.kind === "network") return;
    setDialog(null);
    switch (dialog.kind) {
      case "switch": {
        const label = displayName(dialog.account.name, dialog.account.email);
        ipc
          .switchAccount(dialog.account.id)
          .then(() =>
            setDialog({ kind: "result", tone: "success", message: t("progress.done", { email: label }) }),
          )
          .catch(fail);
        break;
      }
      case "remove":
        ipc.removeAccount(dialog.account.id).catch(fail);
        break;
      case "enableUsage":
        ipc.setUsageEnabled(true).catch(fail);
        break;
      case "stop":
        ipc.stopClient().catch(fail);
        break;
      case "restart":
        ipc.restartClient().catch(fail);
        break;
    }
  };

  const accountMenu = (event: MouseEvent, account: Account) =>
    showAccountMenu(event, account, {
      onReauth: addAccount,
      onRemove: () => setDialog({ kind: "remove", account }),
    });

  return (
    <div ref={panel} className={`panel platform-${snapshot.platform}`}>
      <div className="panel-content">
        <Header
          snapshot={snapshot}
          status={status}
          onRefresh={() => ipc.refresh(true).catch(fail)}
          onMore={(event) =>
            showAppMenu(event, snapshot, {
              onNetwork: () => setDialog({ kind: "network" }),
              onToggleUsage: () => {
                if (snapshot.usageEnabled) ipc.setUsageEnabled(false).catch(fail);
                else setDialog({ kind: "enableUsage" });
              },
            }).catch(fail)
          }
        />

        <div className="accounts-panel">
          {accounts.length === 0 ? (
            <EmptyState disabled={busy || !status.installed} onAdd={addAccount} client={antigravity.client} />
          ) : (
            <main className="account-list">
              {accounts.map((account) => (
                <AccountCard
                  key={account.id}
                  account={account}
                  active={account.id === status.activeId && !status.identityStale}
                  lastKnown={account.id === status.activeId && status.identityStale}
                  best={account.id === best}
                  disabled={busy || !status.installed}
                  onSwitch={() => askSwitch(account).catch(fail)}
                  onReauth={addAccount}
                  onMenu={(event) => accountMenu(event, account)}
                />
              ))}
            </main>
          )}
        </div>
        <Footer
          status={status}
          busy={busy}
          onAdd={addAccount}
          onRestart={() => askClient("restart").catch(fail)}
          onStop={() => askClient("stop").catch(fail)}
        />
      </div>

      {snapshot.operation.kind !== "idle" ? (
        <ActivitySheet
          operation={snapshot.operation}
          accounts={snapshot.accounts}
          onCancelAdd={() => ipc.cancelAddAccount()}
          onCopySignInLink={ipc.copySignInLink}
        />
      ) : dialog?.kind === "network" ? (
        <NetworkSheet status={snapshot.network} onClose={closeDialog} />
      ) : dialog?.kind === "enableUsage" ? (
        <ConfirmSheet
          title={t("usage.title")}
          body={t("usage.risk")}
          note={t("usage.refresh")}
          action={t("usage.enable")}
          focusCancel
          onConfirm={confirmDialog}
          onCancel={closeDialog}
        />
      ) : dialog?.kind === "result" ? (
        <ResultSheet tone={dialog.tone} message={dialog.message} onClose={closeDialog} />
      ) : dialog?.kind === "switch" ? (
        <ConfirmSheet
          title={t("confirm.switch.title", { email: dialog.account.email })}
          body={t("confirm.switch.body", antigravity)}
          note={describeProcesses(dialog.processes)}
          action={t("confirm.switch.action")}
          onConfirm={confirmDialog}
          onCancel={closeDialog}
        />
      ) : dialog?.kind === "remove" ? (
        <ConfirmSheet
          title={t("confirm.remove.title", { email: dialog.account.email })}
          body={t("confirm.remove.body")}
          note={dialog.account.id === status.activeId ? t("confirm.remove.active", antigravity) : null}
          action={t("confirm.remove.action")}
          destructive
          onConfirm={confirmDialog}
          onCancel={closeDialog}
        />
      ) : dialog?.kind === "stop" || dialog?.kind === "restart" ? (
        <ConfirmSheet
          title={t(`confirm.${dialog.kind}.title`, antigravity)}
          body={t(`confirm.${dialog.kind}.body`, antigravity)}
          note={describeProcesses(dialog.processes)}
          action={t(`confirm.${dialog.kind}.action`)}
          destructive={dialog.kind === "stop"}
          onConfirm={confirmDialog}
          onCancel={closeDialog}
        />
      ) : null}
    </div>
  );
}
