// The only module that knows command and event names.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { BackendError, NetworkSettings, RelatedProcess, Snapshot } from "./types";

export const ipc = {
  snapshot: () => invoke<Snapshot>("get_snapshot"),
  refresh: (force: boolean) => invoke<void>("refresh", { force }),
  setUsageEnabled: (enabled: boolean) => invoke<void>("set_usage_enabled", { enabled }),
  testNetwork: (settings: NetworkSettings) => invoke<void>("test_network", { settings }),
  saveNetwork: (settings: NetworkSettings) => invoke<void>("save_network", { settings }),
  addAccount: () => invoke<string>("add_account"),
  cancelAddAccount: () => invoke<void>("cancel_add_account"),
  copySignInLink: () => invoke<void>("copy_sign_in_link"),
  removeAccount: (id: string) => invoke<void>("remove_account", { id }),
  switchAccount: (id: string) => invoke<void>("switch_account", { id }),
  runningProcesses: () => invoke<RelatedProcess[]>("running_processes"),
  stopClient: () => invoke<void>("stop_client"),
  restartClient: () => invoke<void>("restart_client"),
  resizePanel: (height: number) => invoke<void>("resize_panel", { height }),
  revealLogs: () => invoke<void>("reveal_logs"),
  autostartEnabled: () => invoke<boolean>("autostart_enabled"),
  setAutostart: (enabled: boolean) => invoke<void>("set_autostart", { enabled }),
  quit: () => invoke<void>("quit"),
};

export const onSnapshot = (handler: (snapshot: Snapshot) => void): Promise<UnlistenFn> =>
  listen<Snapshot>("orbitswitch://snapshot", (event) => handler(event.payload));

export const onShown = (handler: () => void): Promise<UnlistenFn> => listen("orbitswitch://shown", handler);

export function asBackendError(error: unknown): BackendError {
  if (error && typeof error === "object" && "code" in error && "message" in error) {
    return error as BackendError;
  }
  return { code: "unexpected", message: String(error) };
}
