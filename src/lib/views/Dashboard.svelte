<script lang="ts">
  import type { View, Diagnosis, DailyRating, AbnormalResult } from '$lib/types';
  import {
    getDiagnoses,
    getDailyRatings,
    getLatestAbnormalWithPrevious,
    getHeatmapConfig,
  } from '$lib/db';
  import { upcomingCare } from '$lib/demo';

  let { onNavigate, openGlossary }: {
    onNavigate: (view: View) => void;
    openGlossary?: (test?: string) => void;
  } = $props();

  let diagnoses: Diagnosis[] = $state([]);
  let ratings: DailyRating[] = $state([]);
  let abnormals: AbnormalResult[] = $state([]);
  let heatmapColors: string[] = $state([]);
  let heatmapLabels: string[] = $state([]);

  const today = new Date();
  const todayIso = isoDate(today);

  function isoDate(d: Date): string {
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  }
  function fmtLongDate(d: Date): string {
    return d.toLocaleDateString(undefined, { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' });
  }
  function greeting(d: Date): string {
    const h = d.getHours();
    if (h < 12) return 'Good morning';
    if (h < 18) return 'Good afternoon';
    return 'Good evening';
  }
  function fmtMonth(iso: string): string {
    return new Date(iso + 'T00:00').toLocaleDateString(undefined, { month: 'short' }).toUpperCase();
  }
  function fmtDay(iso: string): string {
    return String(new Date(iso + 'T00:00').getDate()).padStart(2, '0');
  }

  $effect(() => {
    getDiagnoses().then((rows) => {
      diagnoses = rows.filter((d) => !d.resolution_date);
    });
    getHeatmapConfig().then((c) => {
      heatmapColors = c.colors;
      heatmapLabels = c.labels;
    });
    getLatestAbnormalWithPrevious().then((rows) => {
      abnormals = rows;
    });
    // Fetch this year plus last year to cover the trailing 7-day window across year boundaries.
    const year = today.getFullYear();
    Promise.all([getDailyRatings(year), getDailyRatings(year - 1)]).then(([a, b]) => {
      ratings = [...a, ...b];
    });
  });

  const last7 = $derived.by(() => {
    const map = new Map(ratings.map((r) => [r.log_date, r]));
    const dow = ['S', 'M', 'T', 'W', 'T', 'F', 'S'];
    const out: { date: string; label: string; rating: DailyRating | null }[] = [];
    for (let i = 6; i >= 0; i--) {
      const d = new Date(today);
      d.setDate(d.getDate() - i);
      const iso = isoDate(d);
      out.push({ date: iso, label: dow[d.getDay()], rating: map.get(iso) ?? null });
    }
    return out;
  });

  const todayRating = $derived(last7[last7.length - 1]?.rating ?? null);
  const fallbackRating = $derived.by(() => {
    if (todayRating) return todayRating;
    for (let i = last7.length - 2; i >= 0; i--) if (last7[i].rating) return last7[i].rating;
    return null;
  });
  const wellnessLabel = $derived(
    fallbackRating ? (heatmapLabels[fallbackRating.wellness_score - 1] ?? '') : 'Not logged'
  );
  const wellnessColor = $derived(
    fallbackRating ? (heatmapColors[fallbackRating.wellness_score - 1] ?? 'var(--text-muted)') : 'var(--text-muted)'
  );
  const wellnessSub = $derived(
    todayRating
      ? 'Today, based on your last entry'
      : fallbackRating
        ? `As of ${fallbackRating.log_date}`
        : 'Log today to see your wellness score'
  );

  const upcoming = upcomingCare(3);
</script>

<div class="dashboard">
  <div class="header">
    <div class="date-line">{fmtLongDate(today)}</div>
    <h1 class="greeting">{greeting(today)}</h1>
  </div>

  <div class="cards">
    <section class="card wellness-card">
      <header class="card-header">
        <span class="eyebrow">WELLNESS</span>
        <button class="link" onclick={() => onNavigate('daily-rating')}>Open daily log →</button>
      </header>
      <div class="wellness-word" style="color: {wellnessColor}">{wellnessLabel}</div>
      <div class="wellness-sub">{wellnessSub}</div>
      <div class="last7-label">LAST 7 DAYS</div>
      <div class="last7">
        {#each last7 as day}
          <div class="last7-col">
            <div class="last7-letter">{day.label}</div>
            <div
              class="last7-cell"
              style="background: {day.rating && heatmapColors[day.rating.wellness_score - 1] ? heatmapColors[day.rating.wellness_score - 1] : 'var(--border)'}"
            ></div>
          </div>
        {/each}
      </div>
    </section>

    <section class="card diagnoses-card">
      <header class="card-header">
        <span class="eyebrow">ACTIVE DIAGNOSES</span>
        <button class="link" onclick={() => onNavigate('care-team')}>Care team →</button>
      </header>
      {#if diagnoses.length === 0}
        <div class="empty">No active diagnoses.</div>
      {:else}
        {#each diagnoses as d}
          <div class="dx-row">
            <div class="dx-main">
              <div class="dx-name">
                {d.name}
                {#if d.short_name}<span class="dx-code">{d.short_name}</span>{/if}
              </div>
              <div class="dx-sub">
                {#if d.onset_date}Since {d.onset_date}{/if}
                {#if d.source} · {d.source}{/if}
              </div>
            </div>
            {#if d.chronic}<span class="pill-tag">CHRONIC</span>{/if}
          </div>
        {/each}
      {/if}
    </section>

    <section class="card upcoming-card">
      <header class="card-header">
        <span class="eyebrow">UPCOMING CARE</span>
        <button class="link" onclick={() => onNavigate('care-team')}>See all →</button>
      </header>
      {#each upcoming as a}
        <div class="up-row">
          <div class="date-tile">
            <div class="date-tile-mo">{fmtMonth(a.date)}</div>
            <div class="date-tile-day">{fmtDay(a.date)}</div>
          </div>
          <div class="up-body">
            <div class="up-title">{a.title}</div>
            <div class="up-sub">{a.location}</div>
          </div>
        </div>
      {/each}
    </section>
  </div>

  <section class="card attention-card">
    <header class="card-header">
      <span class="eyebrow">
        NEEDS ATTENTION
        <span class="attention-count">{abnormals.length} abnormal values</span>
      </span>
      <button class="link" onclick={() => onNavigate('lab-results')}>View all labs →</button>
    </header>
    {#each abnormals as r}
      <div class="attn-row">
        <div class="attn-name">
          {r.test_name}
          {#if openGlossary}
            <button class="info-btn" onclick={() => openGlossary?.(r.test_name)} title="About this test">?</button>
          {/if}
        </div>
        <div class="attn-meta">
          <span class="attn-date">{r.test_date}</span>
          <span class="attn-value">{r.value ?? r.text_value}</span>
          <span class="attn-unit">{r.unit}</span>
          <span class="flag-pill" data-flag={r.flag}>{r.flag}</span>
        </div>
      </div>
    {/each}
  </section>
</div>

<style>
  .dashboard { max-width: 1160px; margin: 0 auto; padding: 24px 8px; }
  .header { margin-bottom: 24px; }
  .date-line { color: var(--text-muted); font-size: 13px; }
  .greeting { font-family: var(--font-heading); font-size: 32px; font-weight: 900; margin-top: 4px; }

  .cards { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; margin-bottom: 16px; }
  @media (max-width: 1000px) { .cards { grid-template-columns: 1fr; } }

  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 18px 20px;
  }

  .card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
  .eyebrow {
    font-size: 11px; font-weight: 700; letter-spacing: 0.08em;
    color: var(--text-muted); text-transform: uppercase;
  }
  .link {
    background: none; border: none; color: var(--primary);
    font-size: 12px; font-weight: 600; cursor: pointer; padding: 0;
  }

  .wellness-word { font-family: var(--font-heading); font-size: 44px; font-weight: 900; line-height: 1; margin: 8px 0 4px; }
  .wellness-sub { color: var(--text-muted); font-size: 13px; margin-bottom: 20px; }
  .last7-label { font-size: 10px; letter-spacing: 0.08em; color: var(--text-muted); margin-bottom: 6px; }
  .last7 { display: grid; grid-template-columns: repeat(7, 1fr); gap: 6px; }
  .last7-col { display: flex; flex-direction: column; align-items: center; gap: 4px; }
  .last7-letter { font-size: 11px; color: var(--text-muted); }
  .last7-cell { width: 100%; aspect-ratio: 1; border-radius: 6px; }

  .dx-row { display: flex; justify-content: space-between; align-items: flex-start; padding: 8px 0; gap: 8px; }
  .dx-name { font-weight: 600; }
  .dx-code { font-weight: 500; color: var(--text-muted); margin-left: 6px; font-size: 12px; }
  .dx-sub { color: var(--text-muted); font-size: 12px; margin-top: 2px; }
  .empty { color: var(--text-muted); font-size: 13px; padding: 8px 0; }

  .pill-tag {
    background: color-mix(in oklab, var(--primary) 12%, transparent);
    color: var(--primary);
    font-size: 10px; font-weight: 700; letter-spacing: 0.06em;
    padding: 3px 8px; border-radius: 999px;
  }

  .up-row { display: flex; align-items: center; gap: 12px; padding: 8px 0; }
  .date-tile {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 4px 10px;
    text-align: center;
    min-width: 46px;
  }
  .date-tile-mo { font-size: 10px; color: var(--text-muted); letter-spacing: 0.06em; }
  .date-tile-day { font-family: var(--font-heading); font-weight: 700; font-size: 16px; }
  .up-title { font-weight: 600; }
  .up-sub { color: var(--text-muted); font-size: 12px; }

  .attention-count { color: var(--accent-bad); margin-left: 6px; }
  .attn-row {
    display: flex; justify-content: space-between; align-items: center;
    padding: 10px 0; border-top: 1px solid var(--border);
  }
  .attn-row:first-of-type { border-top: none; }
  .attn-name { font-weight: 600; }
  .attn-meta { display: flex; align-items: center; gap: 12px; color: var(--text-muted); font-size: 12px; }
  .attn-value { font-family: var(--font-mono); color: var(--text); font-weight: 600; font-size: 15px; }
  .flag-pill {
    display: inline-flex; align-items: center; justify-content: center;
    width: 22px; height: 22px; border-radius: 6px;
    font-size: 11px; font-weight: 700; color: white;
  }
  .flag-pill[data-flag="H"], .flag-pill[data-flag="HH"] { background: var(--accent-high); }
  .flag-pill[data-flag="L"], .flag-pill[data-flag="LL"] { background: var(--accent-low); }
</style>
