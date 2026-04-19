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

  function kindGlyph(item) {
    const kind = classify(item);
    if (kind === "app") return "◉";
    if (kind === "file") return "▦";
    if (kind === "folder") return "▣";
    return "◌";
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
                <span class="title">
                  <span class="glyph">{kindGlyph(item)}</span>
                  {item.title}
                </span>
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
    padding: 8px;
    background: transparent;
    backdrop-filter: none;
  }

  .overlay {
    width: 100%;
    min-height: 100%;
    border: 1px solid rgba(255, 255, 255, 0.45);
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.9), rgba(239, 246, 255, 0.82)),
      rgba(230, 239, 251, 0.9);
    backdrop-filter: blur(34px) saturate(1.3);
    border-radius: 20px;
    box-shadow:
      0 26px 70px rgba(35, 58, 102, 0.32),
      inset 0 1px 0 rgba(255, 255, 255, 0.75),
      inset 0 -1px 0 rgba(255, 255, 255, 0.18);
    display: grid;
    grid-template-rows: auto 1fr auto;
    overflow: hidden;
  }

  .search-box {
    padding: 14px 14px 10px;
    border-bottom: 1px solid rgba(167, 186, 210, 0.42);
    display: grid;
    gap: 10px;
  }

  input {
    width: 100%;
    border: 1px solid rgba(255, 255, 255, 0.5);
    background: rgba(248, 252, 255, 0.86);
    color: #304764;
    border-radius: 12px;
    padding: 11px 14px;
    font-size: 1rem;
    outline: none;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.68);
  }

  input::placeholder {
    color: rgba(54, 82, 113, 0.7);
  }

  input:focus {
    border-color: rgba(130, 175, 255, 0.75);
    box-shadow: 0 0 0 3px rgba(144, 183, 255, 0.24);
  }

  .filters {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .filters button {
    border: 1px solid rgba(255, 255, 255, 0.4);
    background: rgba(249, 252, 255, 0.88);
    color: rgba(54, 78, 106, 0.9);
    border-radius: 999px;
    padding: 4px 9px;
    font-size: 0.7rem;
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
    border-color: rgba(163, 194, 255, 0.82);
    background: rgba(224, 239, 255, 0.95);
    color: rgba(33, 60, 92, 0.96);
  }

  .count {
    border-radius: 999px;
    background: rgba(94, 128, 167, 0.22);
    padding: 2px 7px;
    font-size: 0.7rem;
  }

  .content {
    min-height: 0;
    display: grid;
    grid-template-columns: 1.1fr 0.9fr;
  }

  .results {
    overflow: auto;
    border-right: 1px solid rgba(167, 186, 210, 0.42);
    padding: 6px 8px;
  }

  .results button {
    width: 100%;
    border: 1px solid transparent;
    background: transparent;
    color: #2f4764;
    text-align: left;
    border-radius: 10px;
    padding: 8px 10px;
    margin-bottom: 2px;
    cursor: pointer;
    transition: 120ms ease;
    opacity: 0;
    transform: translateY(4px);
    animation: listIn 240ms ease forwards;
    animation-delay: calc(var(--i) * 14ms);
  }

  .results button:hover,
  .results button.selected {
    background: rgba(209, 231, 255, 0.92);
    border-color: rgba(165, 195, 255, 0.62);
  }

  .row-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .title {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    font-weight: 600;
    font-size: 0.98rem;
  }

  .glyph {
    width: 20px;
    height: 20px;
    display: inline-grid;
    place-items: center;
    border-radius: 999px;
    font-size: 0.7rem;
    color: rgba(59, 84, 113, 0.84);
    background: rgba(255, 255, 255, 0.45);
    border: 1px solid rgba(255, 255, 255, 0.65);
  }

  .kind {
    font-size: 0.7rem;
    color: rgba(76, 104, 137, 0.9);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .subtitle {
    margin-top: 2px;
    font-size: 0.84rem;
    color: rgba(84, 114, 148, 0.84);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .preview {
    padding: 14px;
    display: grid;
    align-content: start;
    gap: 10px;
  }

  .preview h2 {
    margin: 0;
    font-size: 1.75rem;
    color: #213954;
    letter-spacing: -0.01em;
  }

  .preview p {
    margin: 0;
    color: rgba(69, 95, 128, 0.88);
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
    font-size: 0.76rem;
    background: rgba(248, 252, 255, 0.86);
    border: 1px solid rgba(255, 255, 255, 0.52);
    color: rgba(64, 92, 124, 0.94);
    border-radius: 999px;
    padding: 4px 9px;
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
    border: 1px solid rgba(255, 255, 255, 0.56);
    background: rgba(247, 251, 255, 0.9);
    color: rgba(40, 65, 94, 0.94);
    border-radius: 10px;
    padding: 7px 11px;
    cursor: pointer;
    font-weight: 600;
  }

  .actions button:hover {
    background: rgba(233, 244, 255, 1);
  }

  .hint {
    font-size: 0.78rem;
    color: rgba(69, 96, 130, 0.88);
  }

  .status {
    border-top: 1px solid rgba(167, 186, 210, 0.42);
    color: rgba(74, 104, 139, 0.86);
    padding: 8px 12px;
    font-size: 0.76rem;
    background: rgba(246, 250, 255, 0.78);
  }

  .empty {
    color: rgba(80, 108, 141, 0.88);
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
