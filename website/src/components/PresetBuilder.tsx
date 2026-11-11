import { createMemo, createSignal, For, Show } from "solid-js";
import {
  applicationPlatforms, catalogGroups, createPreset, filterApplications,
  presetFilename, resolveSelection, starterPresets, toggleSelection, type Catalog,
} from "../lib/presets";

export default function PresetBuilder(props: { catalog: Catalog }) {
  const [query, setQuery] = createSignal("");
  const [group, setGroup] = createSignal("");
  const [ids, setIds] = createSignal<string[]>([]);
  const [name, setName] = createSignal("my-preset");
  const [message, setMessage] = createSignal("");
  const groups = catalogGroups(props.catalog);
  const visible = createMemo(() => filterApplications(props.catalog, query(), group()));
  const selected = createMemo(() => resolveSelection(props.catalog, ids()));
  const selectedIds = createMemo(() => new Set(selected().map(app => app.id)));
  const filename = createMemo(() => presetFilename(name()));
  const groupName = (id?: string) => groups.find(group => group.id === id)?.name ?? "Other";

  function changeSelection(next: string[]) {
    setIds(next);
    setMessage("");
  }

  function download(event: SubmitEvent) {
    event.preventDefault();
    try {
      const content = JSON.stringify(createPreset(props.catalog, ids()), null, 2) + "\n";
      const url = URL.createObjectURL(new Blob([content], { type: "application/json" }));
      const link = document.createElement("a");
      link.href = url;
      link.download = filename();
      document.body.append(link);
      link.click();
      link.remove();
      // Give the browser time to start reading the download before releasing it.
      setTimeout(() => URL.revokeObjectURL(url), 1000);
      setMessage(`Download started: ${filename()}`);
    } catch {
      setMessage("Could not create the download. Please try again.");
    }
  }

  return (
    <div class="**:focus-visible:outline-2 **:focus-visible:outline-offset-4 **:focus-visible:outline-accent">
      <a href="#builder" class="sr-only focus:not-sr-only focus:absolute focus:bg-paper focus:p-4">Skip to preset builder</a>
      <header class="mx-auto flex max-w-[1240px] items-center justify-between gap-4 border-b border-line px-5 py-6 sm:px-8">
        <a href="/" class="font-heading text-2xl font-bold tracking-tight" aria-label="Settupper home">
          <span class="mr-2 rounded-md bg-accent px-2 py-1 font-mono text-base text-white" aria-hidden="true">&gt;_</span>
          settupper<span class="text-accent">.</span>
        </a>
        <nav aria-label="Main navigation" class="flex gap-5 text-sm">
          <a href="/" class="text-muted hover:text-accent">Home</a>
          <a href="/presets" aria-current="page" class="font-semibold text-accent">Presets</a>
        </nav>
      </header>

      <main id="builder" class="mx-auto max-w-[1240px] px-5 pb-16 sm:px-8">
        <section class="max-w-3xl pt-12 pb-9 sm:pt-16">
          <p class="mb-4 font-mono text-xs tracking-widest text-accent uppercase">Your setup, your selection</p>
          <h1 class="font-heading text-4xl leading-tight font-semibold tracking-tight sm:text-6xl">A fresh start.<br />With your favorite tools.</h1>
          <p class="mt-5 max-w-xl text-base leading-relaxed text-muted">Pick your applications and save a configuration for Settupper. Start with a collection below or build your own from scratch.</p>
        </section>

        <section aria-labelledby="starters-heading" class="mb-10">
          <h2 id="starters-heading" class="mb-3 text-sm font-semibold">Start with a collection</h2>
          <div class="grid gap-3 sm:grid-cols-3">
            <For each={starterPresets}>{preset => (
              <button type="button" onClick={() => { changeSelection([...new Set([...ids(), ...preset.ids])]); setName(preset.name); }}
                class="rounded-xl border border-line bg-white/50 p-5 text-left transition-colors hover:border-accent hover:bg-white">
                <span class="flex items-center justify-between gap-2 font-heading text-lg font-semibold">{preset.name}<span aria-hidden="true" class="text-accent">+</span></span>
                <span class="mt-2 block text-sm leading-relaxed text-muted">{preset.description}</span>
                <span class="mt-3 block font-mono text-xs text-accent">Add collection →</span>
              </button>
            )}</For>
          </div>
        </section>

        <div class="grid items-start gap-7 lg:grid-cols-[minmax(0,1fr)_340px]">
          <section aria-labelledby="catalog-heading" class="min-w-0">
            <div class="mb-5 flex flex-wrap items-baseline justify-between gap-2">
              <h2 id="catalog-heading" class="font-heading text-2xl font-semibold tracking-tight">Application catalog</h2>
              <p role="status" class="text-sm text-muted">{visible().length} of {props.catalog.apps.length} applications</p>
            </div>
            <a href="#your-preset" class="mb-5 inline-block text-sm font-semibold text-accent underline lg:hidden">View your preset ({selected().length}) ↓</a>
            <div class="mb-5 grid gap-3 sm:grid-cols-[1fr_190px]">
              <label class="text-xs font-semibold">Search applications
                <input type="search" value={query()} onInput={event => setQuery(event.currentTarget.value)} placeholder="Name or description…"
                  class="mt-2 w-full rounded-lg border border-line bg-white px-4 py-3 text-sm font-normal" />
              </label>
              <label class="text-xs font-semibold">Category
                <select value={group()} onChange={event => setGroup(event.currentTarget.value)} class="mt-2 w-full rounded-lg border border-line bg-white px-3 py-3 text-sm font-normal">
                  <option value="">All categories</option>
                  <For each={groups}>{group => <option value={group.id}>{group.name}</option>}</For>
                </select>
              </label>
            </div>
            <div class="grid gap-3 sm:grid-cols-2">
              <For each={visible()}>{app => {
                const automatic = () => selectedIds().has(app.id) && !ids().includes(app.id);
                return (
                  <article class={`min-w-0 rounded-xl border p-5 ${selectedIds().has(app.id) ? "border-accent bg-white" : "border-line bg-white/50"}`}>
                    <label class="flex cursor-pointer items-center gap-3 font-heading text-lg font-semibold">
                      <input type="checkbox" checked={selectedIds().has(app.id)} disabled={automatic()}
                        onChange={() => changeSelection(toggleSelection(ids(), app.id))}
                        class="size-5 shrink-0 accent-accent" />
                      {app.name}
                    </label>
                    <p class="mt-3 text-sm leading-relaxed text-muted">{app.description}</p>
                    <p class="mt-4 font-mono text-[11px] text-accent">{groupName(app.group)}</p>
                    <p class="mt-2 text-xs leading-relaxed text-muted">{applicationPlatforms(app)}</p>
                    <Show when={automatic()}><p class="mt-3 text-xs font-semibold">Included as a dependency. Remove the applications that require it to deselect.</p></Show>
                    <details class="mt-4 border-t border-line pt-3 text-xs">
                      <summary class="cursor-pointer text-muted hover:text-ink">Application details &amp; commands</summary>
                      <pre class="mt-3 max-h-64 overflow-auto rounded-md bg-terminal p-3 text-[11px] leading-relaxed whitespace-pre-wrap wrap-anywhere text-paper">{JSON.stringify(app, null, 2)}</pre>
                    </details>
                  </article>
                );
              }}</For>
            </div>
            <Show when={!visible().length}>
              <div class="rounded-xl border border-dashed border-line p-10 text-center">
                <p class="font-semibold">No applications found</p>
                <p class="mt-2 text-sm text-muted">Try another name or category.</p>
                <button type="button" onClick={() => { setQuery(""); setGroup(""); }} class="mt-4 text-sm font-semibold text-accent underline">Reset filters</button>
              </div>
            </Show>
            <p class="mt-5 text-xs leading-relaxed text-muted">Platforms shown have explicit install commands. Availability depends on your system’s package sources. The website only creates a file; Settupper runs the commands on your machine.</p>
          </section>

          <aside id="your-preset" aria-labelledby="selection-heading" class="rounded-xl border border-line bg-white p-6 lg:sticky lg:top-6">
            <div class="flex items-center justify-between gap-2">
              <h2 id="selection-heading" class="font-heading text-xl font-semibold">Your preset <span role="status" class="text-muted">({selected().length})</span></h2>
              <button type="button" disabled={!ids().length} onClick={() => changeSelection([])} class="text-xs text-muted underline hover:text-accent disabled:opacity-40">Clear all</button>
            </div>
            <Show when={selected().length} fallback={<p class="my-7 text-sm leading-relaxed text-muted">Your setup starts here. Select an application or add a starter collection.</p>}>
              <ul class="my-5 divide-y divide-line">
                <For each={selected()}>{app => (
                  <li class="flex items-center justify-between gap-3 py-3 text-sm">
                    <div>
                      <span class="font-semibold">{app.name}</span>
                      <Show when={!ids().includes(app.id)}>
                        <p class="mt-1 text-xs text-muted">Required by {selected().filter(parent => parent.depends_on?.includes(app.id)).map(parent => parent.name).join(", ")}</p>
                      </Show>
                    </div>
                    <Show when={ids().includes(app.id)}>
                      <button type="button" aria-label={`Remove ${app.name}`} onClick={() => changeSelection(ids().filter(id => id !== app.id))} class="shrink-0 px-2 py-1 text-lg text-muted hover:text-accent">×</button>
                    </Show>
                  </li>
                )}</For>
              </ul>
            </Show>
            <form onSubmit={download} class="border-t border-line pt-5">
              <label for="preset-name" class="text-xs font-semibold">Preset name</label>
              <input id="preset-name" value={name()} onInput={event => { setName(event.currentTarget.value); setMessage(""); }} maxLength={100} placeholder="my-preset"
                class="mt-2 w-full rounded-lg border border-line px-3 py-2.5 text-sm" />
              <p class="mt-2 truncate font-mono text-xs text-muted" title={filename()}>{filename()}</p>
              <button type="submit" disabled={!selected().length} class="mt-5 w-full rounded-lg bg-accent px-4 py-3 text-sm font-semibold text-white hover:bg-[#bf3b1d] disabled:cursor-not-allowed disabled:opacity-40">Download JSON ↓</button>
              <p role="status" class="mt-3 text-xs leading-relaxed text-muted">{message()}</p>
            </form>
            <Show when={selected().length}>
              <details class="mt-3 text-xs">
                <summary class="cursor-pointer font-semibold">Preview configuration</summary>
                <pre class="mt-3 max-h-64 overflow-auto rounded-lg bg-terminal p-3 text-[11px] whitespace-pre-wrap wrap-anywhere text-paper">{JSON.stringify(createPreset(props.catalog, ids()), null, 2)}</pre>
              </details>
            </Show>
            <div class="mt-6 rounded-lg bg-paper p-4">
              <p class="text-xs font-semibold">After downloading, run:</p>
              <code class="mt-2 block font-mono text-xs leading-relaxed break-all">settupper {filename()}</code>
              <a href="/#install" class="mt-3 inline-block text-xs text-accent underline">Install Settupper →</a>
            </div>
          </aside>
        </div>
      </main>
      <footer class="border-t border-line px-5 py-6 text-center text-xs text-muted">One configuration file. Your next machine, already set up.</footer>
    </div>
  );
}
