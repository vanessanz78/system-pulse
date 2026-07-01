import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./styles.css";

type HealthState = "healthy" | "attention" | "critical";
type ViewMode = "quick" | "today";

type DomainHealth = {
  label: string;
  headline: string;
  detail: string;
  value: string;
};

type ApplicationImpact = {
  name: string;
  memoryDisplay: string;
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
  applicationHealth: DomainHealth;
  browserHealth?: DomainHealth;
  topApplications: ApplicationImpact[];
};

const app = document.querySelector<HTMLElement>("#app");
const USER_NAME = "Vanessa";

if (!app) {
  throw new Error("System Pulse root element is missing.");
}

const appRoot = app;
const APP_VERSION = "0.1.5";
const AUTO_REFRESH_MS = 60_000;
let currentPulse: TodayPulse | null = null;
let isRefreshing = false;
let currentView: ViewMode = "quick";
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

function canKeepWorking(state: HealthState): string {
  if (state === "healthy") return "Yes. Keep working.";
  if (state === "attention") return "Yes, finish this task first.";
  return "Pause for care soon.";
}

function comfortLine(state: HealthState): string {
  if (state === "healthy") return "You're still working comfortably.";
  if (state === "attention") {
    return "You have enough room to keep going, but care may help at your next break.";
  }
  return "Your Mac is likely to get in the way unless you take a care moment.";
}

function immediateAction(pulse: TodayPulse): string {
  if (pulse.healthState === "healthy") return "No action needed right now.";
  return pulse.primaryRecommendation;
}

function whyHeadline(state: HealthState): string {
  if (state === "healthy") return "Nothing is asking for care.";
  if (state === "attention") return "One thing may start getting in your way.";
  return "Care is likely to help soon.";
}

function signalMark(domain: DomainHealth): string {
  const label = domain.label.toLowerCase();
  if (label.includes("recommended") || label.includes("care")) {
    return "!";
  }
  return "✓";
}

function signalClass(domain: DomainHealth): string {
  const label = domain.label.toLowerCase();
  if (label.includes("recommended") || label.includes("care")) {
    return "needs-care";
  }
  return "ok";
}

function formatCollectedAt(value: string): string {
  const seconds = Number(value.replace(/^Unix\s+/, ""));
  if (Number.isFinite(seconds) && seconds > 0) {
    return new Date(seconds * 1000).toLocaleString([], {
      dateStyle: "medium",
      timeStyle: "short",
    });
  }

  return value;
}

function quickSignal(label: string, domain: DomainHealth): string {
  return `
    <li class="${signalClass(domain)}">
      <strong>${escapeHtml(label)}</strong>
      <span aria-hidden="true">${signalMark(domain)}</span>
    </li>
  `;
}

function careMessageHtml(): string {
  if (!careMessage) return "";
  return `<p class="care-message" role="status">${escapeHtml(careMessage)}</p>`;
}

function liveStatusLine(pulse: TodayPulse, refreshing = false): string {
  const checkedAt = formatCollectedAt(pulse.collectedAt);
  if (refreshing) return `Refreshing local data. Last checked ${checkedAt}.`;
  return `Live local check every minute. Last checked ${checkedAt}.`;
}

function visibleApplicationOpportunities(pulse: TodayPulse): ApplicationImpact[] {
  return pulse.topApplications
    .filter((application) => application.showOpportunity)
    .filter((application) => application.actionKind !== "none")
    .filter((application) => !shouldHideOpportunity(application))
    .slice(0, 3);
}

function protectedWorkNotes(pulse: TodayPulse): ApplicationImpact[] {
  return pulse.topApplications
    .filter((application) => application.protectedWork)
    .slice(0, 2);
}

function opportunityButtons(application: ApplicationImpact): string {
  return `
    <div class="care-actions">
      <button
        class="primary-care-button"
        type="button"
        data-care-action="${escapeHtml(actionId(application))}"
      >
        ${escapeHtml(application.actionLabel || "Do this now")}
      </button>
      <button
        class="secondary-care-button"
        type="button"
        data-care-remind="${escapeHtml(actionId(application))}"
      >
        Remind me in 30 minutes
      </button>
      <button
        class="quiet-care-button"
        type="button"
        data-care-ignore="${escapeHtml(actionId(application))}"
      >
        Ignore today
      </button>
    </div>
  `;
}

