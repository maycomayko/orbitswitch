import { t } from "../i18n";
import type { Plan } from "../types";
import "./plan-badge.css";

export function PlanBadge({ plan }: { plan: Plan | null }) {
  if (!plan) return null;
  return (
    <span className={`plan-badge plan-${plan.kind}`} title={plan.name}>
      {plan.kind === "unknown" ? t("plan.unknown") : plan.kind.toUpperCase()}
    </span>
  );
}
