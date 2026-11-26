// Mirrors src-tauri/src/model.rs. The backend is the source of truth.

export type GroupKey = "gemini" | "claude" | "other";
export type PlanKind =
  | "free"
  | "pro"
  | "ultra"
  | "plus"
  | "max"
  | "team"
  | "business"
  | "enterprise"
  | "unknown";

export interface QuotaWindow {
  /** 0–1 share still available. */
  remaining: number;
  resetAt: string | null;
  label: string;
  durationMinutes: number | null;
}

export interface QuotaGroup {
  key: GroupKey;
  label: string;
  models: string[];
  fiveHour: QuotaWindow | null;
  weekly: QuotaWindow | null;
  extra: QuotaWindow[];
}

export interface Plan {
  kind: PlanKind;
  name: string;
}

export type QuotaState =
  | { status: "loading" }
  | { status: "ready"; groups: QuotaGroup[]; fetchedAt: string; stale: boolean }
  | { status: "failed"; code: string; message: string }
  | { status: "reauth" };

export interface Account {
  id: string;
  email: string;
  name: string | null;
  picture: string | null;
  plan: Plan | null;
  quota: QuotaState;
}

export type SwitchStep = "preparing" | "closing" | "writing" | "launching";

export type Operation =
  | { kind: "idle" }
  | { kind: "adding"; canCopyLink: boolean }
  | { kind: "removing" }
  | { kind: "switching"; targetId: string; step: SwitchStep }
  | { kind: "stopping" }
  | { kind: "restarting" };

export interface Snapshot {
  platform: "macos" | "windows" | "linux";
  version: string;
  accounts: Account[];
  operation: Operation;
  client: ClientStatus;
  refreshing: boolean;
  refreshedAt: string | null;
}

export type ProcessKind = "app" | "helper" | "languageServer" | "cli";

export interface RelatedProcess {
  pid: number;
  kind: ProcessKind;
}

/** Error shape serialized by src-tauri/src/error.rs. */
export interface BackendError {
  code: string;
  message: string;
}

export interface ClientStatus {
  activeId: string | null;
  installed: boolean;
  running: boolean;
}
