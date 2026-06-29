import { invoke } from "@tauri-apps/api/core";
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
  topApplications: ApplicationImpact[];
};

const app = document.querySelector<HTMLElement>("#app");

if (!app) {
  throw new Error("System Pulse root element is missing.");
}

const appRoot = app;

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

function appRow(application: ApplicationImpact): string {
  return `
    <li class="app-row">
      <div>
        <strong>${escapeHtml(application.name)}</strong>
        <span>${escapeHtml(application.detail)}</span>
      </div>
      <div class="app-metrics">
        <span>${escapeHtml(application.memoryDisplay)}</span>
        <b>${escapeHtml(application.impactLabel)}</b>
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
      <span class="metric-pill">${escapeHtml(domain.value)}</span>
    </section>
  `;
}

function renderToday(pulse: TodayPulse): void {
  const applications = pulse.topApplications.length
    ? pulse.topApplications.map(appRow).join("")
    : `<li class="app-row empty-row"><div><strong>No heavy applications</strong><span>Nothing is standing out right now.</span></div></li>`;

  appRoot.innerHTML = `
    <div class="shell" data-state="${pulse.healthState}">
      <header class="topbar">
        <div class="brand-mark" aria-label="System Pulse health state">
          <span class="heart" aria-hidden="true">&hearts;</span>
          <span class="score">${pulse.systemScore}</span>
        </div>
        <div>
          <p class="eyebrow">Today</p>
          <h1>Your system is ${healthText(pulse.healthState).toLowerCase()}.</h1>
        </div>
        <span class="platform">${pulse.platform}</span>
      </header>

      <section class="hero card">
        <div class="hero-score">
          <span class="heart large-heart" aria-hidden="true">&hearts;</span>
          <strong>${pulse.systemScore}</strong>
          <span>${healthText(pulse.healthState)}</span>
        </div>
        <div class="hero-copy">
          <p class="eyebrow">Recommended for you</p>
          <h2>${escapeHtml(pulse.primaryRecommendation)}</h2>
          <p>${escapeHtml(pulse.primaryExplanation)}</p>
          <div class="hero-facts">
            <span><b>${escapeHtml(pulse.expectedImprovement)}</b> expected improvement</span>
            <span><b>${pulse.confidence}%</b> confidence</span>
          </div>
        </div>
      </section>

      <section class="grid">
        ${domainCard("Memory", pulse.memoryHealth)}
        ${domainCard("Storage", pulse.storageHealth)}
      </section>

      <section class="card">
        <div class="card-heading">
          <span>Applications</span>
          <b>Memory impact</b>
        </div>
        <ul class="app-list">
          ${applications}
        </ul>
      </section>

      <footer>
        <span>Collected locally at ${escapeHtml(pulse.collectedAt)}</span>
        <span>No cloud. No account. No automatic optimisation.</span>
      </footer>
    </div>
  `;
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
        <h1>Something unexpected happened.</h1>
        <p>${escapeHtml(message)}</p>
        <button id="retry-button" type="button">Try again</button>
      </section>
    </div>
  `;
  document.querySelector<HTMLButtonElement>("#retry-button")?.addEventListener("click", loadToday);
}

async function loadToday(): Promise<void> {
  renderLoading();
  try {
    const pulse = await invoke<TodayPulse>("get_today_pulse");
    renderToday(pulse);
  } catch (error) {
    renderError(error instanceof Error ? error.message : String(error));
  }
}

void loadToday();
