<script lang="ts">
  import { onMount } from 'svelte';
  import { getSymptoms, saveSymptom, deleteSymptom, reorderSymptoms } from '$lib/db';
  import type { Symptom } from '$lib/types';

  let symptoms: Symptom[] = $state([]);
  let loading = $state(true);

  // New symptom form
  let newName = $state('');
  let newCategory = $state('');
  let newDescription = $state('');

  // Edit state
  let editingId: number | null = $state(null);
  let editName = $state('');
  let editCategory = $state('');
  let editDescription = $state('');
  let editActive = $state(true);

  onMount(loadSymptoms);

  async function loadSymptoms() {
    loading = true;
    try {
      symptoms = await getSymptoms();
    } catch (e) {
      console.error('Failed to load symptoms:', e);
    }
    loading = false;
  }

  async function handleAdd() {
    if (!newName.trim()) return;
    try {
      await saveSymptom({
        id: null,
        name: newName.trim(),
        category: newCategory.trim(),
        description: newDescription.trim(),
        active: true,
        sort_order: 0,
      });
      newName = '';
      newCategory = '';
      newDescription = '';
      await loadSymptoms();
    } catch (e) {
      console.error('Failed to add symptom:', e);
    }
  }

  function startEdit(s: Symptom) {
    editingId = s.id;
    editName = s.name;
    editCategory = s.category;
    editDescription = s.description;
    editActive = s.active;
  }

  function cancelEdit() {
    editingId = null;
  }

  async function handleSaveEdit() {
    if (!editName.trim() || editingId == null) return;
    const symptom = symptoms.find(s => s.id === editingId);
    if (!symptom) return;
    try {
      await saveSymptom({
        id: editingId,
        name: editName.trim(),
        category: editCategory.trim(),
        description: editDescription.trim(),
        active: editActive,
        sort_order: symptom.sort_order,
      });
      editingId = null;
      await loadSymptoms();
    } catch (e) {
      console.error('Failed to save symptom:', e);
    }
  }

  async function handleDelete(id: number) {
    if (!confirm('Delete this symptom? This will also delete all its log entries.')) return;
    try {
      await deleteSymptom(id);
      await loadSymptoms();
    } catch (e) {
      console.error('Failed to delete symptom:', e);
    }
  }

  async function moveUp(index: number) {
    if (index === 0) return;
    const ids = symptoms.map(s => s.id!);
    [ids[index - 1], ids[index]] = [ids[index], ids[index - 1]];
    try {
      await reorderSymptoms(ids);
      await loadSymptoms();
    } catch (e) {
      console.error('Failed to reorder:', e);
    }
  }

  async function moveDown(index: number) {
    if (index >= symptoms.length - 1) return;
    const ids = symptoms.map(s => s.id!);
    [ids[index], ids[index + 1]] = [ids[index + 1], ids[index]];
    try {
      await reorderSymptoms(ids);
      await loadSymptoms();
    } catch (e) {
      console.error('Failed to reorder:', e);
    }
  }
</script>

<div class="symptom-editor">
  <h1>Manage Symptoms</h1>

  <div class="add-form">
    <h3>Add New Symptom</h3>
    <div class="form-row">
      <input type="text" bind:value={newName} placeholder="Symptom name" />
      <input type="text" bind:value={newCategory} placeholder="Category" />
      <input type="text" bind:value={newDescription} placeholder="Description (optional)" />
      <button class="primary" onclick={handleAdd} disabled={!newName.trim()}>Add</button>
    </div>
  </div>

  {#if loading}
    <p class="muted">Loading...</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th class="col-order"></th>
          <th>Name</th>
          <th>Category</th>
          <th>Description</th>
          <th>Active</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each symptoms as symptom, i}
          {#if editingId === symptom.id}
            <tr class="editing-row">
              <td></td>
              <td><input type="text" class="edit-input" bind:value={editName} /></td>
              <td><input type="text" class="edit-input" bind:value={editCategory} /></td>
              <td><input type="text" class="edit-input" bind:value={editDescription} /></td>
              <td>
                <input type="checkbox" bind:checked={editActive} />
              </td>
              <td class="actions">
                <button onclick={handleSaveEdit}>Save</button>
                <button onclick={cancelEdit}>Cancel</button>
              </td>
            </tr>
          {:else}
            <tr class:inactive={!symptom.active}>
              <td class="order-buttons">
                <button class="arrow-btn" onclick={() => moveUp(i)} disabled={i === 0}>&uarr;</button>
                <button class="arrow-btn" onclick={() => moveDown(i)} disabled={i === symptoms.length - 1}>&darr;</button>
              </td>
              <td class="name">{symptom.name}</td>
              <td class="category">{symptom.category}</td>
              <td class="desc">{symptom.description || '--'}</td>
              <td>{symptom.active ? 'Yes' : 'No'}</td>
              <td class="actions">
                <button onclick={() => startEdit(symptom)}>Edit</button>
                <button class="danger" onclick={() => symptom.id && handleDelete(symptom.id)}>Delete</button>
              </td>
            </tr>
          {/if}
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .symptom-editor { max-width: 900px; }

  h1 { margin-bottom: 20px; }

  .add-form {
    margin-bottom: 24px;
    padding: 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
  }

  .add-form h3 { margin-bottom: 8px; }

  .form-row {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .form-row input {
    flex: 1;
    min-width: 120px;
  }

  .muted { color: var(--color-text-muted); }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th {
    text-align: left;
    padding: 6px 8px;
    color: var(--color-text-muted);
    font-weight: 500;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    border-bottom: 1px solid var(--color-border);
  }

  td {
    padding: 5px 8px;
    border-bottom: 1px solid var(--color-border);
    font-size: 13px;
  }

  .col-order { width: 50px; }
  .name { font-weight: 500; }
  .category { color: var(--color-text-muted); }
  .desc { color: var(--color-text-muted); max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .inactive td { opacity: 0.5; }

  .order-buttons {
    display: flex;
    gap: 2px;
  }

  .arrow-btn {
    padding: 1px 5px;
    font-size: 11px;
    border: 1px solid var(--color-border);
    background: none;
    border-radius: 3px;
    line-height: 1;
  }

  .arrow-btn:disabled {
    opacity: 0.3;
    cursor: default;
  }

  .actions {
    text-align: right;
    white-space: nowrap;
  }

  .actions button {
    font-size: 12px;
    padding: 2px 6px;
    margin-left: 3px;
  }

  .edit-input {
    width: 100%;
    font-size: 13px;
    padding: 3px 6px;
  }

  .editing-row {
    background: var(--color-surface-raised);
  }
</style>
