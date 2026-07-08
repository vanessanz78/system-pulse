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

type CareMuteState = Record<
  string,
  {
    remindUntil?: number;
    ignoredOn?: string;
  }
>;

const CARE_STATE_KEY = "system-pulse-care-state-v1";

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

function shouldHideOpportunity(application: ApplicationImpact): boolean {
  const state = readCareState()[actionId(application)];
  if (!state) return false;
  if (state.ignoredOn === todayKey()) return true;
  return Boolean(state.remindUntil && state.remindUntil > Date.now());
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
  if (domainNeedsCare(pulse.storageHealth)) {
    tasks.push(domainCareTask("Storage", "storage", pulse.storageHealth, "Open Settings", "openStorageSettings"));
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
            <strong>${escapeHtml(pulse.flowRemainingLabel)}</strong>
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
          <strong>${escapeHtml(pulse.flowRemainingLabel)}</strong>
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
      flowRemainingLabel: pulse.flowRemainingLabel,
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
