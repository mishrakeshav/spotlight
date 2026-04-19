<script>
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";

  let query = "";
  let results = [];
  let selected = 0;
  let status = "Ready";
  let queryInput;
  let activeFilter = "all";
  let actionPanelOpen = false;
  let searchTimer;
  const appWindow = getCurrentWindow();

  const filters = ["all", "app", "file", "folder", "other"];

  function classify(item) {
    const kind = String(item.kind || "").toLowerCase();
    if (kind === "app") return "app";
    if (kind === "file") return "file";
    if (kind === "folder") return "folder";
    return "other";
  }

  function displayKind(item) {
    return classify(item).toUpperCase();
  }

  function scheduleSearch() {
    clearTimeout(searchTimer);
    searchTimer = setTimeout(() => {
      void runSearch();
    }, 80);
  }

  async function runSearch() {
    try {
      const data = await invoke("search", { query, limit: 30 });
      results = data;
      if (selected >= filteredResults.length) {
        selected = Math.max(filteredResults.length - 1, 0);
      }
      status = `${results.length} results`;
    } catch (err) {
      status = `search error: ${err}`;
    }
  }

  async function openSelected() {
    const item = selectedItem;
    if (!item) return;
    try {
      const msg = await invoke("open_result", { id: item.id });
      status = msg;
      await appWindow.hide();
    } catch (err) {
      status = `open error: ${err}`;
    }
  }

  async function revealSelected() {
    const item = selectedItem;
    if (!item) return;
    try {
      const msg = await invoke("reveal_result", { id: item.id });
      status = msg;
    } catch (err) {
      status = `reveal error: ${err}`;
    }
  }

  async function copySelectedPath() {
    const item = selectedItem;
    if (!item) return;

    try {
      const path = await invoke("path_for_result", { id: item.id });
      if (!path) {
        status = "No path available for this result";
        return;
      }
      await navigator.clipboard.writeText(path);
      status = `copied path: ${path}`;
    } catch (err) {
      status = `copy error: ${err}`;
    }
  }

  function selectFilter(filter) {
    activeFilter = filter;
    selected = 0;
  }

  function onKeyDown(event) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      selected = Math.min(selected + 1, Math.max(filteredResults.length - 1, 0));
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      selected = Math.max(selected - 1, 0);
    } else if (event.key === "Enter") {
      event.preventDefault();
      void openSelected();
    } else if (event.key === "Tab" || event.key === "ArrowRight") {
      event.preventDefault();
      actionPanelOpen = true;
    } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "c") {
      event.preventDefault();
      void copySelectedPath();
    } else if (event.key.toLowerCase() === "r") {
      event.preventDefault();
      void revealSelected();
    } else if (event.key === "Escape") {
      event.preventDefault();
      if (actionPanelOpen) {
        actionPanelOpen = false;
      } else if (query) {
        query = "";
        scheduleSearch();
      }
    }
  }

  $: filteredResults =
    activeFilter === "all"
      ? results
      : results.filter((item) => classify(item) === activeFilter);

  $: counts = {
    all: results.length,
    app: results.filter((item) => classify(item) === "app").length,
    file: results.filter((item) => classify(item) === "file").length,
    folder: results.filter((item) => classify(item) === "folder").length,
    other: results.filter((item) => classify(item) === "other").length,
  };

  $: if (selected >= filteredResults.length) {
    selected = Math.max(filteredResults.length - 1, 0);
  }

  $: selectedItem = filteredResults[selected];

  onMount(() => {
    queryInput?.focus();
    void runSearch();
    return () => clearTimeout(searchTimer);
  });
</script>

<div class="backdrop">
  <div class="overlay" role="application">
    <div class="search-box">
      <input
        bind:this={queryInput}
        bind:value={query}
        on:input={scheduleSearch}
        on:keydown={onKeyDown}
        placeholder="Search apps, files, folders..."
      />

      <div class="filters" aria-label="Result filters">
        {#each filters as filter}
          <button
            class:active={activeFilter === filter}
            on:click={() => selectFilter(filter)}
          >
            <span>{filter}</span>
            <span class="count">{counts[filter]}</span>
          </button>
        {/each}
      </div>
    </div>

    <div class="content">
      <section class="results">
        {#if filteredResults.length === 0}
          <p class="empty">No matches in this filter</p>
        {:else}
          {#each filteredResults as item, idx (item.id)}
            <button
              style={`--i:${idx}`}
              class:selected={idx === selected}
              on:click={() => (selected = idx)}
              on:dblclick={() => void openSelected()}
            >
              <div class="row-top">
                <span class="title">{item.title}</span>
                <span class="kind">{displayKind(item)}</span>
              </div>
              {#if item.subtitle}
                <div class="subtitle">{item.subtitle}</div>
              {/if}
            </button>
          {/each}
        {/if}
      </section>

      <section class="preview">
        {#if selectedItem}
          <h2>{selectedItem.title}</h2>
          <p>{selectedItem.subtitle}</p>
          <div class="meta">
            <div class="pill">Kind {displayKind(selectedItem)}</div>
            <div class="pill">Score {selectedItem.score.toFixed(2)}</div>
          </div>

          <div class="actions" class:open={actionPanelOpen}>
            <button on:click={() => void openSelected()}>Open</button>
            <button on:click={() => void revealSelected()}>Reveal</button>
            <button on:click={() => void copySelectedPath()}>Copy Path</button>
          </div>

          <p class="hint">Enter: Open | R: Reveal | Ctrl/Cmd+C: Copy Path | Tab: Action Panel</p>
        {:else}
          <p>Select an item to preview</p>
        {/if}
      </section>
    </div>

    <div class="status">{status}</div>
  </div>
</div>

<style>
  .backdrop {
    min-height: 100vh;
    width: 100%;
    display: grid;
    place-items: center;
    padding: 24px;
    background: rgba(4, 10, 16, 0.38);
    backdrop-filter: blur(16px) saturate(1.1);
  }

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
    display: grid;
    gap: 12px;
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

  .filters {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .filters button {
    border: 1px solid rgba(200, 247, 255, 0.22);
    background: rgba(200, 247, 255, 0.06);
    color: var(--text);
    border-radius: 999px;
    padding: 6px 10px;
    font-size: 0.76rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    transition: 150ms ease;
  }

  .filters button:hover,
  .filters button.active {
    border-color: rgba(77, 226, 186, 0.5);
    background: rgba(77, 226, 186, 0.16);
  }

  .count {
    border-radius: 999px;
    background: rgba(7, 18, 28, 0.7);
    padding: 2px 7px;
    font-size: 0.72rem;
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
    opacity: 0;
    transform: translateY(4px);
    animation: listIn 240ms ease forwards;
    animation-delay: calc(var(--i) * 14ms);
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

  .meta {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
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

  .actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    opacity: 0.85;
    transform: translateY(2px);
    transition: 140ms ease;
  }

  .actions.open,
  .actions:hover {
    opacity: 1;
    transform: translateY(0);
  }

  .actions button {
    border: 1px solid rgba(77, 226, 186, 0.45);
    background: rgba(77, 226, 186, 0.16);
    color: var(--text);
    border-radius: 10px;
    padding: 8px 12px;
    cursor: pointer;
  }

  .hint {
    font-size: 0.78rem;
    color: var(--muted);
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

  @keyframes listIn {
    from {
      opacity: 0;
      transform: translateY(4px);
    }

    to {
      opacity: 1;
      transform: translateY(0);
    }
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

    .actions {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
