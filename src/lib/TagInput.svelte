<script lang="ts">
  interface Props {
    tags: string[];
    suggestions: string[];
    onchange: (tags: string[]) => void;
  }
  let { tags, suggestions, onchange }: Props = $props();

  let inputValue = $state('');
  let showSuggestions = $state(false);

  let filteredSuggestions = $derived(
    inputValue.length > 0
      ? suggestions
          .filter(s => s.toLowerCase().includes(inputValue.toLowerCase()) && !tags.includes(s))
          .slice(0, 8)
      : []
  );

  function addTag(value: string) {
    const trimmed = value.trim().toLowerCase();
    if (trimmed && !tags.includes(trimmed)) {
      onchange([...tags, trimmed]);
    }
    inputValue = '';
    showSuggestions = false;
  }

  function removeTag(tag: string) {
    onchange(tags.filter(t => t !== tag));
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      addTag(inputValue);
    } else if (e.key === 'Backspace' && inputValue === '' && tags.length > 0) {
      removeTag(tags[tags.length - 1]);
    } else if (e.key === 'Escape') {
      showSuggestions = false;
    }
  }
</script>

<div class="tag-input-wrapper">
  <div class="tags-container">
    {#each tags as tag}
      <span class="tag">
        {tag}
        <button class="tag-remove" onclick={() => removeTag(tag)}>×</button>
      </span>
    {/each}
    <input
      type="text"
      bind:value={inputValue}
      {onkeydown}
      onfocus={() => (showSuggestions = true)}
      onblur={() => setTimeout(() => (showSuggestions = false), 150)}
      placeholder={tags.length === 0 ? 'Add symptom tags...' : ''}
      class="tag-text-input"
    />
  </div>
  {#if showSuggestions && filteredSuggestions.length > 0}
    <ul class="suggestions">
      {#each filteredSuggestions as s}
        <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
        <li role="option" aria-selected="false">
          <button onmousedown={() => addTag(s)}>{s}</button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .tag-input-wrapper {
    position: relative;
    width: 100%;
  }
  .tags-container {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    align-items: center;
    min-height: 36px;
    padding: 4px 8px;
    border: 1px solid var(--color-border, #e5e7eb);
    border-radius: 6px;
    background: var(--color-surface, #ffffff);
    cursor: text;
  }
  .tag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    border-radius: 12px;
    background: var(--color-accent, #2E6171);
    color: #ffffff;
    font-size: 0.8rem;
  }
  .tag-remove {
    background: none;
    border: none;
    cursor: pointer;
    color: inherit;
    padding: 0;
    line-height: 1;
    opacity: 0.7;
  }
  .tag-remove:hover { opacity: 1; }
  .tag-text-input {
    flex: 1;
    min-width: 120px;
    background: none;
    border: none;
    outline: none;
    color: var(--color-text, #1a1a1a);
    font-size: 0.9rem;
  }
  .suggestions {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin: 2px 0 0;
    padding: 4px 0;
    list-style: none;
    background: var(--color-surface, #ffffff);
    border: 1px solid var(--color-border, #e5e7eb);
    border-radius: 6px;
    box-shadow: var(--shadow, 0 1px 4px rgba(0,0,0,0.08));
    z-index: 100;
    max-height: 200px;
    overflow-y: auto;
  }
  .suggestions li button {
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    padding: 6px 12px;
    cursor: pointer;
    color: var(--color-text, #1a1a1a);
    font-size: 0.9rem;
  }
  .suggestions li button:hover {
    background: var(--color-surface-raised, #f9fafb);
  }
</style>