function opportunityCard(application: ApplicationImpact): string {
  return `
    <article class="opportunity-card" data-application="${escapeHtml(actionId(application))}">
      <div class="opportunity-main">
        <p class="eyebrow">Recommended care</p>
        <h3>${escapeHtml(application.name)}</h3>
        <strong>${escapeHtml(application.careLabel)}</strong>
        <p>${escapeHtml(application.careDetail)}</p>
      </div>
      <div class="opportunity-decision">
        <div class="time-gain compact">
          <span>Estimated benefit</span>
          <strong>${escapeHtml(application.careEstimatedImprovement)}</strong>
        </div>
        ${opportunityButtons(application)}
      </div>
    </article>
  `;
}

function protectedWorkCard(application: ApplicationImpact): string {
  return `
    <article class="opportunity-card protected-work">
      <div class="opportunity-main">
        <p class="eyebrow">Protected work</p>
        <h3>${escapeHtml(application.name)}</h3>
        <strong>${escapeHtml(application.careLabel)}</strong>
        <p>${escapeHtml(application.careDetail)}</p>
      </div>
      <div class="opportunity-decision quiet-decision">
        <span>No button shown</span>
        <strong>Active work comes first.</strong>
      </div>
    </article>
  `;
}

function storageOpportunity(pulse: TodayPulse): string {
  if (pulse.storageHealth.label === "OK") return "";
  return `
    <article class="opportunity-card">
      <div class="opportunity-main">
        <p class="eyebrow">Storage</p>
        <h3>Review storage when you have a quiet moment.</h3>
        <strong>${escapeHtml(pulse.storageHealth.headline)}</strong>
        <p>${escapeHtml(pulse.storageHealth.detail)}</p>
      </div>
      <div class="opportunity-decision">
        <div class="time-gain compact">
          <span>Estimated benefit</span>
          <strong>+15 minutes</strong>
        </div>
        <div class="care-actions">
          <button
            class="primary-care-button"
            type="button"
            data-domain-action="openStorageSettings"
          >
            Open Storage Settings
          </button>
        </div>
      </div>
    </article>
  `;
}

function todaysOpportunities(pulse: TodayPulse): string {
  const applications = visibleApplicationOpportunities(pulse)
    .map(opportunityCard)
    .join("");
  const protectedNotes = protectedWorkNotes(pulse)
    .map(protectedWorkCard)
    .join("");
  const storage = storageOpportunity(pulse);
  const hasOpportunities = Boolean(applications || storage);

  return `
    <section class="opportunities-card">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Today's Opportunities</p>
          <h2>${hasOpportunities ? "Lowest-disruption care." : "No useful action right now."}</h2>
        </div>
        <span>${hasOpportunities ? "You stay in control" : "Keep working"}</span>
      </div>
      ${careMessageHtml()}
      ${
        hasOpportunities
          ? `<div class="opportunity-list">${applications}${storage}${protectedNotes}</div>`
          : `<p class="reassurance-copy">System Pulse is staying quiet because nothing would meaningfully protect your momentum right now.</p>${protectedNotes}`
      }
    </section>
  `;
}

function localDetails(pulse: TodayPulse): string {
  const details = [
    ["Applications", pulse.applicationHealth],
    ["Storage", pulse.storageHealth],
    ["Memory", pulse.memoryHealth],
    ["Browser", pulse.browserHealth],
  ]
    .filter(([, domain]) => Boolean(domain))
    .map(([label, domain]) => {
      const typedDomain = domain as DomainHealth;
      return `
        <li>
          <strong>${escapeHtml(label as string)}</strong>
          <span>${escapeHtml(typedDomain.headline)}</span>
          <em>${escapeHtml(typedDomain.value)}</em>
        </li>
      `;
    })
    .join("");

  return `
    <details class="card detail-card">
      <summary>Show local signals</summary>
      <ul>${details}</ul>
    </details>
  `;
}

