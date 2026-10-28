import { IconCornerDownRight } from "@tabler/icons-solidjs";
import { createSignal, onMount } from "solid-js";
import CopyButton from "./CopyButton";

const platforms = [
  {
    id: "unix",
    label: "Linux / macOS",
    shell: "SHELL",
    command:
      "curl -fsSL https://raw.githubusercontent.com/devbaraus/settupper/main/install.sh | sh",
    path: "~/.local/bin/settupper",
  },
  {
    id: "powershell",
    label: "PowerShell",
    shell: "POWERSHELL",
    command:
      "irm https://raw.githubusercontent.com/devbaraus/settupper/main/install.ps1 | iex",
    path: "%LOCALAPPDATA%\\Programs\\settupper\\bin",
  },
  {
    id: "cmd",
    label: "CMD",
    shell: "CMD",
    command:
      'powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/devbaraus/settupper/main/install.ps1 | iex"',
    path: "%LOCALAPPDATA%\\Programs\\settupper\\bin",
  },
];

export default function InstallTabs() {
  const [active, setActive] = createSignal(0);
  const [ready, setReady] = createSignal(false);
  const tabs: HTMLButtonElement[] = [];
  onMount(() => setReady(true));

  return (
    <div class="min-w-0 rounded-lg border border-[#42493d] bg-terminal text-[#e1e6d9] shadow-lg shadow-[#27321f0a]">
      <div
        hidden={!ready()}
        role="tablist"
        aria-label="Installation platform"
        class="flex gap-4 border-b border-[#454b3e] px-4 sm:gap-5 sm:px-6"
      >
        {platforms.map((platform, index) => (
          <button
            ref={(element) => {
              tabs[index] = element;
            }}
            type="button"
            id={`tab-${platform.id}`}
            role="tab"
            aria-selected={active() === index}
            aria-controls={`panel-${platform.id}`}
            tabIndex={active() === index ? 0 : -1}
            onClick={() => setActive(index)}
            onKeyDown={(event) => {
              let next: number;
              if (event.key === "ArrowRight")
                next = (index + 1) % platforms.length;
              else if (event.key === "ArrowLeft")
                next = (index - 1 + platforms.length) % platforms.length;
              else if (event.key === "Home") next = 0;
              else if (event.key === "End") next = platforms.length - 1;
              else return;
              event.preventDefault();
              setActive(next);
              tabs[next].focus();
            }}
            class="cursor-pointer border-b-2 border-transparent bg-transparent pt-5 pb-4 text-[10px] text-[#a8b49d] aria-selected:border-[#e9b07b] aria-selected:text-[#f4e8ca] focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-[#f2c381] sm:text-xs"
          >
            {platform.label}
          </button>
        ))}
      </div>
      {platforms.map((platform, index) => (
        <div
          id={`panel-${platform.id}`}
          role="tabpanel"
          aria-label={ready() ? undefined : platform.label}
          aria-labelledby={ready() ? `tab-${platform.id}` : undefined}
          tabIndex={0}
          hidden={ready() && active() !== index}
          class="min-h-[185px] p-5 sm:p-6"
        >
          <div class="mb-4 flex items-center justify-between font-mono text-[9px] tracking-wider text-[#a8b49d]">
            <span>{platform.shell}</span>
            <CopyButton
              text={platform.command}
              label={`Copy ${platform.id} installation command`}
            />
          </div>
          <pre class="font-mono text-[11px] leading-[1.9] whitespace-pre-wrap text-[#efcd9b] wrap-anywhere sm:text-[13px]">
            <code>{platform.command}</code>
          </pre>
          <p class="mt-5 text-[10px] text-[#a8b49d] wrap-anywhere">
            Installs to{" "}
            <code class="font-mono text-[#c8d1bc]">{platform.path}</code>
          </p>
        </div>
      ))}
      <div class="border-t border-[#454b3e] px-5 py-3.5 text-[10px] leading-relaxed text-[#bcc4b0] sm:px-6">
        <IconCornerDownRight
          size={14}
          class="mr-1 inline-block align-middle text-[#e8ba7b]"
          aria-hidden="true"
        />{" "}
        Linux &amp; macOS: make sure <code class="font-mono">~/.local/bin</code>{" "}
        is in your PATH.
      </div>
    </div>
  );
}
