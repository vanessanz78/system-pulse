import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./styles.css";

type HealthState = "healthy" | "attention" | "critical";
type ViewMode = "quick" | "today";
type FocusState = "green" | "yellow" | "orange" | "red";
type FocusDomain =
  | "applications"
  | "memory"
  | "processor"
  | "browser"
  | "storage"
  | "disk"
  | "desktop"
  | "system";
type StalenessStatus = "fresh" | "stale" | "unknown";
type SafetyLevel = "safe" | "caution" | "restricted" | "blocked";
type SessionPreservationRisk = "none" | "low" | "medium" | "high" | "unknown";

type DomainHealth = {
  label: string;
  headline: string;
  detail: string;
  value: string;
  metricLabel: string;
  metricPercent: string;
};

type ApplicationImpact = {
  name: string;
  memoryDisplay: string;
  cpuDisplay: string;
  impactLabel: string;
  detail: string;
  careLabel: string;
  careDetail: string;
  careEstimatedImprovement: string;
  actionKind: string;
  actionTarget: string;
  actionLabel: string;
  showOpportunity: boolean;
  protectedWork: boolean;
};

type SupportingMetric = {
  label: string;
  value: string;
};

type PredictionStaleness = {
  status: StalenessStatus;
  ageSeconds: number;
};

type MenuBarState = {
  state: FocusState;
  heartColor: string;
  minutesLabel: string;
  showsMinutes: boolean;
  criticalPulse: boolean;
};

type FocusContributor = {
  domain: FocusDomain;
  label: string;
  state: FocusState;
  risk: number;
  impactMinutes: number;
  reason: string;
  supportingMetrics: SupportingMetric[];
  protectedWork: boolean;
  actionAvailable: boolean;
};

type FocusPrediction = {
  remainingMinutes: number;
  state: FocusState;
  confidence: number;
  primaryReducer: FocusContributor | null;
  contributors: FocusContributor[];
  lastUpdated: string;
  staleness: PredictionStaleness;
  menuBarState: MenuBarState;
};

type RecoveryCandidate = {
  domain: FocusDomain;
  actionKind: string;
  target: string;
  expectedGainMinutes: number;
  estimatedInterruptionSeconds: number;
  confidence: number;
  safetyLevel: SafetyLevel;
  requiresConfirmation: boolean;
  canAutomate: boolean;
  sessionPreservationRisk: SessionPreservationRisk;
  reason: string;
  trustNotes: string;
};

type MissionAction = {
  id: string;
  title: string;
  description: string;
  confidence: string;
  confidenceReason: string;
  whyRecommended: string;
  estimatedBenefit: string;
  estimatedBenefitBytes: number;
  interruption: string;
  risk: string;
  previewItemCount: number;
  status: string;
};

type PulseMission = {
  id: string;
  category: string;
  missionTitle: string;
  title: string;
  summary: string;
  explanation: string;
  confidence: string;
  confidenceReason: string;
  status: string;
  priority: number;
  estimatedBenefit: string;
  estimatedBenefitBytes: number;
  expectedBenefit: string;
  expectedInterruption: string;
  estimatedDuration: string;
  diagnosis: string;
  recoveryPlan: string;
  actions: MissionAction[];
};

type MissionRegistrySnapshot = {
  topMission: PulseMission | null;
  otherOpportunities: PulseMission[];
};

type MissionPreviewFile = {
  name: string;
  itemKind: string;
  size: string;
  path: string;
  reason: string;
  confidence: string;
  expectedBenefit: string;
  interruption: string;
};

type MissionActionPreview = {
  actionId: string;
  title: string;
  whatIFound: string;
  whySelected: string;
  confidence: string;
  risk: string;
  interruption: string;
  estimatedRecovery: string;
  estimatedRecoveryBytes: number;
  files: MissionPreviewFile[];
  omittedCount: number;
};

type MissionActionExplanation = {
  actionId: string;
  title: string;
  reason: string;
  confidence: string;
  confidenceReason: string;
  expectedBenefit: string;
  risk: string;
  interruption: string;
};

type MissionActionRunResult = {
  actionId: string;
  title: string;
  success: boolean;
  completed: boolean;
  skipped: boolean;
  failed: boolean;
  storageBefore: string;
  storageBeforeBytes: number;
  storageAfter: string;
  storageAfterBytes: number;
  recovered: string;
  recoveredBytes: number;
  recoveredSpace: string;
  recoveredSpaceBytes: number;
  currentFreeSpace: string;
  currentFreeSpaceBytes: number;
  storageHealth: string;
  duration: string;
  durationSeconds: number;
  actionsCompleted: number;
  skippedItems: number;
  verified: boolean;
  verification: string;
  errors: string[];
};

type MissionActionDetail =
  | { kind: "preview"; preview: MissionActionPreview }
  | { kind: "explain"; explanation: MissionActionExplanation }
  | { kind: "result"; result: MissionActionRunResult };

type ActionResult = {
  actionKind: string;
  target: string;
  startedAt: string;
  completedAt: string | null;
  success: boolean;
  interruptionSeconds: number;
  beforePrediction: FocusPrediction | null;
  afterPrediction: FocusPrediction | null;
  actualGainMinutes: number | null;
  errors: string[];
  userCancelled: boolean;
};

type TodayPulse = {
  collectedAt: string;
  platform: string;
  systemScore: number;
  healthState: HealthState;
  primaryExplanation: string;
  primaryRecommendation: string;
  estimatedAdditionalWorkLabel: string;
  flowRemainingLabel: string;
  flowRemainingMinutes: number;
  memoryHealth: DomainHealth;
  storageHealth: DomainHealth;
  processorHealth: DomainHealth;
  applicationHealth: DomainHealth;
  batteryHealth?: DomainHealth;
  browserHealth?: DomainHealth;
  topApplications: ApplicationImpact[];
  focusPrediction: FocusPrediction;
  recoveryCandidates: RecoveryCandidate[];
};

type KnowledgeItem = {
  label: string;
  metric?: string;
};

const app = document.querySelector<HTMLElement>("#app");
const USER_NAME = "Vanessa";

if (!app) {
  throw new Error("System Pulse root element is missing.");
}

const appRoot = app;
const VISIBLE_REFRESH_MS = 60_000;
let currentPulse: TodayPulse | null = null;
let isRefreshing = false;
let currentView: ViewMode = "quick";
let selectedApplicationId: string | null = null;
let careMessage = "";
let autoRefreshTimer: number | undefined;
let currentPulseMission: PulseMission | null = null;
let missionLoading = false;
let missionError = "";
let missionActionDetail: MissionActionDetail | null = null;
let missionActionStatuses: Record<string, string> = {};

type CareMuteState = Record<
  string,
  {
    remindUntil?: number;
    ignoredOn?: string;
  }
>;

const CARE_STATE_KEY = "system-pulse-care-state-v1";
const MISSION_LATER_KEY = "system-pulse-mission-later-until";
const MISSION_TELEMETRY_KEY = "system-pulse-mission-telemetry-v1";

type LocalMissionTelemetryEvent = {
  missionId: string;
  actionId?: string;
  event: "Mission started" | "Mission completed" | "Mission cancelled" | "Mission deferred";
  timestamp: string;
  durationMs?: number;
  verification?: string;
};

