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
let currentPulse: TodayPulse | null = null;
let isRefreshing = false;
let currentView: ViewMode = "quick";

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

function nextCareCard(pulse: TodayPulse): string {
  const application = pulse.topApplications[0];
  if (pulse.healthState === "healthy" || !application) {
    return "";
  }

  return `
    <section class="card care-card">
      <p class="eyebrow">Lowest disruption</p>
      <h2>${escapeHtml(application.careLabel)}</h2>
      <p>${escapeHtml(application.careDetail)}</p>
      <div class="time-gain">
        <span>Estimated additional uninterrupted work</span>
        <strong>${escapeHtml(pulse.estimatedAdditionalWorkLabel)}</strong>
      </div>
      <details class="quiet-details">
        <summary>Why this app?</summary>
        <p>${escapeHtml(application.impactLabel)}</p>
      </details>
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

        <p class="quick-footnote">Local check only. Nothing changes on your Mac.</p>
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
  const careCard = nextCareCard(pulse);

  appRoot.innerHTML = `
    <div class="shell today-shell" data-state="${pulse.healthState}">
      <header class="topbar">
        <div class="brand-mark" aria-label="System Pulse health state">
          <span class="heart" aria-hidden="true">&hearts;</span>
          <span class="score">${pulse.systemScore}</span>
        </div>
        <div>
          <p class="eyebrow">Today</p>
          <h1>${USER_NAME}'s next decision</h1>
          <p class="topbar-subtitle">Why, and what to do next.</p>
        </div>
        <div class="topbar-actions">
          <span class="platform">${pulse.platform}</span>
          <button id="quick-view-button" type="button">Check-in</button>
          <button id="refresh-button" type="button" ${refreshing ? "disabled" : ""}>${refreshLabel}</button>
        </div>
      </header>

      <section class="hero card">
        <div class="hero-score">
          <span class="heart large-heart" aria-hidden="true">&hearts;</span>
          <strong>${pulse.systemScore}</strong>
        </div>
        <div class="hero-copy">
          <p class="eyebrow">Can I keep working?</p>
          <h2>${canKeepWorking(pulse.healthState)}</h2>
          <p>${comfortLine(pulse.healthState)}</p>
          <div class="hero-facts">
            <span><b>Estimated uninterrupted work time</b> ${escapeHtml(pulse.flowRemainingLabel)}</span>
            <span><b>Right now</b> ${escapeHtml(immediateAction(pulse))}</span>
          </div>
        </div>
      </section>

      <section class="decision-grid">
        <section class="card why-card">
          <p class="eyebrow">Why?</p>
          <h2>${whyHeadline(pulse.healthState)}</h2>
          <p>${escapeHtml(pulse.primaryExplanation)}</p>
        </section>

        <section class="card action-card">
          <p class="eyebrow">Next best step</p>
          <h2>${escapeHtml(immediateAction(pulse))}</h2>
          <p>${pulse.healthState === "healthy" ? "Close this and keep working." : "Choose the lowest-disruption moment. Protect active work first."}</p>
          <div class="time-gain">
            <span>Estimated additional uninterrupted work</span>
            <strong>${escapeHtml(pulse.estimatedAdditionalWorkLabel)}</strong>
          </div>
        </section>
      </section>

      ${careCard}
      ${localDetails(pulse)}

      <footer>
        <span>Collected locally at ${escapeHtml(formatCollectedAt(pulse.collectedAt))}</span>
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

async function loadToday(options: { keepExisting?: boolean } = {}): Promise<void> {
  if (isRefreshing) return;
  isRefreshing = true;

  if (options.keepExisting && currentPulse) {
    renderCurrentView(currentPulse, true);
  } else {
    renderLoading();
  }

  try {
    const pulse = await invoke<TodayPulse>("get_today_pulse");
    currentPulse = pulse;
    renderCurrentView(pulse);
    await updateTray(pulse);
  } catch (error) {
    renderError(error instanceof Error ? error.message : String(error));
  } finally {
    isRefreshing = false;
  }
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
void loadToday();