function decisionLabel(domain: DomainHealth): string {
  if (domain.label === "OK" || domain.label.toLowerCase() === "healthy") {
    return "No action needed today";
  }
  if (domain.label === "Care") return "Care soon";
  return "Best reviewed at your next break";
}

function statusClass(label: string): string {
  const lower = label.toLowerCase();
  if (lower.includes("recommended") || lower.includes("care")) return "needs-care";
  if (lower.includes("break") || lower.includes("protect")) return "attention";
  return "ok";
}

function statusCard(label: string, status: string, headline: string, detail: string, value = ""): string {
  return `
    <section class="card status-card ${statusClass(status)}">
      <div class="card-heading">
        <span>${escapeHtml(label)}</span>
        <b>${escapeHtml(status)}</b>
      </div>
      <h3>${escapeHtml(headline)}</h3>
      <p>${escapeHtml(detail)}</p>
      ${value ? `<em>${escapeHtml(value)}</em>` : ""}
    </section>
  `;
}

function todayStatusGrid(pulse: TodayPulse, refreshing = false): string {
  const browser = pulse.browserHealth ?? pulse.applicationHealth;
  const nextStepStatus =
    pulse.healthState === "healthy" ? "No action needed today" : "Recommended today";

  return `
    <section class="today-status-grid" aria-label="Today at a glance">
      ${statusCard(
        "Flow",
        "Live",
        `${pulse.flowRemainingLabel} uninterrupted work time`,
        liveStatusLine(pulse, refreshing),
        `v${APP_VERSION}`,
      )}
      ${statusCard(
        "Next best step",
        nextStepStatus,
        immediateAction(pulse),
        pulse.primaryExplanation,
        pulse.estimatedAdditionalWorkLabel,
      )}
      ${statusCard(
        "Browser",
        decisionLabel(browser),
        browser.headline,
        browser.detail,
        browser.value,
      )}
      ${statusCard(
        "Applications",
        decisionLabel(pulse.applicationHealth),
        pulse.applicationHealth.headline,
        pulse.applicationHealth.detail,
        pulse.applicationHealth.value,
      )}
      ${statusCard(
        "Memory",
        decisionLabel(pulse.memoryHealth),
        pulse.memoryHealth.headline,
        pulse.memoryHealth.detail,
        pulse.memoryHealth.value,
      )}
      ${statusCard(
        "Storage",
        decisionLabel(pulse.storageHealth),
        pulse.storageHealth.headline,
        pulse.storageHealth.detail,
        pulse.storageHealth.value,
      )}
    </section>
  `;
}

function renderCurrentView(pulse: TodayPulse, refreshing = false): void {
  if (currentView === "today") {
    renderToday(pulse, refreshing);
    return;
  }

  renderQuickCheckin(pulse, refreshing);
}

function renderQuickCheckin(pulse: TodayPulse, refreshing = false): void {
  const refreshLabel = refreshing ? "Refreshing..." : "Refresh";
  const glanceRows = [
    quickSignal("Applications", pulse.applicationHealth),
    quickSignal("Storage", pulse.storageHealth),
    quickSignal("Memory", pulse.memoryHealth),
  ].join("");

  appRoot.innerHTML = `
    <main class="quick-shell" data-state="${pulse.healthState}">
      <section class="quick-card">
        <div class="quick-tools">
          <span class="pulse-pill">
            <span class="heart mini-heart" aria-hidden="true">&hearts;</span>
            <strong>${pulse.systemScore}</strong>
          </span>
          <button id="quick-refresh-button" class="icon-button" type="button" ${refreshing ? "disabled" : ""}>${refreshLabel}</button>
        </div>

        <div class="quick-answer">
          <p class="eyebrow">Can I keep working?</p>
          <h1>${canKeepWorking(pulse.healthState)}</h1>
          <p>${comfortLine(pulse.healthState)}</p>
        </div>

        <div class="quick-time">
          <span>Estimated uninterrupted work time</span>
          <strong>${escapeHtml(pulse.flowRemainingLabel)}</strong>
        </div>

        <div class="quick-now">
          <span>Do I need to do anything?</span>
          <strong>${escapeHtml(immediateAction(pulse))}</strong>
        </div>

        <div class="quick-glance">
          <ul>${glanceRows}</ul>
        </div>

        <button id="open-today-button" class="open-today-button" type="button">
          Open Today
          <span aria-hidden="true">&rsaquo;</span>
        </button>

        <p class="quick-footnote">${escapeHtml(liveStatusLine(pulse, refreshing))}</p>
        <p class="quick-footnote">Local check only. Nothing changes on your Mac. v${APP_VERSION}</p>
      </section>
    </main>
  `;

  document.querySelector<HTMLButtonElement>("#quick-refresh-button")?.addEventListener("click", () => {
    void loadToday({ keepExisting: true });
  });
  document.querySelector<HTMLButtonElement>("#open-today-button")?.addEventListener("click", () => {
    currentView = "today";
    void invoke("open_today_window");
    renderToday(pulse);
  });
}

