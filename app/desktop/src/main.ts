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

type StorageCareAction = {
  id: string;
  title: string;
  description: string;
  estimatedBenefit: string;
  estimatedBenefitBytes: number;
  interruption: string;
  risk: string;
  confidence: number;
  previewItemCount: number;
};

type StorageRecoveryPlan = {
  id: string;
  title: string;
  explanation: string;
  estimatedBenefit: string;
  estimatedBenefitBytes: number;
  estimatedTime: string;
  interruption: string;
  confidence: number;
  actions: StorageCareAction[];
};

type StoragePreviewFile = {
  name: string;
  size: string;
  path: string;
};

type StorageCareActionPreview = {
  actionId: string;
  title: string;
  estimatedRecovery: string;
  estimatedRecoveryBytes: number;
  files: StoragePreviewFile[];
  omittedCount: number;
};

type StorageCareActionExplanation = {
  actionId: string;
  title: string;
  reason: string;
  expectedBenefit: string;
  risk: string;
  interruption: string;
};

type StorageCareActionRunResult = {
  actionId: string;
  title: string;
  success: boolean;
  recovered: string;
  recoveredBytes: number;
  currentFreeSpace: string;
  currentFreeSpaceBytes: number;
  storageHealth: string;
  verified: boolean;
  errors: string[];
};

type StorageActionDetail =
  | { kind: "preview"; preview: StorageCareActionPreview }
  | { kind: "explain"; explanation: StorageCareActionExplanation }
  | { kind: "result"; result: StorageCareActionRunResult };

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
let currentStorageRecoveryPlan: StorageRecoveryPlan | null = null;
let storageRecoveryLoading = false;
let storageRecoveryError = "";
let storageActionDetail: StorageActionDetail | null = null;

type CareMuteState = Record<
  string,
  {
    remindUntil?: number;
    ignoredOn?: string;
  }
>;

const CARE_STATE_KEY = "system-pulse-care-state-v1";
const STORAGE_RECOVERY_LATER_KEY = "system-pulse-storage-recovery-later-until";

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

function isStorageRecoveryDeferred(): boolean {
  const value = localStorage.getItem(STORAGE_RECOVERY_LATER_KEY);
  if (!value) return false;
  const laterUntil = Number.parseInt(value, 10);
  return Number.isFinite(laterUntil) && laterUntil > Date.now();
}

