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
  flowRemainingLabel: string;
  flowRemainingMinutes: number;
  memoryHealth: DomainHealth;
  storageHealth: DomainHealth;
  experienceHealth: DomainHealth;
  applicationHealth: DomainHealth;
  momentumHealth: DomainHealth;
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

function scoreInterpretation(score: number): string {
  if (score >= 95) return "Excellent";
  if (score >= 80) return "Working well";
  if (score >= 60) return "Worth watching";
  if (score >= 40) return "Plan some care";
  if (score >= 20) return "Recommended today";
  return "Act soon";
}

function todayHeading(): string {
  return `${USER_NAME}'s Today`;
}

function scoreFeeling(state: HealthState): string {
  if (state === "healthy") return "Everything feels steady today.";
  if (state === "attention") return "A few things are worth noticing.";
  return "Your computer needs a calmer moment.";
}

function quickHeadline(state: HealthState): string {
  if (state === "healthy") return "Everything is healthy.";
  if (state === "attention") return "A little care may help soon.";
  return "Your momentum needs care.";
}

function quickBody(state: HealthState): string {
  if (state === "healthy") return "You can keep working. No action is needed right now.";
  if (state === "attention") {
    return "You can keep working, but one thing is worth watching.";
  }
  return "Pause for care soon so your computer does not interrupt your work.";
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

function appRow(application: ApplicationImpact): string {
  return `
    <li class="app-row">
      <div class="app-main">
        <strong>${escapeHtml(application.name)}</strong>
        <b>${escapeHtml(application.careLabel)}</b>
        <span>${escapeHtml(application.detail)}</span>
      </div>
      <div class="app-care">
        <span>Why it matters</span>
        <strong>${escapeHtml(application.impactLabel)}</strong>
        <p>${escapeHtml(application.careDetail)}</p>
        <details class="app-details">
          <summary>Details</summary>
          <span>${escapeHtml(application.memoryDisplay)} observed locally</span>
        </details>
      </div>
    </li>
  `;
}

function domainCard(title: string, domain: DomainHealth): string {
  return `
    <section class="card compact-card">
      <div class="card-heading">
        <span>${escapeHtml(title)}</span>
        <b>${escapeHtml(domain.label)}</b>
      </div>
      <h3>${escapeHtml(domain.headline)}</h3>
      <p>${escapeHtml(domain.detail)}</p>
      <details class="metric-details">
        <summary>Details</summary>
        <span>${escapeHtml(domain.value)}</span>
      </details>
    </section>
  `;
}

function glanceRow(label: string, domain: DomainHealth): string {
  return `
    <li>
      <span class="glance-dot" aria-hidden="true"></span>
      <strong>${escapeHtml(label)}</strong>
      <b>${escapeHtml(domain.label)}</b>
    </li>
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
    glanceRow("Applications", pulse.applicationHealth),
    glanceRow("Storage", pulse.storageHealth),
    glanceRow("Memory", pulse.memoryHealth),
    glanceRow("Experience", pulse.experienceHealth),
  ].join("");

  appRoot.innerHTML = `
    <main class="quick-shell" data-state="${pulse.healthState}">
      <section class="quick-card">
        <div class="quick-tools">
          <span>System Pulse</span>
          <button id="quick-refresh-button" class="icon-button" type="button" ${refreshing ? "disabled" : ""}>${refreshLabel}</button>
        </div>

        <div class="quick-hero">
          <div class="quick-score" aria-label="System Pulse score ${pulse.systemScore}">
            <span class="heart quick-heart" aria-hidden="true">&hearts;</span>
            <strong>${pulse.systemScore}</strong>
          </div>
          <div class="quick-summary">
            <h1>${quickHeadline(pulse.healthState)}</h1>
            <p>${quickBody(pulse.healthState)}</p>
            <span>Estimated uninterrupted work time</span>
            <b>${escapeHtml(pulse.flowRemainingLabel)}</b>
          </div>
        </div>

        <div class="quick-care">
          <span>Next best step</span>
          <strong>${escapeHtml(pulse.primaryRecommendation)}</strong>
          <p>${escapeHtml(pulse.primaryExplanation)}</p>
        </div>

        <div class="quick-glance">
          <h2>At a Glance</h2>
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
  const applications = pulse.topApplications.length
    ? pulse.topApplications.map((application) => appRow(application)).join("")
    : `<li class="app-row empty-row"><div class="app-main"><strong>No action needed</strong><span>No application is standing out right now.</span></div><div class="app-care"><span>Why it matters</span><strong>Your apps are not competing with your work.</strong><p>Nothing is likely to interrupt your momentum.</p></div></li>`;
  const domainCards = [
    domainCard("Work", pulse.momentumHealth),
    pulse.browserHealth ? domainCard("Browser", pulse.browserHealth) : "",
    domainCard("Experience", pulse.experienceHealth),
    domainCard("Applications", pulse.applicationHealth),
    domainCard("Memory", pulse.memoryHealth),
    domainCard("Storage", pulse.storageHealth),
  ].join("");
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
          <h1>${todayHeading()}</h1>
          <p class="topbar-subtitle">Diagnosis and lowest-disruption care.</p>
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
          <span>${scoreInterpretation(pulse.systemScore)}</span>
        </div>
        <div class="hero-copy">
          <p class="eyebrow">Can I keep working?</p>
          <h2>${scoreFeeling(pulse.healthState)}</h2>
          <p class="primary-recommendation">${escapeHtml(pulse.primaryRecommendation)}</p>
          <p>${escapeHtml(pulse.primaryExplanation)}</p>
          <div class="hero-facts">
            <span><b>Estimated uninterrupted work time</b> ${escapeHtml(pulse.flowRemainingLabel)}</span>
            <span><b>Decision</b> ${scoreInterpretation(pulse.systemScore)}</span>
          </div>
        </div>
      </section>

      <section class="grid health-grid">
        ${domainCards}
      </section>

      <section class="card section-card">
        <div class="card-heading">
          <span>Care opportunities</span>
          <b>Lowest disruption first</b>
        </div>
        <ul class="app-list">
          ${applications}
        </ul>
      </section>

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
        <p>System Pulse is reading local signals and deciding what matters.</p>
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
