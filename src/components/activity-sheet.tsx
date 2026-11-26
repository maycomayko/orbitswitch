import { useState } from "react";
import { antigravity } from "../client";
import { displayName } from "../format";
import { t } from "../i18n";
import type { Account, Operation, SwitchStep } from "../types";
import { CheckIcon, OrbitMark } from "./icons";
import { Sheet, SheetActions } from "./sheet";
import "./activity-sheet.css";

const STEPS: SwitchStep[] = ["preparing", "closing", "writing", "launching"];

interface Props {
  operation: Exclude<Operation, { kind: "idle" }>;
  accounts: Account[];
  onCancelAdd: () => void;
  onCopySignInLink: () => Promise<void>;
}

/** Progress for whatever the backend is doing right now. */
export function ActivitySheet({ operation, accounts, onCancelAdd, onCopySignInLink }: Props) {
  const [copyState, setCopyState] = useState<"idle" | "copying" | "copied" | "failed">("idle");
  if (operation.kind === "adding") {
    return (
      <Sheet onDismiss={onCancelAdd}>
        <div className="activity-hero">
          <OrbitMark className="orbiting" width={40} height={40} />
        </div>
        <h2 className="center">{t("adding.title")}</h2>
        <p className="center">{t("adding.body", antigravity)}</p>

        <p className="sheet-note center" role="status">
          {t(copyState === "failed" ? "adding.copyFailed" : "adding.privateHint")}
        </p>
        <SheetActions>
          <button type="button" className="btn wide" onClick={onCancelAdd}>
            {t("adding.cancel")}
          </button>

          <button
            type="button"
            className="btn btn-primary wide"
            disabled={!operation.canCopyLink || copyState === "copying"}
            onClick={async () => {
              setCopyState("copying");
              try {
                await onCopySignInLink();
                setCopyState("copied");
              } catch {
                setCopyState("failed");
              }
            }}
          >
            {t(copyState === "copied" ? "adding.copied" : "adding.copyLink")}
          </button>
        </SheetActions>
      </Sheet>
    );
  }

  if (operation.kind === "switching") {
    const target = accounts.find((a) => a.id === operation.targetId);
    const current = STEPS.indexOf(operation.step);
    return (
      <Sheet>
        <h2>{target ? displayName(target.name, target.email) : ""}</h2>
        <ol className="steps">
          {STEPS.map((step, index) => (
            <li
              key={step}
              className={index < current ? "step done" : index === current ? "step now" : "step"}
            >
              <span className="step-mark">
                {index < current ? <CheckIcon /> : <span className="step-dot" />}
              </span>
              {t(`progress.${step}`, antigravity)}
            </li>
          ))}
        </ol>
      </Sheet>
    );
  }

  return (
    <Sheet>
      <div className="activity-line">
        <span className="spinner" />
        {t(`progress.${operation.kind}`, antigravity)}
      </div>
    </Sheet>
  );
}
