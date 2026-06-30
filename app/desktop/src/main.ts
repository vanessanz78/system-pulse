import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./styles.css";

type HealthState = "healthy" | "attention" | "critical";

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
};

type TodayPulse = {
  collectedAt: string;
  platform: string;
  systemScore: number;
  healthState: HealthState;
  primaryExplanation: string;
  primaryRecommendation: string;
  confidence: number;
  expectedImprovement: string;
  memoryHealth: DomainHealth;
  storageHealth: DomainHealth;
  experienceHealth: DomainHealth;
  applicationHealth: DomainHealth;
  momentumHealth: DomainHealth;
  browserHealth?: DomainHealth;
  rendererHealth?: DomainHealth;
  windowServerHealth?: DomainHealth;
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

function healthText(state: HealthState): string {
  if (state === "healthy") return "Healthy";
  if (state === "attention") return "Needs attention";
  return "Immediate action";
}

function greeting(): string {
  const hour = new Date().getHours();
  if (hour < 5) return `Hello, ${USER_NAME}.`;
  if (hour < 12) return `Good morning, ${USER_NAME}.`;
  if (hour < 17) return `Good afternoon, ${USER_NAME}.`;
  if (hour < 22) return `Good evening, ${USER_NAME}.`;
  return `Hello, ${USER_NAME}.`;
}

function experienceLine(state: HealthState): string {
  if (state === "healthy") return "Your computer feels healthy today.";
  if (state === "attention") return "Your computer may feel a little heavier today.";
  return "Your computer may feel under pressure today.";
}

function scoreFeeling(state: HealthState): string {
  if (state === "healthy") return "Everything feels steady today.";
  if (state === "attention") return "A few things are worth noticing.";
  return "Your computer needs a calmer moment.";
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

function parseImprovement(value: string): number {
  const parsed = Number(value.replace("+", ""));
  return Number.isFinite(parsed) ? parsed : 0;
}

function expectedPulse(currentScore: number, improvement: string): number {
  return Math.min(100, currentScore + parseImprovement(improvement));
}

function expectedPulseLabel(currentScore: number, improvement: string): string {
  const pulse = expectedPulse(currentScore, improvement);
  return parseImprovement(improvement) > 0 ? `${pulse} after care` : `${pulse} now`;
}

function appRow(application: ApplicationImpact, currentScore: number): string {
  const improvement = parseImprovement(application.careEstimatedImprovement);
  const expectedPulse = Math.min(100, currentScore + improvement);
  const pulseCopy =
    improvement > 0
      ? `<em>Expected Pulse ${expectedPulse} after care</em>`
      : `<em>Pulse steady</em>`;

  return `
    <li class="app-row">
      <div class="app-main">
        <strong>${escapeHtml(application.name)}</strong>
        <b>${escapeHtml(application.impactLabel)}</b>
        <span>${escapeHtml(application.detail)}</span>
      </div>
      <div class="app-care">
        <span>Suggested Care</span>
        <strong>${escapeHtml(application.careLabel)}</strong>
        <p>${escapeHtml(application.careDetail)}</p>
        ${pulseCopy}
      </div>
      <div class="app-metrics">
        <span>Memory</span>
        <strong>${escapeHtml(application.memoryDisplay)}</strong>
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

function renderToday(pulse: TodayPulse, refreshing = false): void {
  const projectedPulse = expectedPulseLabel(pulse.systemScore, pulse.expectedImprovement);
  const applications = pulse.topApplications.length
    ? pulse.topApplications.map((application) => appRow(application, pulse.systemScore)).join("")
    : `<li class="app-row empty-row"><div class="app-main"><strong>No heavy applications</strong><span>Nothing is standing out right now.</span></div><div class="app-care"><span>Suggested Care</span><strong>No care needed</strong><p>Nothing is likely to interrupt your momentum.</p><em>Pulse steady</em></div></li>`;
  const domainCards = [
    domainCard("Momentum", pulse.momentumHealth),
    pulse.browserHealth ? domainCard("Browser Health", pulse.browserHealth) : "",
    domainCard("Experience", pulse.experienceHealth),
    domainCard("Applications", pulse.applicationHealth),
    domainCard("Memory", pulse.memoryHealth),
    domainCard("Storage", pulse.storageHealth),
  ].join("");
  const refreshLabel = refreshing ? "Refreshing..." : "Refresh";

  appRoot.innerHTML = `
    <div class="shell" data-state="${pulse.healthState}">
      <header class="topbar">
        <div class="brand-mark" aria-label="System Pulse health state">
          <span class="heart" aria-hidden="true">&hearts;</span>
          <span class="score">${pulse.systemScore}</span>
        </div>
        <div>
          <p class="eyebrow">Today</p>
          <h1>${greeting()}</h1>
          <p class="topbar-subtitle">${experienceLine(pulse.healthState)}</p>
        </div>
        <div class="topbar-actions">
          <span class="platform">${pulse.platform}</span>
          <button id="refresh-button" type="button" ${refreshing ? "disabled" : ""}>${refreshLabel}</button>
        </div>
      </header>

      <section class="hero card">
        <div class="hero-score">
          <span class="heart large-heart" aria-hidden="true">&hearts;</span>
          <strong>${pulse.systemScore}</strong>
          <span>${healthText(pulse.healthState)}</span>
        </div>
        <div class="hero-copy">
          <p class="eyebrow">Momentum</p>
          <h2>${scoreFeeling(pulse.healthState)}</h2>
          <p class="primary-recommendation">${escapeHtml(pulse.primaryRecommendation)}</p>
          <p>${escapeHtml(pulse.primaryExplanation)}</p>
          <div class="hero-facts">
            <span><b>Expected Pulse</b> ${projectedPulse}</span>
            <span><b>${pulse.confidence}%</b> confidence</span>
          </div>
        </div>
      </section>

      <section class="grid health-grid">
        ${domainCards}
      </section>

      <section class="card section-card">
        <div class="card-heading">
          <span>Applications</span>
          <b>PulseCore context</b>
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
}

function renderLoading(): void {
  appRoot.innerHTML = `
    <div class="shell loading-shell">
      <section class="card loading-card">
        <span class="heart large-heart" aria-hidden="true">&hearts;</span>
        <h1>Learning about your computer.</h1>
        <p>System Pulse is reading local system signals and asking PulseCore what matters.</p>
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
    });
  } catch {
    // Tray title updates are best-effort; the Today screen remains authoritative.
  }
}

async function loadToday(options: { keepExisting?: boolean } = {}): Promise<void> {
  if (isRefreshing) return;
  isRefreshing = true;

  if (options.keepExisting && currentPulse) {
    renderToday(currentPulse, true);
  } else {
    renderLoading();
  }

  try {
    const pulse = await invoke<TodayPulse>("get_today_pulse");
    currentPulse = pulse;
    renderToday(pulse);
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
void loadToday();
