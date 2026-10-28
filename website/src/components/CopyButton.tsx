import { createSignal, onCleanup, onMount } from "solid-js";
import { IconCheck, IconCopy } from "@tabler/icons-solidjs";

export default function CopyButton(props: { text: string; label: string }) {
  const [ready, setReady] = createSignal(false);
  const [feedback, setFeedback] = createSignal("");
  const [announcement, setAnnouncement] = createSignal("");
  let timer: ReturnType<typeof setTimeout> | undefined;

  onMount(() => setReady(true));
  onCleanup(() => clearTimeout(timer));

  async function copy() {
    try {
      await navigator.clipboard.writeText(props.text);
      setFeedback("Copied");
      setAnnouncement("Copied to clipboard.");
    } catch {
      setFeedback("Select text to copy");
      setAnnouncement(
        "Clipboard unavailable. Select and copy the command manually.",
      );
    }
    timer = setTimeout(() => setFeedback(""), 2200);
  }

  return (
    <>
      <button
        type="button"
        hidden={!ready()}
        disabled={Boolean(feedback())}
        aria-label={props.label}
        onClick={copy}
        class="inline-flex shrink-0 cursor-pointer items-center gap-1.5 rounded border border-[#4a5244] bg-transparent px-2 py-1.5 font-mono text-[10px] whitespace-nowrap text-[#b7bdae] hover:bg-[#3a4333] hover:text-[#fff8dc] focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-[#f2c381] disabled:cursor-default"
      >
        <span>{feedback() || "Copy"}</span>
        {!feedback() && <IconCopy size={12} aria-hidden="true" />}
        {feedback() === "Copied" && <IconCheck size={12} aria-hidden="true" />}
      </button>
      <span class="sr-only" role="status" aria-live="polite">
        {announcement()}
      </span>
    </>
  );
}