function deferStorageRecovery(): void {
  localStorage.setItem(STORAGE_RECOVERY_LATER_KEY, String(Date.now() + 30 * 60 * 1000));
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

function storageRecoveryMetric(label: string, value: string): string {
  return `
    <span>
      <small>${escapeHtml(label)}</small>
      <strong>${escapeHtml(value)}</strong>
    </span>
  `;
}

function storageActionIdLabel(actionId: string): string {
  if (actionId === "empty-trash") return "Trash";
  if (actionId === "delete-downloaded-installers") return "Downloaded installers";
  if (actionId === "clear-obsolete-caches") return "Application caches";
  return "Storage";
}

function storageActionDetailHtml(detail: StorageActionDetail | null): string {
  if (!detail) return "";

  if (detail.kind === "preview") {
    const preview = detail.preview;
    const files = preview.files.length
      ? preview.files
          .map(
            (file) => `
              <li>
                <span>
                  <strong>${escapeHtml(file.name)}</strong>
                  <small>${escapeHtml(file.path)}</small>
                </span>
                <em>${escapeHtml(file.size)}</em>
              </li>
            `,
          )
          .join("")
      : `<li><span><strong>No files found for this action.</strong></span></li>`;
    const omitted = preview.omittedCount
      ? `<p class="storage-preview-note">And ${preview.omittedCount} more item${preview.omittedCount === 1 ? "" : "s"}.</p>`
      : "";

    return `
      <section class="storage-action-detail" aria-label="${escapeHtml(preview.title)} preview">
        <p class="care-task-label">Preview</p>
        <h3>${escapeHtml(preview.title)}</h3>
        <p>Estimated recovery ${escapeHtml(preview.estimatedRecovery)}.</p>
        <ul class="storage-file-list">${files}</ul>
        ${omitted}
        <div class="storage-detail-actions">
          <button class="recommended-primary-button" type="button" data-storage-run="${escapeHtml(preview.actionId)}">Run</button>
          <button class="recommended-secondary-button" type="button" data-storage-cancel>Cancel</button>
        </div>
      </section>
    `;
  }

  if (detail.kind === "explain") {
    const explanation = detail.explanation;
    return `
      <section class="storage-action-detail" aria-label="${escapeHtml(explanation.title)} explanation">
        <p class="care-task-label">Explain</p>
        <h3>${escapeHtml(explanation.title)}</h3>
        <dl class="storage-explain-list">
          <div><dt>Reason</dt><dd>${escapeHtml(explanation.reason)}</dd></div>
          <div><dt>Expected benefit</dt><dd>${escapeHtml(explanation.expectedBenefit)}</dd></div>
          <div><dt>Risk</dt><dd>${escapeHtml(explanation.risk)}</dd></div>
          <div><dt>Interruption</dt><dd>${escapeHtml(explanation.interruption)}</dd></div>
        </dl>
        <div class="storage-detail-actions">
          <button class="recommended-primary-button" type="button" data-storage-preview="${escapeHtml(explanation.actionId)}">Preview</button>
          <button class="recommended-secondary-button" type="button" data-storage-cancel>Close</button>
        </div>
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
      <p class="care-task-label">${result.success ? "Complete" : "Needs review"}</p>
      <h3>${result.success ? "Complete" : "Partly complete"}</h3>
      <div class="storage-result-grid">
        ${storageRecoveryMetric("Recovered", result.recovered)}
        ${storageRecoveryMetric("Current free space", result.currentFreeSpace)}
        ${storageRecoveryMetric("Storage health", result.storageHealth)}
      </div>
      <p>${result.verified ? "System Pulse measured your free space again after the action." : "System Pulse ran the action, but macOS has not reported a free-space increase yet."}</p>
      ${errors}
      <div class="storage-detail-actions">
        <button class="recommended-secondary-button" type="button" data-storage-cancel>Close</button>
      </div>
    </section>
  `;
}

function storageRecoveryPlanHtml(plan: StorageRecoveryPlan): string {
  if (!plan.actions.length) {
    return `
      <section class="summary-section care-panel calm-panel">
        <h2>${storageActionDetail ? "Storage recovery complete" : "No storage care needed"}</h2>
        ${careMessageHtml()}
        <p class="summary-answer">${escapeHtml(plan.explanation)}</p>
        ${storageActionDetailHtml(storageActionDetail)}
      </section>
    `;
  }

  const actions = plan.actions
    .map(
      (action) => `
        <div class="storage-action-row">
          <div>
            <p class="care-task-label">${escapeHtml(storageActionIdLabel(action.id))}</p>
            <strong>${escapeHtml(action.title)}</strong>
            <small>${escapeHtml(action.previewItemCount.toString())} item${action.previewItemCount === 1 ? "" : "s"} · ${escapeHtml(action.estimatedBenefit)} recoverable</small>
            <p class="care-task-detail">${escapeHtml(action.description)}</p>
          </div>
          <div class="storage-action-buttons">
            <button class="recommended-secondary-button" type="button" data-storage-preview="${escapeHtml(action.id)}">Preview</button>
            <button class="recommended-primary-button" type="button" data-storage-run="${escapeHtml(action.id)}">Run</button>
            <button class="recommended-secondary-button" type="button" data-storage-later>Later</button>
            <button class="recommended-secondary-button" type="button" data-storage-explain="${escapeHtml(action.id)}">Explain</button>
          </div>
        </div>
      `,
    )
    .join("");

  return `
    <section class="summary-section care-panel attention-panel storage-recovery-panel">
      <p class="panel-kicker recommended-kicker"><span aria-hidden="true">&#9733;</span> Recovery plan</p>
      <h2>${escapeHtml(plan.title)}</h2>
      <p class="care-plan-intro">${escapeHtml(plan.explanation)}</p>
      ${careMessageHtml()}
      <div class="storage-recovery-metrics">
        ${storageRecoveryMetric("Estimated time", plan.estimatedTime)}
        ${storageRecoveryMetric("Expected improvement", plan.estimatedBenefit)}
        ${storageRecoveryMetric("Interruption", plan.interruption)}
      </div>
      <div class="storage-action-list" aria-label="Largest recoverable items">
        ${actions}
      </div>
      ${storageActionDetailHtml(storageActionDetail)}
    </section>
  `;
}

function storageRecoveryLoadingHtml(): string {
  return `
    <section class="summary-section care-panel attention-panel storage-recovery-panel">
      <p class="panel-kicker recommended-kicker"><span aria-hidden="true">&#9733;</span> Recovery plan</p>
      <h2>Checking safe storage recovery.</h2>
      ${careMessageHtml()}
      <p class="care-plan-intro">System Pulse is checking Trash, old installers, and conservative app caches. Nothing changes on your Mac.</p>
    </section>
  `;
}

function storageRecoveryErrorHtml(): string {
  return `
    <section class="summary-section care-panel attention-panel storage-recovery-panel">
      <p class="panel-kicker recommended-kicker"><span aria-hidden="true">&#9733;</span> Recovery plan</p>
      <h2>Storage recovery could not be checked yet.</h2>
      ${careMessageHtml()}
      <p class="care-plan-intro">${escapeHtml(storageRecoveryError)}</p>
      <button class="recommended-secondary-button" type="button" data-storage-retry>Try again</button>
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
    metric: `${application.memoryDisplay} RAM · ${application.cpuDisplay} CPU`,
  }));
  return items.length ? items : [{ label: "No non-browser application is standing out." }];
}

function applicationUsageList(pulse: TodayPulse): string {
  const items = applicationUsageItems(pulse)
    .map(
      (item) => `
        <li>
          <span>${escapeHtml(item.label)}</span>
          ${item.metric ? `<strong>${escapeHtml(item.metric)}</strong>` : ""}
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
        <p class="care-task-benefit">Expected interruption about 20 seconds · expected benefit +35 minutes</p>
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
  if (!isStorageRecoveryDeferred()) {
    if (currentStorageRecoveryPlan && (currentStorageRecoveryPlan.actions.length || storageActionDetail)) {
      return storageRecoveryPlanHtml(currentStorageRecoveryPlan);
    }
    if (storageRecoveryLoading) {
      return storageRecoveryLoadingHtml();
    }
    if (storageRecoveryError) {
      return storageRecoveryErrorHtml();
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
  void ensureStorageRecoveryPlan();
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

  document.querySelectorAll<HTMLButtonElement>("[data-storage-preview]").forEach((button) => {
    button.addEventListener("click", () => {
      const actionId = button.dataset.storagePreview;
      if (actionId) void previewStorageAction(actionId);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-storage-explain]").forEach((button) => {
    button.addEventListener("click", () => {
      const actionId = button.dataset.storageExplain;
      if (actionId) void explainStorageAction(actionId);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-storage-run]").forEach((button) => {
    button.addEventListener("click", () => {
      const actionId = button.dataset.storageRun;
      if (actionId) void runStorageAction(actionId);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-storage-later]").forEach((button) => {
    button.addEventListener("click", () => {
      deferStorageRecovery();
      storageActionDetail = null;
      careMessage = "Storage recovery will reappear in 30 minutes.";
      renderToday(pulse);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-storage-cancel]").forEach((button) => {
    button.addEventListener("click", () => {
      storageActionDetail = null;
      renderToday(pulse);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-storage-retry]").forEach((button) => {
    button.addEventListener("click", () => {
      void loadStorageRecoveryPlan(true);
    });
  });
}

async function ensureStorageRecoveryPlan(): Promise<void> {
  if (currentView !== "today") return;
  if (isStorageRecoveryDeferred()) return;
  if (currentStorageRecoveryPlan || storageRecoveryLoading || storageRecoveryError) return;
  await loadStorageRecoveryPlan();
}

async function loadStorageRecoveryPlan(force = false): Promise<void> {
  if (storageRecoveryLoading) return;
  if (!force && currentStorageRecoveryPlan) return;
  if (isStorageRecoveryDeferred()) return;

  storageRecoveryLoading = true;
  storageRecoveryError = "";
  if (currentPulse) renderToday(currentPulse);

  try {
    currentStorageRecoveryPlan = await invoke<StorageRecoveryPlan>("get_storage_recovery_plan");
    storageActionDetail = null;
  } catch (error) {
    currentStorageRecoveryPlan = null;
    storageRecoveryError = error instanceof Error ? error.message : String(error);
  } finally {
    storageRecoveryLoading = false;
    if (currentPulse && currentView === "today") renderToday(currentPulse);
  }
}

async function previewStorageAction(actionId: string): Promise<void> {
  careMessage = "Preparing preview...";
  if (currentPulse) renderToday(currentPulse);

  try {
    const preview = await invoke<StorageCareActionPreview>("preview_storage_care_action", { actionId });
    careMessage = "";
    storageActionDetail = { kind: "preview", preview };
  } catch (error) {
    careMessage = error instanceof Error ? error.message : String(error);
  }
  if (currentPulse) renderToday(currentPulse);
}

async function explainStorageAction(actionId: string): Promise<void> {
  careMessage = "Preparing explanation...";
  if (currentPulse) renderToday(currentPulse);

  try {
    const explanation = await invoke<StorageCareActionExplanation>("explain_storage_care_action", { actionId });
    careMessage = "";
    storageActionDetail = { kind: "explain", explanation };
  } catch (error) {
    careMessage = error instanceof Error ? error.message : String(error);
  }
  if (currentPulse) renderToday(currentPulse);
}

async function runStorageAction(actionId: string): Promise<void> {
  const action = currentStorageRecoveryPlan?.actions.find((item) => item.id === actionId);
  const title = action?.title || "this storage action";
  const benefit = action?.estimatedBenefit || "storage space";
  const confirmed = window.confirm(
    `System Pulse will run ${title} and try to recover ${benefit}. Continue?`,
  );
  if (!confirmed) return;

  careMessage = `Running ${title}...`;
  storageActionDetail = null;
  if (currentPulse) renderToday(currentPulse);

  try {
    const result = await invoke<StorageCareActionRunResult>("run_storage_care_action", { actionId });
    storageActionDetail = { kind: "result", result };
    careMessage = result.success
      ? `${result.title} complete. Recovered ${result.recovered}.`
      : `${result.title} partly completed. Review the details below.`;
    currentStorageRecoveryPlan = await invoke<StorageRecoveryPlan>("get_storage_recovery_plan");
    await loadToday({ keepExisting: true, quiet: true });
  } catch (error) {
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