function escapeHtml(value: string): string {
  return value.replace(/[&<>"']/g, (character) => {
    switch (character) {
      case "&":
        return "&amp;";
      case "<":
        return "&lt;";
      case ">":
        return "&gt;";
      case "\"":
        return "&quot;";
      case "'":
        return "&#39;";
      default:
        return character;
    }
  });
}

function todayKey(): string {
  return new Date().toISOString().slice(0, 10);
}

function actionId(application: ApplicationImpact): string {
  return `${application.actionKind}:${application.actionTarget || application.name}`;
}

function readCareState(): CareMuteState {
  try {
    const raw = localStorage.getItem(CARE_STATE_KEY);
    if (!raw) return {};
    return JSON.parse(raw) as CareMuteState;
  } catch {
    return {};
  }
}

function writeCareState(state: CareMuteState): void {
  localStorage.setItem(CARE_STATE_KEY, JSON.stringify(state));
}

function isMissionDeferred(): boolean {
  const value = localStorage.getItem(MISSION_LATER_KEY);
  if (!value) return false;
  const laterUntil = Number.parseInt(value, 10);
  return Number.isFinite(laterUntil) && laterUntil > Date.now();
}

function deferMission(): void {
  localStorage.setItem(MISSION_LATER_KEY, String(Date.now() + 30 * 60 * 1000));
}

function recordMissionTelemetry(event: LocalMissionTelemetryEvent): void {
  try {
    const existing = localStorage.getItem(MISSION_TELEMETRY_KEY);
    const events = existing ? (JSON.parse(existing) as LocalMissionTelemetryEvent[]) : [];
    events.push(event);
    localStorage.setItem(MISSION_TELEMETRY_KEY, JSON.stringify(events.slice(-50)));
  } catch {
    // Telemetry is local-only and best-effort; mission execution must not depend on it.
  }
}

function shouldHideOpportunity(application: ApplicationImpact): boolean {
  const state = readCareState()[actionId(application)];
  if (!state) return false;
  if (state.ignoredOn === todayKey()) return true;
  return Boolean(state.remindUntil && state.remindUntil > Date.now());
}

function focusTimeLabel(pulse: TodayPulse): string {
  const menuLabel = pulse.focusPrediction.menuBarState.minutesLabel;
  if (menuLabel) return menuLabel;

  const minutes = pulse.focusPrediction.remainingMinutes;
  if (minutes >= 60) {
    const hours = Math.floor(minutes / 60);
    const remainingMinutes = minutes % 60;
    return `${hours}h ${remainingMinutes.toString().padStart(2, "0")}m`;
  }
  return `${minutes}m`;
}

function companionHeadline(state: HealthState): string {
  if (state === "critical") return "Care is needed soon.";
  return `${dayGreeting()}, ${USER_NAME}.`;
}

function companionDetailLine(state: HealthState): string {
  if (state === "healthy") return "You're good to keep working.";
  if (state === "attention") return "Finish what you're doing first.";
  return "Take a care moment when you can.";
}

function focusLine(state: HealthState): string {
  if (state === "healthy") return "You're in a good place to focus.";
  if (state === "attention") return "You're okay to keep going for now.";
  return "Let's make a little room before deep work.";
}

function domainNeedsCare(domain: DomainHealth): boolean {
  const label = domain.label.toLowerCase();
  return label !== "ok" && label !== "healthy";
}

function signalClass(domain: DomainHealth): string {
  return domainNeedsCare(domain) ? "needs-care" : "ok";
}

function careMessageHtml(): string {
  if (!careMessage) return "";
  return `<p class="care-message" role="status">${escapeHtml(careMessage)}</p>`;
}

function missionMetric(label: string, value: string): string {
  return `
    <span>
      <small>${escapeHtml(label)}</small>
      <strong>${escapeHtml(value)}</strong>
    </span>
  `;
}

function missionActionLocalId(actionId: string): string {
  return actionId.split(":").pop() || actionId;
}

function missionActionIdLabel(actionId: string): string {
  const localId = missionActionLocalId(actionId);
  if (localId === "empty-trash") return "Trash";
  if (localId === "delete-downloaded-installers") return "Old installers";
  if (localId === "clear-obsolete-caches") return "Temporary files";
  return "Mission";
}

function missionActionTitle(action: MissionAction): string {
  const localId = missionActionLocalId(action.id);
  if (localId === "delete-downloaded-installers") return "Remove old installers";
  if (localId === "clear-obsolete-caches") return "Clean temporary files";
  return action.title;
}

function missionActionButtonLabel(action: MissionAction): string {
  const localId = missionActionLocalId(action.id);
  if (missionActionStatus(action) === "Running") return "Cleaning...";
  if (missionActionStatus(action) === "Completed") return "Done";
  if (localId === "empty-trash") return "Empty now";
  if (localId === "delete-downloaded-installers") return "Remove now";
  return "Clean now";
}

function missionActionWhatWillHappen(action: MissionAction): string {
  const localId = missionActionLocalId(action.id);
  if (localId === "empty-trash") return "Items already in Trash will be permanently removed.";
  if (localId === "delete-downloaded-installers") return "Old installer files in Downloads will be removed. Installed apps and documents stay where they are.";
  if (localId === "clear-obsolete-caches") return "Temporary files from applications that no longer need them will be removed.";
  return action.description;
}

function missionActionSummary(action: MissionAction): string {
  const localId = missionActionLocalId(action.id);
  if (localId === "empty-trash") return "Items already placed in Trash.";
  if (localId === "delete-downloaded-installers") return "Old app installers that are usually only needed once.";
  if (localId === "clear-obsolete-caches") return "Temporary files your applications can recreate automatically.";
  return action.description;
}

function safetyLabel(confidence: string): string {
  const normalized = confidence.toLowerCase();
  if (normalized.includes("high")) return "Very safe";
  if (normalized.includes("medium")) return "Safe";
  return "Needs review";
}

function safetyClass(confidence: string): string {
  const normalized = confidence.toLowerCase();
  if (normalized.includes("high")) return "very-safe";
  if (normalized.includes("medium")) return "safe";
  return "needs-review";
}

function safetySentence(confidence: string): string {
  const label = safetyLabel(confidence);
  if (label === "Very safe") return "These files are already discarded or can be removed without touching personal work.";
  if (label === "Safe") return "Nothing personal will be removed. Some apps may rebuild files later if they need them.";
  return "Review the details before cleaning.";
}

function actionNotice(action: MissionAction): string {
  const localId = missionActionLocalId(action.id);
  if (localId === "clear-obsolete-caches") {
    return "Applications may open slightly slower the first time after cleaning.";
  }
  return action.interruption === "None" ? "Nothing should interrupt your work." : action.interruption;
}

function isMissionActionExpanded(actionId: string): boolean {
  return Boolean(
    missionActionDetail &&
      (missionActionDetail.kind === "preview"
        ? missionActionDetail.preview.actionId === actionId
        : missionActionDetail.kind === "explain"
          ? missionActionDetail.explanation.actionId === actionId
      : missionActionDetail.result.actionId === actionId),
  );
}

