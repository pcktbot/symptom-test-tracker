<script lang="ts">
  let { onDismiss }: { onDismiss: () => void } = $props();
</script>

<div class="welcome">
  <div class="welcome-card">
    <h1>Symptom Tracker</h1>
    <p class="tagline">Track lab results and symptoms for autoimmune conditions</p>

    <div class="features">
      <div class="feature">
        <h3>Lab Results</h3>
        <p>Enter and compare lab panels over time. Abnormal values are flagged automatically with trend indicators.</p>
      </div>
      <div class="feature">
        <h3>Symptom Logging</h3>
        <p>Daily check-in with a quick wellness rating and symptom checklist. Simple enough for every day.</p>
      </div>
      <div class="feature">
        <h3>Trends</h3>
        <p>Visualize lab values and symptom patterns over time. See how treatments affect your numbers.</p>
      </div>
      <div class="feature">
        <h3>Data Export</h3>
        <p>Export your data as CSV or JSON to share with your healthcare team or keep as a personal backup.</p>
      </div>
    </div>

    <div class="quickstart">
      <h2>Quick Start</h2>
      <ol>
        <li><strong>Enter lab results</strong> &mdash; Click "+ New Lab Entry" on the Dashboard to log your first set of results.</li>
        <li><strong>Set up symptom tracking</strong> &mdash; Go to Symptoms &gt; Manage to customize your symptom list, then use Symptoms &gt; Log to record daily check-ins.</li>
        <li><strong>Watch trends</strong> &mdash; After a few entries, visit Trends to see how your values change over time.</li>
      </ol>
    </div>

    <div class="about-info">
      <p>This app stores all data locally on your device. Nothing is sent to external servers. Your health data stays private.</p>
    </div>

    <button class="primary get-started" onclick={onDismiss}>Get Started</button>
  </div>
</div>

<style>
  .welcome {
    max-width: 640px;
    margin: 0 auto;
  }

  .welcome-card {
    padding: 8px 0;
  }

  h1 {
    font-size: 24px;
    font-weight: 700;
    margin-bottom: 4px;
  }

  .tagline {
    color: var(--color-text-muted);
    font-size: 15px;
    margin-bottom: 28px;
  }

  .features {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    margin-bottom: 28px;
  }

  .feature {
    padding: 14px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-surface-raised);
  }

  .feature h3 {
    font-size: 14px;
    font-weight: 600;
    margin-bottom: 4px;
  }

  .feature p {
    font-size: 13px;
    color: var(--color-text-muted);
    line-height: 1.5;
  }

  .quickstart {
    margin-bottom: 24px;
  }

  .quickstart h2 {
    font-size: 16px;
    margin-bottom: 10px;
  }

  .quickstart ol {
    padding-left: 20px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .quickstart li {
    font-size: 13px;
    line-height: 1.5;
    color: var(--color-text);
  }

  .about-info {
    padding: 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    margin-bottom: 24px;
  }

  .about-info p {
    font-size: 13px;
    color: var(--color-text-muted);
    line-height: 1.5;
  }

  .get-started {
    width: 100%;
    padding: 10px;
    font-size: 15px;
    font-weight: 600;
  }
</style>
