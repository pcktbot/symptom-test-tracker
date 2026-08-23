<script lang="ts">
  import type { DailyRating, HeatmapConfig } from '$lib/types';
  import { scoreToColor, isValidDate } from '$lib/heatmap';

  interface Props {
    ratings: Map<string, DailyRating>;
    config: HeatmapConfig;
    selectedDate: string | null;
    year: number;
    onselect: (date: string) => void;
  }
  let { ratings, config, selectedDate, year, onselect }: Props = $props();

  const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
  const DAYS = Array.from({ length: 31 }, (_, i) => i + 1);

  function dateStr(month: number, day: number): string {
    return `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
  }

  const today = new Date().toISOString().slice(0, 10);
</script>

<div class="heatmap-scroll">
  <table class="heatmap-table">
    <thead>
      <tr>
        <th class="corner"></th>
        {#each MONTHS as month}
          <th class="month-header"><span>{month}</span></th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each DAYS as day}
        <tr>
          <td class="day-label">{day}</td>
          {#each Array.from({ length: 12 }, (_, i) => i + 1) as month}
            {@const valid = isValidDate(year, month, day)}
            {@const date = valid ? dateStr(month, day) : ''}
            {@const rating = valid ? ratings.get(date) : null}
            {@const color = rating ? scoreToColor(rating.wellness_score, config) : ''}
            {@const isToday = date === today}
            {@const isSelected = date === selectedDate}
            <td
              class="cell"
              class:invalid={!valid}
              class:today={isToday}
              class:selected={isSelected}
              class:scored={!!rating}
            >
              {#if valid}
                <button
                  style={color ? `background-color: ${color}` : ''}
                  onclick={() => onselect(date)}
                  title={date}
                ></button>
              {/if}
            </td>
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  .heatmap-scroll {
    overflow-x: auto;
    width: 100%;
  }
  .heatmap-table {
    border-collapse: collapse;
    table-layout: fixed;
  }
  .corner {
    width: 24px;
  }
  .month-header {
    width: 24px;
    padding: 0 2px 4px;
    text-align: center;
    font-size: 0.7rem;
    color: var(--text-muted);
    white-space: nowrap;
  }
  .month-header span {
    display: inline-block;
    transform: rotate(-45deg);
    transform-origin: bottom left;
    padding-bottom: 4px;
  }
  .day-label {
    padding: 1px 6px 1px 0;
    text-align: right;
    font-size: 0.7rem;
    color: var(--text-muted);
    width: 24px;
  }
  .cell {
    padding: 1px;
    width: 24px;
    height: 20px;
  }
  .cell button {
    display: block;
    width: 100%;
    height: 100%;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--surface);
    cursor: pointer;
    padding: 0;
    transition: opacity 0.1s, transform 0.1s;
  }
  .cell button:hover {
    opacity: 0.8;
    transform: scale(1.1);
    border-color: var(--border);
  }
  .cell.invalid button {
    display: none;
  }
  .cell.invalid {
    background: transparent;
  }
  .cell.today button {
    outline: 2px solid var(--primary);
    outline-offset: 1px;
  }
  .cell.selected button {
    outline: 2px solid var(--text);
    outline-offset: 1px;
  }
</style>