function missionPreviewFilesHtml(preview: MissionActionPreview): string {
  const friendlyFiles = preview.files.length
    ? preview.files
        .map(
          (file) => `
            <li>
              <span>
                <strong>${escapeHtml(file.name)}</strong>
                <small>${escapeHtml(file.size)}</small>
              </span>
              <span class="storage-file-plain">${escapeHtml(file.reason)}</span>
            </li>
          `,
        )
        .join("")
    : `<li><span><strong>No files found for this action.</strong></span></li>`;
  const technicalFiles = preview.files.length
    ? preview.files
        .map(
          (file) => `
            <li>
              <span>
                <small>${escapeHtml(file.itemKind)}</small>
                <strong>${escapeHtml(file.name)}</strong>
                <small>${escapeHtml(file.path)}</small>
                <span class="storage-file-reason">${escapeHtml(file.reason)}</span>
              </span>
              <em>
                ${escapeHtml(file.size)}
                <small>${escapeHtml(safetyLabel(file.confidence))}</small>
              </em>
            </li>
          `,
        )
        .join("")
    : `<li><span><strong>No technical file details available.</strong></span></li>`;
  const omitted = preview.omittedCount
    ? `<p class="storage-preview-note">There are ${preview.omittedCount} more item${preview.omittedCount === 1 ? "" : "s"} not shown here.</p>`
    : "";

  return `
    <details class="storage-disclosure">
      <summary>Files affected</summary>
      <ul class="storage-file-list friendly-file-list">${friendlyFiles}</ul>
      ${omitted}
    </details>
    <details class="storage-disclosure">
      <summary>Show technical details</summary>
      <ul class="storage-file-list">${technicalFiles}</ul>
    </details>
  `;
}

function missionActionStatus(action: MissionAction): string {
  return missionActionStatuses[action.id] || action.status || "Ready";
}

function missionStatus(plan: PulseMission): string {
  const statuses = plan.actions.map(missionActionStatus);
  if (statuses.includes("Running")) return "Running";
  if (statuses.includes("Completed")) return "Completed";
  if (statuses.includes("Deferred")) return "Deferred";
  return plan.status || "Ready";
}

