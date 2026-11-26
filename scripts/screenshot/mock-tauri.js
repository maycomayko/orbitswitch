// Stands in for Tauri's IPC so the real UI renders demo accounts in a browser.
// Only fictional names and example.com addresses: these screenshots are public.
// Query: ?scene=hero|list|confirm|switching|adding

const params = new URLSearchParams(location.search);
const scene = params.get("scene") ?? "list";
if (scene === "hero") document.documentElement.classList.add("readme-demo");
// Screenshots are English regardless of the machine's language.
Object.defineProperty(Navigator.prototype, "language", { get: () => params.get("lang") ?? "en-US" });

const MIN = 60_000;
const HOUR = 60 * MIN;
const at = (ms) => new Date(Date.now() + ms).toISOString();
const group = (key, label, models, [fiveHour, weekly, fiveReset, weekReset]) => ({
  key,
  label,
  models,
  fiveHour:
    fiveHour === null
      ? null
      : { remaining: fiveHour, resetAt: at(fiveReset), label: "5h", durationMinutes: 300 },
  weekly: { remaining: weekly, resetAt: at(weekReset), label: "weekly", durationMinutes: 10080 },
  extra: [],
});
const quota = (gemini, claude, geminiModels = ["Gemini 4 Neon", "Gemini 4 Carbon", "Gemini 4 Hydrogen"]) => ({
  status: "ready",
  fetchedAt: at(-MIN),
  stale: false,
  groups: [
    group("gemini", "Gemini", geminiModels, gemini),
    ...(claude ? [group("claude", "Claude", ["Claude Opus 5.5", "Claude Sonnet 5.5"], claude)] : []),
  ],
});
const pro = { kind: "pro", name: "Google AI Pro" };

const snapshot = {
  platform: "macos",
  version: "0.2.1",
  client: { activeId: "ada", identityStale: false, installed: true, running: true },
  network: { localHttpProxyPort: null, invalid: false },
  usageEnabled: true,
  operation:
    scene === "switching"
      ? {
          kind: "switching",
          targetId: "grace",
          step: "writing",
        }
      : scene === "adding"
        ? { kind: "adding", canCopyLink: params.get("link") !== "preparing" }
        : { kind: "idle" },
  refreshing: false,
  refreshedAt: at(-MIN),
  accounts: [
    {
      id: "ada",
      email: "ada@example.com",
      name: "Ada Lovelace",
      picture: null,
      plan: pro,
      quota: quota([0.62, 0.88, 3 * HOUR, 96 * HOUR], [0.07, 0.41, 72 * MIN, 96 * HOUR]),
    },
    {
      id: "grace",
      email: "grace@example.com",
      name: "Grace Hopper",
      picture: null,
      plan: pro,
      quota: quota([1, 0.97, 5 * HOUR, 130 * HOUR], null, ["Gemini 4 Hydrogen"]),
    },
    {
      id: "alan",
      email: "alan@example.com",
      name: "Alan Turing",
      picture: null,
      plan: { kind: "ultra", name: "Google AI Ultra" },
      quota: quota([0.74, 0.9, 2 * HOUR, 40 * HOUR], [0.31, 0.58, 3 * HOUR + 20 * MIN, 40 * HOUR]),
    },
  ],
};

if (scene === "stale") {
  const account = snapshot.accounts[0];
  account.quota.stale = true;
  (account.quota.groups[0].fiveHour ?? account.quota.groups[0].weekly).remaining = 0;
}
if (scene === "empty") snapshot.accounts = [];
if (scene === "hero") {
  snapshot.usageEnabled = false;
  snapshot.refreshedAt = null;
  snapshot.accounts = snapshot.accounts.slice(0, 2).map((account) => ({
    ...account,
    plan: null,
    quota: { status: "disabled" },
  }));
}
window.__orbitDemo = { calls: [], snapshot };
const processes = [
  { pid: 1, kind: "app" },
  { pid: 2, kind: "languageServer" },
  { pid: 3, kind: "helper" },
];

let nextCallback = 1;
window.__TAURI_INTERNALS__ = {
  metadata: { currentWindow: { label: "main" }, currentWebview: { windowLabel: "main", label: "main" } },
  transformCallback: () => nextCallback++,
  invoke: async (command, args) => {
    window.__orbitDemo.calls.push({ command, args });
    if (command === "get_snapshot") return snapshot;
    if (command === "running_processes") return processes;
    if (command === "copy_sign_in_link" && params.get("copy") === "failed")
      throw { code: "sign_in_link_unavailable", message: "test sign-in link unavailable" };
    return null;
  },
};

document.addEventListener("DOMContentLoaded", () => {
  if (document.querySelector(".clock")) document.querySelector(".clock").textContent = "Sun Oct 25  6:15 PM";
  if (scene !== "confirm") return;
  // Open the switch confirmation as soon as the recommended card renders.
  const observer = new MutationObserver(() => {
    const button = document.querySelector(".card:has(.tag-best) .card-identity");
    if (button) {
      observer.disconnect();
      button.click();
    }
  });
  observer.observe(document.body, { childList: true, subtree: true });
});
