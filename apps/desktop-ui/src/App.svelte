<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let query = "";
  let results = [];
  let selected = 0;
  let status = "Ready";
  let queryInput;

  async function runSearch() {
    try {
      const data = await invoke("search", { query, limit: 20 });
      results = data;
      if (selected >= results.length) {
        selected = Math.max(results.length - 1, 0);
      }
      status = `${results.length} results`;
    } catch (err) {
      status = `search error: ${err}`;
    }
  }

  async function openSelected() {
    const item = results[selected];
    if (!item) return;
    try {
      const msg = await invoke("open_result", { id: item.id });
      status = msg;
    } catch (err) {
      status = `open error: ${err}`;
    }
  }

  function onKeyDown(event) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      selected = Math.min(selected + 1, Math.max(results.length - 1, 0));
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      selected = Math.max(selected - 1, 0);
    } else if (event.key === "Enter") {
      event.preventDefault();
      void openSelected();
    }
  }

  $: void runSearch();

  onMount(() => {
    queryInput?.focus();
  });
</script>

<div class="overlay" role="application">
  <div class="search-box">
    <input
      bind:this={queryInput}
      bind:value={query}
      on:keydown={onKeyDown}
      placeholder="Search apps, files, folders..."
    />
  </div>

  <div class="content">
    <section class="results">
      {#if results.length === 0}
        <p class="empty">No matches</p>
      {:else}
        {#each results as item, idx}
          <button
            class:selected={idx === selected}
            on:click={() => (selected = idx)}
            on:dblclick={() => void openSelected()}
          >
            <div class="row-top">
              <span class="title">{item.title}</span>
              <span class="kind">{item.kind}</span>
            </div>
            {#if item.subtitle}
              <div class="subtitle">{item.subtitle}</div>
            {/if}
          </button>
        {/each}
      {/if}
    </section>

    <section class="preview">
      {#if results[selected]}
        <h2>{results[selected].title}</h2>
        <p>{results[selected].subtitle}</p>
        <div class="pill">Score {results[selected].score.toFixed(2)}</div>
        <div class="actions">
          <button on:click={() => void openSelected()}>Open</button>
        </div>
      {:else}
        <p>Select an item to preview</p>
      {/if}
    </section>
  </div>

  <div class="status">{status}</div>
</div>

<style>
  .overlay {
    width: min(1080px, 100%);
    min-height: min(720px, 90vh);
    border: 1px solid var(--border);
    background: var(--panel);
    backdrop-filter: blur(18px);
    border-radius: 24px;
    box-shadow: 0 32px 80px rgba(0, 0, 0, 0.35);
    display: grid;
    grid-template-rows: auto 1fr auto;
    overflow: hidden;
  }

  .search-box {
    padding: 18px;
    border-bottom: 1px solid var(--border);
  }

  input {
    width: 100%;
    border: 1px solid rgba(220, 245, 255, 0.18);
    background: rgba(7, 18, 28, 0.75);
    color: var(--text);
    border-radius: 14px;
    padding: 14px 16px;
    font-size: 1.03rem;
    outline: none;
  }

  input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-soft);
  }

  .content {
    min-height: 0;
    display: grid;
    grid-template-columns: 1.3fr 1fr;
  }

  .results {
    overflow: auto;
    border-right: 1px solid var(--border);
    padding: 8px;
  }

  .results button {
    width: 100%;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text);
    text-align: left;
    border-radius: 12px;
    padding: 10px 12px;
    margin-bottom: 6px;
    cursor: pointer;
    transition: 120ms ease;
  }

  .results button:hover,
  .results button.selected {
    background: rgba(200, 247, 255, 0.08);
    border-color: rgba(200, 247, 255, 0.18);
  }

  .row-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .title {
    font-weight: 600;
  }

  .kind {
    font-size: 0.77rem;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .subtitle {
    margin-top: 4px;
    font-size: 0.85rem;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .preview {
    padding: 20px;
    display: grid;
    align-content: start;
    gap: 12px;
  }

  .preview h2 {
    margin: 0;
    font-size: 1.2rem;
  }

  .preview p {
    margin: 0;
    color: var(--muted);
    word-break: break-word;
  }

  .pill {
    display: inline-flex;
    width: fit-content;
    font-size: 0.8rem;
    background: rgba(77, 226, 186, 0.12);
    border: 1px solid rgba(77, 226, 186, 0.3);
    border-radius: 999px;
    padding: 4px 10px;
  }

  .actions button {
    border: 1px solid rgba(77, 226, 186, 0.45);
    background: rgba(77, 226, 186, 0.16);
    color: var(--text);
    border-radius: 10px;
    padding: 8px 12px;
    cursor: pointer;
  }

  .status {
    border-top: 1px solid var(--border);
    color: var(--muted);
    padding: 10px 14px;
    font-size: 0.83rem;
  }

  .empty {
    color: var(--muted);
    padding: 16px;
  }

  @media (max-width: 880px) {
    .content {
      grid-template-columns: 1fr;
      grid-template-rows: 1fr auto;
    }

    .results {
      border-right: 0;
      border-bottom: 1px solid var(--border);
      max-height: 44vh;
    }
  }
</style>