function missionActionDetailHtml(detail: MissionActionDetail | null, action?: MissionAction): string {
  if (!detail) return "";

  if (detail.kind === "preview") {
    const preview = detail.preview;
    const notice = action ? actionNotice(action) : preview.interruption;

    return `
      <section class="storage-action-detail" aria-label="${escapeHtml(preview.title)} preview">
        <p class="care-task-label">What I'm going to clean</p>
        <h3>${escapeHtml(action ? missionActionSummary(action) : preview.whatIFound)}</h3>
        <p>Nothing personal will be removed.</p>
        <dl class="storage-explain-list friendly-explain-list">
          <div><dt>Why this is recommended</dt><dd>${escapeHtml(preview.whySelected)}</dd></div>
          <div><dt>What will happen</dt><dd>${escapeHtml(action ? missionActionWhatWillHappen(action) : preview.whatIFound)}</dd></div>
          <div><dt>You'll recover</dt><dd>${escapeHtml(preview.estimatedRecovery)}</dd></div>
          <div><dt>Safety</dt><dd><span class="safety-pill ${escapeHtml(safetyClass(preview.confidence))}">${escapeHtml(safetyLabel(preview.confidence))}</span> ${escapeHtml(safetySentence(preview.confidence))}</dd></div>
          <div><dt>What you'll notice</dt><dd>${escapeHtml(notice)}</dd></div>
        </dl>
        ${missionPreviewFilesHtml(preview)}
      </section>
    `;
  }

  if (detail.kind === "explain") {
    const explanation = detail.explanation;
    return `
      <section class="storage-action-detail" aria-label="${escapeHtml(explanation.title)} explanation">
        <p class="care-task-label">Why?</p>
        <h3>${escapeHtml(explanation.title)}</h3>
        <dl class="storage-explain-list">
          <div><dt>Reason</dt><dd>${escapeHtml(explanation.reason)}</dd></div>
          <div><dt>Safety</dt><dd><span class="safety-pill ${escapeHtml(safetyClass(explanation.confidence))}">${escapeHtml(safetyLabel(explanation.confidence))}</span> ${escapeHtml(explanation.confidenceReason)}</dd></div>
          <div><dt>You'll recover</dt><dd>${escapeHtml(explanation.expectedBenefit)}</dd></div>
          <div><dt>Risk</dt><dd>${escapeHtml(explanation.risk)}</dd></div>
          <div><dt>What you'll notice</dt><dd>${escapeHtml(explanation.interruption)}</dd></div>
        </dl>
      </section>
    `;
  }

  const result = detail.result;
  const errors = result.errors.length
    ? `
      <ul class="storage-result-errors">
        ${result.errors.map((error) => `<li>${escapeHtml(error)}</li>`).join("")}
      </ul>
    `
    : "";

  return `
    <section class="storage-action-detail storage-result" aria-label="${escapeHtml(result.title)} result">
      <p class="care-task-label">${result.success ? "Done" : "Needs review"}</p>
      <h3>${result.success ? `&#10003; ${escapeHtml(result.title)} complete.` : "Partly complete"}</h3>
      <div class="storage-result-grid">
        ${missionMetric("Recovered", result.recovered)}
        ${missionMetric("Available now", result.currentFreeSpace)}
        ${missionMetric("Storage health", result.storageHealth)}
      </div>
      <p>${result.verified ? `Your Mac now has ${escapeHtml(result.currentFreeSpace)} available.` : "The action ran, but macOS has not reported the extra free space yet."}</p>
      <details class="storage-disclosure">
        <summary>Show technical details</summary>
        <div class="storage-result-grid technical-result-grid">
          ${missionMetric("Storage before", result.storageBefore)}
          ${missionMetric("Storage after", result.storageAfter)}
          ${missionMetric("Duration", result.duration)}
          ${missionMetric("Items cleaned", result.actionsCompleted.toString())}
          ${missionMetric("Skipped items", result.skippedItems.toString())}
          ${missionMetric("Verification", result.verified ? "Passed" : "Pending")}
        </div>
      </details>
      ${errors}
    </section>
  `;
}

function missionPlanHtml(plan: PulseMission): string {
  if (!plan.actions.length) {
    return `
      <section class="summary-section care-panel calm-panel">
        <p class="panel-kicker recommended-kicker"><span aria-hidden="true">&#9733;</span> ${escapeHtml(plan.missionTitle)}</p>
        <h2>${missionActionDetail ? `${escapeHtml(plan.category)} mission complete` : `No ${escapeHtml(plan.category.toLowerCase())} mission needed`}</h2>
        ${careMessageHtml()}
        <p class="summary-answer">${escapeHtml(plan.explanation)}</p>
        ${missionActionDetailHtml(missionActionDetail)}
      </section>
    `;
  }

  const actions = plan.actions
    .map((action) => {
      const status = missionActionStatus(action);
      const expanded = isMissionActionExpanded(action.id);
      const running = status === "Running";
      const completed = status === "Completed";
      const detail =
        expanded && missionActionDetail?.kind !== "result"
          ? missionActionDetailHtml(missionActionDetail, action)
          : "";
      return `
        <div class="storage-action-row ${expanded ? "is-expanded" : ""}" data-mission-card="${escapeHtml(action.id)}">
          <div class="storage-action-copy">
            <span class="mission-timeline-dot" data-status="${escapeHtml(status.toLowerCase())}" aria-hidden="true"></span>
            <p class="care-task-label">${escapeHtml(missionActionIdLabel(action.id))}</p>
            <strong>${escapeHtml(missionActionTitle(action))}</strong>
            <p class="storage-action-subtitle">Recover ${escapeHtml(action.estimatedBenefit)}</p>
            <p class="safety-line ${escapeHtml(safetyClass(action.confidence))}">
              <span aria-hidden="true"></span>
              <strong>${escapeHtml(safetyLabel(action.confidence))}</strong>
              <small>${escapeHtml(safetySentence(action.confidence))}</small>
            </p>
            <p class="care-task-detail">${escapeHtml(missionActionSummary(action))}</p>
            ${running ? `<div class="storage-progress" aria-label="Cleaning"><span></span></div>` : ""}
          </div>
          <div class="storage-action-buttons">
            <button
              class="recommended-primary-button"
              type="button"
              data-mission-run="${escapeHtml(action.id)}"
              ${running || completed ? "disabled" : ""}
            >
              ${escapeHtml(missionActionButtonLabel(action))}
            </button>
            <button
              class="storage-disclosure-button"
              type="button"
              data-mission-toggle-details="${escapeHtml(action.id)}"
              aria-expanded="${expanded ? "true" : "false"}"
              aria-label="${expanded ? "Hide details" : "Show details"} for ${escapeHtml(missionActionTitle(action))}"
            >
              ${expanded ? "Hide details" : "Details"}
            </button>
          </div>
          ${detail}
        </div>
      `;
    })
    .join("");

  const resultDetail = missionActionDetail?.kind === "result" ? missionActionDetailHtml(missionActionDetail) : "";

  return `
    <section class="summary-section care-panel attention-panel storage-recovery-panel">
      <p class="panel-kicker recommended-kicker"><span aria-hidden="true">&#9733;</span> ${escapeHtml(plan.missionTitle)}</p>
      <h2>${escapeHtml(plan.title)}</h2>
      <p class="care-plan-intro">${escapeHtml(plan.summary)}</p>
      ${careMessageHtml()}
      <div class="storage-recovery-metrics">
        ${missionMetric("Status", missionStatus(plan))}
        ${missionMetric("Safety", safetyLabel(plan.confidence))}
        ${missionMetric("Estimated time", plan.estimatedDuration)}
        ${missionMetric("You'll recover", plan.expectedBenefit)}
        ${missionMetric("What you'll notice", plan.expectedInterruption)}
      </div>
      <p class="mission-confidence-note">${escapeHtml(plan.confidenceReason)}</p>
      <div class="storage-action-list" aria-label="Mission actions">
        ${actions}
      </div>
      ${resultDetail}
    </section>
  `;
}

function missionLoadingHtml(): string {
  return `
    <section class="summary-section care-panel attention-panel storage-recovery-panel">
      <p class="panel-kicker recommended-kicker"><span aria-hidden="true">&#9733;</span> Recovery plan</p>
      <h2>Checking today's best mission.</h2>
      ${careMessageHtml()}
      <p class="care-plan-intro">System Pulse is checking the registered missions and ranking the safest useful option. Nothing changes on your Mac.</p>
    </section>
  `;
}

function missionErrorHtml(): string {
  return `
    <section class="summary-section care-panel attention-panel storage-recovery-panel">
      <p class="panel-kicker recommended-kicker"><span aria-hidden="true">&#9733;</span> Recovery plan</p>
      <h2>Today's mission could not be checked yet.</h2>
      ${careMessageHtml()}
      <p class="care-plan-intro">${escapeHtml(missionError)}</p>
      <button class="recommended-secondary-button" type="button" data-mission-retry>Try again</button>
    </section>
  `;
}

function visibleApplicationOpportunities(pulse: TodayPulse): ApplicationImpact[] {
  return pulse.topApplications
    .filter((application) => application.showOpportunity)
    .filter((application) => application.actionKind !== "none")
    .filter((application) => !shouldHideOpportunity(application))
    .slice(0, 3);
}

function dayGreeting(): string {
  const hour = new Date().getHours();
  if (hour < 12) return "Good morning";
  if (hour < 18) return "Good afternoon";
  return "Good evening";
}

function healthyBatteryFallback(): DomainHealth {
  return {
    label: "Healthy",
    headline: "Battery is healthy",
    detail: "",
    value: "Healthy",
    metricLabel: "Battery OK",
    metricPercent: "",
  };
}

function companionStatusLabel(domain: DomainHealth): string {
  return domainNeedsCare(domain) ? "Needs attention" : "Good";
}

function companionSignal(label: string, icon: string, domain: DomainHealth): string {
  return `
    <li class="${signalClass(domain)}">
      <span class="status-dot" aria-hidden="true"></span>
      <span class="status-icon status-icon-${escapeHtml(icon)}" aria-hidden="true"></span>
      <span class="status-name">${escapeHtml(label)}</span>
      <strong>${companionStatusLabel(domain)}</strong>
    </li>
  `;
}

function companionGlance(pulse: TodayPulse): string {
  const items = [
    companionSignal("Applications", "apps", pulse.applicationHealth),
    companionSignal("Memory", "memory", pulse.memoryHealth),
    companionSignal("Processor", "processor", pulse.processorHealth),
    companionSignal("Browser", "browser", pulse.browserHealth ?? healthyBatteryFallback()),
  ].join("");

  return `
    <div class="companion-glance" aria-label="At a glance">
      <ul>${items}</ul>
    </div>
  `;
}

function firstRecommendedApplication(pulse: TodayPulse): ApplicationImpact | undefined {
  return visibleApplicationOpportunities(pulse)[0];
}

function statusDetail(domain: DomainHealth, healthyDetail: string): string {
  if (domainNeedsCare(domain)) {
    return domain.detail || domain.headline || domain.value;
  }
  return domain.detail || domain.value || healthyDetail;
}

function todayStatusCard(label: string, icon: string, domain: DomainHealth, healthyDetail: string): string {
  const status = companionStatusLabel(domain);
  const detail = statusDetail(domain, healthyDetail);
  return `
    <section class="today-status-card ${signalClass(domain)}" aria-label="${escapeHtml(label)} ${status}">
      <span class="card-signal-dot" aria-hidden="true"></span>
      <span class="status-card-icon status-icon status-icon-${escapeHtml(icon)}" aria-hidden="true"></span>
      <h2>${escapeHtml(label)}</h2>
      <strong>${escapeHtml(status)}</strong>
      <p class="card-detail">${escapeHtml(detail)}</p>
      <p class="card-metric">
        <span class="metric-pulse" aria-hidden="true"></span>
        <span>${escapeHtml(domain.metricLabel || domain.value)}</span>
        ${domain.metricPercent ? `<strong>${escapeHtml(domain.metricPercent)}</strong>` : ""}
      </p>
      <details class="status-why">
        <summary>Why?</summary>
        <p>${escapeHtml(detail)}</p>
      </details>
    </section>
  `;
}

function todayStatusCards(pulse: TodayPulse): string {
  return `
    <div class="today-status-grid" aria-label="System status">
      ${todayStatusCard("Applications", "apps", pulse.applicationHealth, "Everything looks clear.")}
      ${todayStatusCard("Memory", "memory", pulse.memoryHealth, "Pressure is low.")}
      ${todayStatusCard("Processor", "processor", pulse.processorHealth, "Processor has room.")}
      ${todayStatusCard("Browser", "browser", pulse.browserHealth ?? healthyBatteryFallback(), "Everything looks clear.")}
    </div>
  `;
}

function applicationUsageItems(pulse: TodayPulse): KnowledgeItem[] {
  const items = pulse.topApplications.slice(0, 6).map((application) => ({
    label: application.name,
    metric: application.detail || application.impactLabel || "Running normally.",
  }));
  return items.length ? items : [{ label: "No non-browser application is standing out." }];
}

function applicationUsageList(pulse: TodayPulse): string {
  const items = applicationUsageItems(pulse)
    .map(
      (item) => `
        <li>
          <span>
            <strong>${escapeHtml(item.label)}</strong>
            ${item.metric ? `<small>${escapeHtml(item.metric)}</small>` : ""}
          </span>
        </li>
      `,
    )
    .join("");
  return `
    <section class="summary-section">
      <h2>Application usage</h2>
      <ul class="summary-list application-usage-list">${items}</ul>
    </section>
  `;
}

function quietCareLabel(application: ApplicationImpact): string {
  if (application.actionKind === "restartApp") return `Restart ${application.name}`;
  if (application.actionKind === "quitApp") return `Quit ${application.name}`;
  if (application.actionKind === "restartFinder") return "Restart Finder";
  return application.actionLabel || application.careLabel;
}

function primaryCareButtonLabel(application: ApplicationImpact): string {
  if (application.actionKind === "restartApp") return "Restart";
  if (application.actionKind === "quitApp") return "Close";
  if (application.actionKind === "restartFinder") return "Restart";
  return application.actionLabel || "Start";
}

function applicationCareTask(application: ApplicationImpact): string {
  return `
    <div class="care-task">
      <span class="care-task-icon status-icon status-icon-apps" aria-hidden="true"></span>
      <div class="care-task-copy">
        <p class="care-task-label">Application</p>
        <strong>${escapeHtml(application.name)}</strong>
        <small>${escapeHtml(application.memoryDisplay)} RAM · ${escapeHtml(application.cpuDisplay)} CPU</small>
        <p class="care-task-detail">${escapeHtml(application.careDetail)}</p>
        <p class="care-task-benefit">Expected benefit ${escapeHtml(application.careEstimatedImprovement)}</p>
      </div>
      <div class="care-task-actions">
        <button
          class="recommended-primary-button"
          type="button"
          data-care-action="${escapeHtml(actionId(application))}"
        >
          ${escapeHtml(primaryCareButtonLabel(application))}
        </button>
        <button
          class="recommended-secondary-button"
          type="button"
          data-care-remind="${escapeHtml(actionId(application))}"
        >
          Later
        </button>
      </div>
    </div>
  `;
}

function browserNameFromDomain(domain: DomainHealth): string {
  const valueName = domain.value.split(":")[0]?.trim();
  if (valueName && valueName !== domain.value) return valueName;
  const headlineName = domain.headline.match(/^(.+?) (looks|is|may|needs)/)?.[1];
  return headlineName || "Browser";
}

function browserCareTask(domain: DomainHealth): string {
  const browserName = browserNameFromDomain(domain);
  const actionKind = browserName === "Safari" ? "quitApp" : "restartApp";
  const actionLabel = browserName === "Safari" ? "Quit" : "Restart";
  return `
    <div class="care-task">
      <span class="care-task-icon status-icon status-icon-browser" aria-hidden="true"></span>
      <div class="care-task-copy">
        <p class="care-task-label">Browser</p>
        <strong>${escapeHtml(browserName)}</strong>
        <small>${escapeHtml(domain.metricLabel || domain.value)}</small>
        <p class="care-task-detail">${escapeHtml(domain.detail)}</p>
        <p class="care-task-benefit">What you'll notice: about 20 seconds. Expected benefit: +35 minutes.</p>
      </div>
      <div class="care-task-actions">
        <button
          class="recommended-primary-button"
          type="button"
          data-browser-action="${escapeHtml(actionKind)}"
          data-browser-target="${escapeHtml(browserName)}"
        >
          ${escapeHtml(actionLabel)}
        </button>
      </div>
    </div>
  `;
}

function domainCareTask(
  label: string,
  icon: string,
  domain: DomainHealth,
  actionLabel: string,
  actionKind: string,
): string {
  return `
    <div class="care-task">
      <span class="care-task-icon status-icon status-icon-${escapeHtml(icon)}" aria-hidden="true"></span>
      <div class="care-task-copy">
        <p class="care-task-label">${escapeHtml(label)}</p>
        <strong>${escapeHtml(domain.headline)}</strong>
        <small>${escapeHtml(domain.metricLabel || domain.value)}</small>
        <p class="care-task-detail">${escapeHtml(domain.detail)}</p>
        <p class="care-task-benefit">Maintenance, not an immediate flow interruption</p>
      </div>
      <div class="care-task-actions">
        <button class="recommended-primary-button" type="button" data-domain-action="${escapeHtml(actionKind)}">
          ${escapeHtml(actionLabel)}
        </button>
      </div>
    </div>
  `;
}

function careTasks(pulse: TodayPulse): string[] {
  const tasks: string[] = [];
  const application = firstRecommendedApplication(pulse);
  if (application) tasks.push(applicationCareTask(application));
  if (pulse.browserHealth && domainNeedsCare(pulse.browserHealth)) {
    tasks.push(browserCareTask(pulse.browserHealth));
  }
  return tasks.slice(0, 4);
}

function quickSuggestion(pulse: TodayPulse): string {
  const application = firstRecommendedApplication(pulse);
  if (!application) return "";

  return `
    <div class="quick-suggestion-section">
      <p>One suggestion</p>
      <button
        class="quick-suggestion-card"
        type="button"
        data-quick-detail-action="${escapeHtml(actionId(application))}"
      >
        <span class="suggestion-icon status-icon status-icon-browser" aria-hidden="true"></span>
        <span>
          <strong>${escapeHtml(quietCareLabel(application))}</strong>
          <small>at your next break.</small>
        </span>
        <span aria-hidden="true">&rsaquo;</span>
      </button>
    </div>
  `;
}

function quietApplicationButtons(application: ApplicationImpact, includePrimary = false): string {
  const primary = includePrimary
    ? `
      <button
        class="quiet-action-button primary-quiet-action"
        type="button"
        data-care-action="${escapeHtml(actionId(application))}"
      >
        ${escapeHtml(quietCareLabel(application))}
      </button>
    `
    : "";
  return `
    <div class="quiet-action-row">
      ${primary}
      <button
        class="quiet-action-button"
        type="button"
        data-care-remind="${escapeHtml(actionId(application))}"
      >
        Later
      </button>
      <button
        class="quiet-action-button"
        type="button"
        data-care-ignore="${escapeHtml(actionId(application))}"
      >
        Ignore
      </button>
    </div>
  `;
}

function recommendedCare(pulse: TodayPulse): string {
  if (!isMissionDeferred()) {
    if (currentPulseMission && (currentPulseMission.actions.length || missionActionDetail)) {
      return missionPlanHtml(currentPulseMission);
    }
    if (missionLoading) {
      return missionLoadingHtml();
    }
    if (missionError) {
      return missionErrorHtml();
    }
  }

  const tasks = careTasks(pulse);
  if (tasks.length) {
    return `
      <section class="summary-section care-panel attention-panel">
        <p class="panel-kicker recommended-kicker"><span aria-hidden="true">&#9733;</span> Recovery plan</p>
        <p class="care-plan-intro">Least disruption first. Only the useful step is shown.</p>
        ${careMessageHtml()}
        <div class="care-task-list">
          ${tasks.join("")}
        </div>
      </section>
    `;
  }

  if (pulse.healthState !== "healthy" || pulse.primaryRecommendation !== "No action needed right now.") {
    return `
      <section class="summary-section care-panel attention-panel">
        <p class="panel-kicker recommended-kicker"><span aria-hidden="true">&#9733;</span> Recovery plan</p>
        ${careMessageHtml()}
        <div class="decision-care-summary">
          <p class="care-task-label">What is happening</p>
          <strong>${escapeHtml(pulse.primaryRecommendation)}</strong>
          <p>${escapeHtml(pulse.primaryExplanation)}</p>
          <p class="care-task-label">Estimated benefit</p>
          <strong>${escapeHtml(pulse.estimatedAdditionalWorkLabel)}</strong>
          <p class="summary-answer">No safe one-click action yet. Finish what you're doing first.</p>
        </div>
      </section>
    `;
  }

  return `
    <section class="summary-section care-panel calm-panel">
      <h2>No action needed</h2>
      ${careMessageHtml()}
      <p class="summary-answer">Nothing right now.</p>
    </section>
  `;
}

function reassuranceStrip(): string {
  return `
    <section class="calm-strip" aria-label="Reassurance">
      <span class="horizon-mark" aria-hidden="true"></span>
      <div>
        <strong>Everything else looks good.</strong>
        <p>You're all set to keep going.</p>
      </div>
    </section>
  `;
}

function todaySummary(pulse: TodayPulse): string {
  return `
    <main class="today-summary" aria-label="Today's Plan">
      <section class="today-hero">
        <div class="summary-pulse" aria-label="System Pulse score ${pulse.systemScore}">
          <strong>${pulse.systemScore}</strong>
        </div>

        <div class="today-hero-copy">
          <div class="summary-intro">
            <h1>${dayGreeting()}, ${USER_NAME}.</h1>
            <p>${focusLine(pulse.healthState)}</p>
          </div>

          <section class="summary-time">
            <span>Estimated uninterrupted work time</span>
            <strong>${escapeHtml(focusTimeLabel(pulse))}</strong>
            <p>${pulse.healthState === "critical" ? "A short care moment will help." : "Plenty of time for deep work."}</p>
          </section>
        </div>
      </section>

      ${todayStatusCards(pulse)}

      <div class="today-panels">
        ${applicationUsageList(pulse)}
        ${recommendedCare(pulse)}
      </div>

      ${reassuranceStrip()}
    </main>
  `;
}

function applicationDetail(application: ApplicationImpact): string {
  return `
    <main class="application-detail-panel" aria-label="${escapeHtml(application.name)} details">
      <button id="summary-view-button" class="detail-back-button" type="button">Today</button>

      <section class="detail-hero">
        <span class="detail-icon status-icon status-icon-browser" aria-hidden="true"></span>
        <div class="summary-intro">
          <p class="eyebrow">Running</p>
          <h1>${escapeHtml(application.name)}</h1>
          <p>${escapeHtml(application.detail || application.impactLabel)}</p>
        </div>
      </section>

      <section class="summary-section">
        <h2>${escapeHtml(application.careLabel)}</h2>
        <p>${escapeHtml(application.careDetail)}</p>
      </section>

      <section class="summary-time compact-time">
        <span>Estimated benefit</span>
        <strong>${escapeHtml(application.careEstimatedImprovement)}</strong>
      </section>

      ${quietApplicationButtons(application, true)}
    </main>
  `;
}

function renderCurrentView(pulse: TodayPulse, refreshing = false): void {
  if (currentView === "today") {
    renderToday(pulse, refreshing);
    return;
  }

  renderQuickCheckin(pulse, refreshing);
}

function renderQuickCheckin(pulse: TodayPulse, _refreshing = false): void {
  appRoot.innerHTML = `
    <main class="quick-shell" data-state="${pulse.healthState}">
      <section class="quick-card" aria-label="System Pulse Companion">
        <span class="companion-gear" aria-hidden="true">&#9881;</span>

        <div class="companion-hero">
          <div class="companion-score" aria-label="System Pulse score ${pulse.systemScore}">
            <strong>${pulse.systemScore}</strong>
          </div>

          <div class="companion-copy">
            <h1>${companionHeadline(pulse.healthState)}</h1>
            <p>${companionDetailLine(pulse.healthState)}</p>
          </div>
        </div>

        <div class="companion-time">
          <span>You have</span>
          <strong>${escapeHtml(focusTimeLabel(pulse))}</strong>
          <p>of uninterrupted work time</p>
        </div>

        <div class="companion-glance-section">
          <h2>At a Glance</h2>
          ${companionGlance(pulse)}
        </div>

        ${quickSuggestion(pulse)}

        <button id="open-today-button" class="open-today-button" type="button">
          Open Today
          <span aria-hidden="true">&rsaquo;</span>
        </button>
      </section>
    </main>
  `;

  document.querySelector<HTMLButtonElement>("#open-today-button")?.addEventListener("click", () => {
    currentView = "today";
    selectedApplicationId = null;
    void invoke("open_today_window");
    renderToday(pulse);
  });
  document.querySelector<HTMLButtonElement>("[data-quick-detail-action]")?.addEventListener("click", (event) => {
    const button = event.currentTarget as HTMLButtonElement;
    currentView = "today";
    selectedApplicationId = button.dataset.quickDetailAction || null;
    void invoke("open_today_window");
    renderToday(pulse);
  });
}

function renderToday(pulse: TodayPulse, _refreshing = false): void {
  const selectedApplication = selectedApplicationId
    ? findApplicationByActionId(pulse, selectedApplicationId)
    : undefined;

  appRoot.innerHTML = `
    <div class="shell today-shell" data-state="${pulse.healthState}">
      <section class="today-window" aria-label="System Pulse">
        <header class="today-titlebar">
          <span class="window-lights" aria-hidden="true">
            <span></span><span></span><span></span>
          </span>
          <span>System Pulse</span>
          <button id="today-refresh-button" class="today-refresh-button" type="button" aria-label="Refresh">&#8635;</button>
        </header>
        <div class="today-window-body">
          ${selectedApplication ? applicationDetail(selectedApplication) : todaySummary(pulse)}
        </div>
      </section>
    </div>
  `;

  document.querySelector<HTMLButtonElement>("#summary-view-button")?.addEventListener("click", () => {
    selectedApplicationId = null;
    renderToday(pulse);
  });
  document.querySelector<HTMLButtonElement>("#today-refresh-button")?.addEventListener("click", () => {
    void loadToday({ keepExisting: true });
  });
  wireCareActions(pulse);
  void ensurePulseMission();
}

function findApplicationByActionId(pulse: TodayPulse, id: string): ApplicationImpact | undefined {
  return pulse.topApplications.find((application) => actionId(application) === id);
}

function wireCareActions(pulse: TodayPulse): void {
  document.querySelectorAll<HTMLButtonElement>("[data-care-action]").forEach((button) => {
    button.addEventListener("click", () => {
      const id = button.dataset.careAction;
      const application = id ? findApplicationByActionId(pulse, id) : undefined;
      if (application) {
        void performApplicationAction(application);
      }
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-care-remind]").forEach((button) => {
    button.addEventListener("click", () => {
      const id = button.dataset.careRemind;
      const application = id ? findApplicationByActionId(pulse, id) : undefined;
      if (!application) return;

      const state = readCareState();
      state[actionId(application)] = {
        remindUntil: Date.now() + 30 * 60 * 1000,
      };
      writeCareState(state);
      careMessage = `${application.name} will reappear in 30 minutes.`;
      selectedApplicationId = null;
      renderToday(pulse);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-care-ignore]").forEach((button) => {
    button.addEventListener("click", () => {
      const id = button.dataset.careIgnore;
      const application = id ? findApplicationByActionId(pulse, id) : undefined;
      if (!application) return;

      const state = readCareState();
      state[actionId(application)] = {
        ignoredOn: todayKey(),
      };
      writeCareState(state);
      careMessage = `${application.name} hidden for today.`;
      selectedApplicationId = null;
      renderToday(pulse);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-browser-action]").forEach((button) => {
    button.addEventListener("click", () => {
      const actionKind = button.dataset.browserAction;
      const target = button.dataset.browserTarget;
      if (actionKind && target) {
        void performBrowserAction(actionKind, target);
      }
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-detail-action]").forEach((button) => {
    button.addEventListener("click", () => {
      selectedApplicationId = button.dataset.detailAction || null;
      renderToday(pulse);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-domain-action]").forEach((button) => {
    button.addEventListener("click", () => {
      const actionKind = button.dataset.domainAction;
      if (actionKind) {
        void performDomainAction(actionKind);
      }
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-mission-preview]").forEach((button) => {
    button.addEventListener("click", () => {
      const actionId = button.dataset.missionPreview;
      if (actionId) void previewMissionAction(actionId);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-mission-explain]").forEach((button) => {
    button.addEventListener("click", () => {
      const actionId = button.dataset.missionExplain;
      if (actionId) void explainMissionAction(actionId);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-mission-run]").forEach((button) => {
    button.addEventListener("click", () => {
      const actionId = button.dataset.missionRun;
      if (actionId) void runMissionAction(actionId);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-mission-toggle-details]").forEach((button) => {
    button.addEventListener("click", (event) => {
      event.stopPropagation();
      const actionId = button.dataset.missionToggleDetails;
      if (actionId) void toggleMissionActionDetails(actionId);
    });
  });

  document.querySelectorAll<HTMLElement>("[data-mission-card]").forEach((card) => {
    card.addEventListener("click", (event) => {
      const target = event.target as HTMLElement | null;
      if (target?.closest("button, a, details, summary")) return;
      const actionId = card.dataset.missionCard;
      if (actionId) void toggleMissionActionDetails(actionId);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-mission-later]").forEach((button) => {
    button.addEventListener("click", () => {
      const actionId = button.closest<HTMLElement>(".storage-action-row")?.querySelector<HTMLButtonElement>("[data-mission-run]")?.dataset.missionRun;
      if (actionId) missionActionStatuses[actionId] = "Deferred";
      deferMission();
      recordMissionTelemetry({
        missionId: currentPulseMission?.id || "unknown",
        actionId,
        event: "Mission deferred",
        timestamp: new Date().toISOString(),
      });
      missionActionDetail = null;
      careMessage = "Mission deferred. It will reappear in 30 minutes.";
      renderToday(pulse);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-mission-cancel]").forEach((button) => {
    button.addEventListener("click", () => {
      const actionId = button.dataset.missionCancel;
      recordMissionTelemetry({
        missionId: currentPulseMission?.id || "unknown",
        actionId,
        event: "Mission cancelled",
        timestamp: new Date().toISOString(),
      });
      missionActionDetail = null;
      renderToday(pulse);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-mission-retry]").forEach((button) => {
    button.addEventListener("click", () => {
      void loadPulseMissions(true);
    });
  });
}

async function ensurePulseMission(): Promise<void> {
  if (currentView !== "today") return;
  if (isMissionDeferred()) return;
  if (currentPulseMission || missionLoading || missionError) return;
  await loadPulseMissions();
}

async function loadPulseMissions(force = false): Promise<void> {
  if (missionLoading) return;
  if (!force && currentPulseMission) return;
  if (isMissionDeferred()) return;

  missionLoading = true;
  missionError = "";
  if (currentPulse) renderToday(currentPulse);

  try {
    const snapshot = await invoke<MissionRegistrySnapshot>("get_pulse_missions");
    currentPulseMission = snapshot.topMission;
    missionActionDetail = null;
  } catch (error) {
    currentPulseMission = null;
    missionError = error instanceof Error ? error.message : String(error);
  } finally {
    missionLoading = false;
    if (currentPulse && currentView === "today") renderToday(currentPulse);
  }
}

async function previewMissionAction(actionId: string): Promise<void> {
  careMessage = "Checking what would be cleaned...";
  if (currentPulse) renderToday(currentPulse);

  try {
    const preview = await invoke<MissionActionPreview>("preview_mission_action", { actionId });
    careMessage = "";
    missionActionDetail = { kind: "preview", preview };
  } catch (error) {
    careMessage = error instanceof Error ? error.message : String(error);
  }
  if (currentPulse) renderToday(currentPulse);
}

async function toggleMissionActionDetails(actionId: string): Promise<void> {
  if (isMissionActionExpanded(actionId) && missionActionDetail?.kind === "preview") {
    missionActionDetail = null;
    if (currentPulse) renderToday(currentPulse);
    return;
  }
  await previewMissionAction(actionId);
}

async function explainMissionAction(actionId: string): Promise<void> {
  careMessage = "Preparing explanation...";
  if (currentPulse) renderToday(currentPulse);

  try {
    const explanation = await invoke<MissionActionExplanation>("explain_mission_action", { actionId });
    careMessage = "";
    missionActionDetail = { kind: "explain", explanation };
  } catch (error) {
    careMessage = error instanceof Error ? error.message : String(error);
  }
  if (currentPulse) renderToday(currentPulse);
}

async function runMissionAction(actionId: string): Promise<void> {
  const action = currentPulseMission?.actions.find((item) => item.id === actionId);
  const title = action ? missionActionTitle(action) : "this storage action";

  const startedAt = Date.now();
  recordMissionTelemetry({
    missionId: currentPulseMission?.id || "unknown",
    actionId,
    event: "Mission started",
    timestamp: new Date().toISOString(),
  });
  missionActionStatuses[actionId] = "Running";
  careMessage = `${title} is cleaning now.`;
  missionActionDetail = null;
  if (currentPulse) renderToday(currentPulse);

  try {
    const result = await invoke<MissionActionRunResult>("run_mission_action", { actionId });
    missionActionStatuses[actionId] = result.success ? "Completed" : "Skipped";
    missionActionDetail = { kind: "result", result };
    recordMissionTelemetry({
      missionId: currentPulseMission?.id || "unknown",
      actionId,
      event: "Mission completed",
      timestamp: new Date().toISOString(),
      durationMs: Date.now() - startedAt,
      verification: result.verification,
    });
    careMessage = result.success
      ? `${result.title} complete. Recovered ${result.recovered}.`
      : `${result.title} partly completed. Review the details below.`;
    const snapshot = await invoke<MissionRegistrySnapshot>("get_pulse_missions");
    currentPulseMission = snapshot.topMission;
    await loadToday({ keepExisting: true, quiet: true });
  } catch (error) {
    missionActionStatuses[actionId] = "Ready";
    careMessage = error instanceof Error ? error.message : String(error);
    if (currentPulse) renderToday(currentPulse);
  }
}

async function performApplicationAction(application: ApplicationImpact): Promise<void> {
  if (application.actionKind === "none") return;
  const target = application.actionTarget || application.name;
  const confirmed = window.confirm(
    `System Pulse will ask macOS to ${application.actionLabel.toLowerCase()} for ${target}. Continue?`,
  );
  if (!confirmed) return;

  selectedApplicationId = null;
  careMessage = `Working on ${target}...`;
  if (currentPulse) renderToday(currentPulse, true);

  try {
    const message = await invoke<string>("perform_care_action", {
      actionKind: application.actionKind,
      target,
    });
    careMessage = message;
    await loadToday({ keepExisting: true });
  } catch (error) {
    careMessage = error instanceof Error ? error.message : String(error);
    if (currentPulse) renderToday(currentPulse);
  }
}

async function performBrowserAction(actionKind: string, target: string): Promise<void> {
  const label = actionKind === "quitApp" ? "quit" : "restart";
  const confirmed = window.confirm(`System Pulse will ask macOS to ${label} ${target}. Continue?`);
  if (!confirmed) return;

  selectedApplicationId = null;
  careMessage = `Working on ${target}...`;
  if (currentPulse) renderToday(currentPulse, true);

  try {
    const message = await invoke<string>("perform_care_action", {
      actionKind,
      target,
    });
    careMessage = message;
    await loadToday({ keepExisting: true });
  } catch (error) {
    careMessage = error instanceof Error ? error.message : String(error);
    if (currentPulse) renderToday(currentPulse);
  }
}

async function performDomainAction(actionKind: string): Promise<void> {
  if (actionKind !== "openStorageSettings") return;
  careMessage = "Opening Storage Settings...";
  if (currentPulse) renderToday(currentPulse, true);

  try {
    const message = await invoke<string>("perform_care_action", {
      actionKind,
      target: "",
    });
    careMessage = message;
    if (currentPulse) renderToday(currentPulse);
  } catch (error) {
    careMessage = error instanceof Error ? error.message : String(error);
    if (currentPulse) renderToday(currentPulse);
  }
}

function renderLoading(): void {
  appRoot.innerHTML = `
    <div class="shell loading-shell">
      <section class="card loading-card">
        <span class="heart large-heart" aria-hidden="true">&hearts;</span>
        <h1>Checking whether you can keep working.</h1>
        <p>System Pulse is checking quietly. Nothing changes on your Mac.</p>
      </section>
    </div>
  `;
}

function renderError(message: string): void {
  appRoot.innerHTML = `
    <div class="shell">
      <section class="card loading-card">
        <span class="heart large-heart" aria-hidden="true">&hearts;</span>
        <h1>System Pulse could not refresh yet.</h1>
        <p>${escapeHtml(message)}</p>
        <p class="quiet-copy">Nothing has been changed on your Mac. System Pulse only reads local health signals.</p>
        <button id="retry-button" type="button">Try again</button>
      </section>
    </div>
  `;
  document.querySelector<HTMLButtonElement>("#retry-button")?.addEventListener("click", () => {
    void loadToday();
  });
}

async function updateTray(pulse: TodayPulse): Promise<void> {
  try {
    await invoke("update_tray_score", {
      systemScore: pulse.systemScore,
      healthState: pulse.healthState,
      flowRemainingLabel: focusTimeLabel(pulse),
    });
  } catch {
    // Tray title updates are best-effort; the Today screen remains authoritative.
  }
}

async function loadToday(options: { keepExisting?: boolean; quiet?: boolean } = {}): Promise<void> {
  if (isRefreshing) return;
  isRefreshing = true;

  if (options.keepExisting && currentPulse && !options.quiet) {
    renderCurrentView(currentPulse, true);
  } else if (!options.keepExisting && !options.quiet) {
    renderLoading();
  }

  try {
    const pulse = await invoke<TodayPulse>("get_today_pulse");
    currentPulse = pulse;
    renderCurrentView(pulse);
    await updateTray(pulse);
  } catch (error) {
    if (options.quiet && currentPulse) {
      careMessage = "The last live check missed. System Pulse will try again when reopened or refreshed.";
      renderCurrentView(currentPulse);
    } else {
      renderError(error instanceof Error ? error.message : String(error));
    }
  } finally {
    isRefreshing = false;
  }
}

function startAutoRefresh(): void {
  if (autoRefreshTimer !== undefined) return;
  autoRefreshTimer = window.setInterval(() => {
    void loadToday({ keepExisting: true, quiet: true });
  }, VISIBLE_REFRESH_MS);
}

function stopAutoRefresh(): void {
  if (autoRefreshTimer === undefined) return;
  window.clearInterval(autoRefreshTimer);
  autoRefreshTimer = undefined;
}

function refreshVisibleView(): void {
  startAutoRefresh();
  void loadToday({ keepExisting: Boolean(currentPulse), quiet: Boolean(currentPulse) });
}

void listen("system-pulse-refresh", () => {
  startAutoRefresh();
  void loadToday({ keepExisting: true });
});
void listen("system-pulse-show-quick-checkin", () => {
  currentView = "quick";
  if (currentPulse) {
    renderQuickCheckin(currentPulse);
    startAutoRefresh();
    void loadToday({ keepExisting: true, quiet: true });
    return;
  }
  refreshVisibleView();
});
void listen("system-pulse-show-today", () => {
  currentView = "today";
  if (currentPulse) {
    renderToday(currentPulse);
    startAutoRefresh();
    void loadToday({ keepExisting: true, quiet: true });
    return;
  }
  refreshVisibleView();
});
window.addEventListener("focus", () => {
  if (!document.hidden) {
    refreshVisibleView();
  }
});
window.addEventListener("blur", stopAutoRefresh);
document.addEventListener("visibilitychange", () => {
  if (document.hidden) {
    stopAutoRefresh();
    return;
  }

  refreshVisibleView();
});
window.addEventListener("beforeunload", stopAutoRefresh);