function renderToday(pulse: TodayPulse, refreshing = false): void {
  const refreshLabel = refreshing ? "Refreshing..." : "Refresh";

  appRoot.innerHTML = `
    <div class="shell today-shell" data-state="${pulse.healthState}">
      <header class="topbar">
        <div class="brand-mark" aria-label="System Pulse health state">
          <span class="heart" aria-hidden="true">&hearts;</span>
          <span class="score">${pulse.systemScore}</span>
        </div>
        <div>
          <p class="eyebrow">Today</p>
          <h1>${USER_NAME}'s Today</h1>
          <p class="topbar-subtitle">${escapeHtml(liveStatusLine(pulse, refreshing))}</p>
        </div>
        <div class="topbar-actions">
          <span class="version-pill">v${APP_VERSION}</span>
          <span class="platform">${pulse.platform}</span>
          <button id="quick-view-button" type="button">Check-in</button>
          <button id="refresh-button" type="button" ${refreshing ? "disabled" : ""}>${refreshLabel}</button>
        </div>
      </header>

      ${todayStatusGrid(pulse, refreshing)}
      ${todaysOpportunities(pulse)}
      ${localDetails(pulse)}

      <footer>
        <span>${escapeHtml(liveStatusLine(pulse, refreshing))}</span>
        <span>No cloud. No account. No automatic optimisation.</span>
      </footer>
    </div>
  `;

  document.querySelector<HTMLButtonElement>("#refresh-button")?.addEventListener("click", () => {
    void loadToday({ keepExisting: true });
  });
  document.querySelector<HTMLButtonElement>("#quick-view-button")?.addEventListener("click", () => {
    currentView = "quick";
    void invoke("open_quick_checkin");
    renderQuickCheckin(pulse);
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
      careMessage = `${application.name} will come back in about 30 minutes if System Pulse is still open.`;
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
      careMessage = `${application.name} is ignored for today.`;
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
  const target = application.actionTarget || application.name;
  const confirmed = window.confirm(
    `System Pulse will ask macOS to ${application.actionLabel.toLowerCase()} for ${target}. Continue?`,
  );
  if (!confirmed) return;

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

async function performDomainAction(actionKind: string): Promise<void> {
  careMessage = "Opening the right place in System Settings...";
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
      careMessage = "The last live check missed. System Pulse will try again shortly.";
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
  }, AUTO_REFRESH_MS);
}

void listen("system-pulse-refresh", () => {
  void loadToday({ keepExisting: true });
});
void listen("system-pulse-show-quick-checkin", () => {
  currentView = "quick";
  if (currentPulse) {
    renderQuickCheckin(currentPulse);
  }
});
void listen("system-pulse-show-today", () => {
  currentView = "today";
  if (currentPulse) {
    renderToday(currentPulse);
  }
});
window.addEventListener("focus", () => {
  void loadToday({ keepExisting: true, quiet: true });
});
document.addEventListener("visibilitychange", () => {
  if (!document.hidden) {
    void loadToday({ keepExisting: true, quiet: true });
  }
});
startAutoRefresh();
void loadToday();
