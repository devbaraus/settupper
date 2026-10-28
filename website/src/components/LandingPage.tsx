import {
  IconArrowDown,
  IconArrowUpRight,
  IconBraces,
  IconCheck,
  IconCommand,
  IconCornerDownRight,
  IconEye,
  IconFileCode,
  IconHierarchy2,
  IconKeyboard,
  IconPlus,
  IconRefresh,
  IconTerminal2,
} from "@tabler/icons-solidjs";
import CopyButton from "./CopyButton";
import InstallTabs from "./InstallTabs";

const github = "https://github.com/devbaraus/settupper";
const example = `version: 1

apps:
  - id: git
    name: Git
    check:
      default:
        - command -v git
    actions:
      install:
        ubuntu:
          - sudo apt-get install -y git
        fedora:
          - sudo dnf install -y git
        arch:
          - sudo pacman -S --noconfirm git
        macos:
          - brew install git`;
const features = [
  {
    number: "01",
    icon: IconTerminal2,
    title: "A terminal that makes sense.",
    text: "Your apps, their status, and real-time logs in one view. Navigate with your keyboard and select exactly what needs attention.",
    large: true,
  },
  {
    number: "02",
    icon: IconHierarchy2,
    title: "Let Smart All do the sorting.",
    text: "Install missing apps and update installed ones. Dependencies run in the right order, so Git is ready before NVM.",
    large: true,
  },
  {
    number: "03",
    icon: IconBraces,
    title: "Your setup is a file.",
    text: "Keep your tools in YAML or JSON. Version it, share it with your team, and bring it to your next machine.",
  },
  {
    number: "04",
    icon: IconEye,
    title: "See it before you run it.",
    text: "Preview planned actions with --dry-run. Export a JSON snapshot whenever you need a record of your setup.",
  },
  {
    number: "05",
    icon: IconRefresh,
    title: "The little things, handled.",
    text: "Group your apps, work on several at once, and get a notice when a reboot or a Settupper update is needed.",
  },
];
const keys = [
  ["↑ / ↓", "Navigate"],
  ["Space", "Select apps"],
  ["i", "Install"],
  ["u", "Update"],
  ["d", "Uninstall"],
  ["a", "Smart action"],
  ["Shift + A", "Smart All"],
  ["Tab", "Switch group"],
  ["r", "Refresh"],
  ["e", "Export JSON"],
  ["Esc", "Clear selection"],
  ["q", "Quit"],
];

export default function LandingPage() {
  return (
    <div class="[&_p]:leading-[1.7] [&_h1]:font-heading [&_h1]:font-semibold [&_h1]:leading-[1.08] [&_h1]:tracking-[-0.065em] [&_h2]:font-heading [&_h2]:text-[clamp(2.3rem,4vw,3.6rem)] [&_h2]:font-semibold [&_h2]:leading-[1.08] [&_h2]:tracking-[-0.065em] [&_h3]:font-heading [&_h3]:font-semibold [&_h3]:leading-[1.3] [&_h3]:tracking-[-0.035em] [&_kbd]:inline-flex [&_kbd]:items-center [&_kbd]:justify-center [&_kbd]:rounded-sm [&_kbd]:border [&_kbd]:border-b-2 [&_kbd]:border-[#cdd0c2] [&_kbd]:bg-[#fbfaf5] [&_kbd]:px-1.5 [&_kbd]:py-0.5 [&_kbd]:font-mono [&_kbd]:text-[0.75em] [&_kbd]:leading-normal [&_kbd]:whitespace-nowrap [&_kbd]:text-[#535b49] **:focus-visible:outline-2 **:focus-visible:outline-offset-4 **:focus-visible:outline-accent">
      <a
        class="sr-only focus:not-sr-only focus:fixed focus:top-5 focus:left-5 focus:z-50 focus:rounded-md focus:bg-ink focus:px-5 focus:py-3.5 focus:text-white"
        href="#main"
      >
        Skip to content
      </a>
      <header class="flex h-[75px] items-center justify-between border-b border-line md:h-[85px] lg:h-[104px] mx-auto w-[calc(100%-36px)] max-w-[1240px] sm:w-[calc(100%-64px)] xl:w-[calc(100%-96px)]">
        <a
          class="inline-flex items-center gap-2.5 font-heading text-xl font-bold tracking-[-0.07em] sm:text-2xl"
          href="#"
          aria-label="Settupper home"
        >
          <span
            class="grid size-7 place-content-center rounded-[7px] bg-accent font-mono text-base font-semibold tracking-[-0.14em] text-[#fffaf4] sm:size-8"
            aria-hidden="true"
          >
            <IconTerminal2 size={20} stroke="1.8" aria-hidden="true" />
          </span>
          settupper<span class="-ml-[9px] text-accent">.</span>
        </a>
        <nav
          class="flex gap-4 text-xs sm:gap-7 [&_a]:hover:text-accent max-sm:[&>a:last-child]:hidden"
          aria-label="Main navigation"
        >
          <a href="#features">Features</a>
          <a href="#install">Install</a>
          <a href="#usage">How it works</a>
        </nav>
        <a
          class="flex items-center gap-2 rounded-md py-2 text-xs font-semibold hover:bg-[#eae9e1] sm:gap-6 sm:border sm:border-[#bfc0b5] sm:px-4 sm:py-2.5"
          href={github}
        >
          GitHub{" "}
          <IconArrowUpRight
            size={14}
            class="inline-block shrink-0 align-middle"
            aria-hidden="true"
          />
        </a>
      </header>

      <main id="main">
        <section class="grid items-center gap-10 py-12 sm:py-16 lg:grid-cols-[1.04fr_1fr] lg:gap-9 lg:py-24 [&_h1]:my-6 [&_h1]:text-[clamp(2.55rem,7.8vw,4.8rem)] lg:[&_h1]:text-[clamp(2.8rem,4.4vw,4.3rem)] [&_h1>span]:text-accent mx-auto w-[calc(100%-36px)] max-w-[1240px] sm:w-[calc(100%-64px)] xl:w-[calc(100%-96px)]">
          <div class="min-w-0">
            <a
              class="font-mono text-[9px] font-medium leading-[1.8] tracking-[0.095em] sm:text-[10px] flex w-fit items-center gap-2 text-[8px] tracking-[0.065em] sm:text-[9px] [&>span:last-child]:ml-1 [&>span:last-child]:text-accent"
              href={`${github}/releases`}
            >
              <span class="mr-1 size-1.5 shrink-0 rounded-full bg-accent ring-4 ring-accent/5"></span>{" "}
              OPEN SOURCE. READY FOR YOUR TERMINAL.{" "}
              <IconArrowUpRight
                size={14}
                class="inline-block shrink-0 align-middle"
                aria-hidden="true"
              />
            </a>
            <h1>
              Your next machine.
              <br />
              <span>Already set up.</span>
            </h1>
            <p class="max-w-[435px] text-sm leading-[1.8] text-muted sm:text-base">
              All your tools. One config file. Install, update, and manage your
              setup from a terminal that feels like home.
            </p>
            <div class="mt-7 flex flex-wrap items-center gap-5 lg:gap-7">
              <a
                class="inline-flex items-center justify-center gap-6 rounded-md px-5 py-4 text-xs font-semibold transition duration-200 hover:-translate-y-0.5 motion-reduce:transform-none motion-reduce:transition-none sm:text-sm bg-accent text-white hover:bg-[#c63e1f]"
                href="#install"
              >
                Get Settupper{" "}
                <IconArrowDown
                  size={14}
                  class="inline-block shrink-0 align-middle"
                  aria-hidden="true"
                />
              </a>
              <a
                class="inline-flex items-center gap-3 text-xs font-semibold hover:underline hover:underline-offset-4 sm:text-[13px]"
                href="#usage"
              >
                See how it works{" "}
                <IconArrowUpRight
                  size={14}
                  class="inline-block shrink-0 align-middle"
                  aria-hidden="true"
                />
              </a>
            </div>
            <p class="mt-5 flex items-center gap-2 text-[10px] text-muted sm:text-[11px] [&>span:first-child]:text-accent">
              <IconCheck
                size={14}
                class="inline-block shrink-0 align-middle"
                aria-hidden="true"
              />{" "}
              Linux, macOS &amp; Windows{" "}
              <span class="mx-1.5 text-[#b6b7aa]">/</span> Built with Rust
            </p>
          </div>

          <figure class="relative isolate w-full max-w-[580px] min-w-0 justify-self-center pt-3 before:absolute before:top-[9%] before:left-[5%] before:-z-10 before:h-[85%] before:w-[92%] before:rotate-3 before:rounded-xl before:border before:border-[#dfdccc] before:bg-[#e9e6db] [&_figcaption]:mt-5 [&_figcaption]:flex [&_figcaption]:items-center [&_figcaption]:gap-2 [&_figcaption]:font-mono [&_figcaption]:text-[7px] [&_figcaption]:tracking-wide [&_figcaption]:text-[#767c6c] [&_figcaption>span:last-child]:ml-auto [&_figcaption>span:last-child]:text-[6px] sm:[&_figcaption]:text-[8px]">
            <div
              class="overflow-hidden rounded-[9px] border border-[#364137] bg-terminal font-mono text-[11px] text-[#d9e0d7] shadow-[0_20px_40px_#1f271518]"
              aria-label="Illustrative Settupper terminal preview"
            >
              <div class="flex items-center justify-between border-b border-[#41483e] bg-[#2b322b] p-3 text-[8px] text-[#a3b09f] sm:px-4 sm:py-3.5 sm:text-[9px]">
                <span
                  class="flex gap-[5px] [&_i]:size-[7px] [&_i]:rounded-full [&_i]:bg-[#ef876e] [&_i:nth-child(2)]:bg-[#e6c778] [&_i:nth-child(3)]:bg-[#9bb88d]"
                  aria-hidden="true"
                >
                  <i></i>
                  <i></i>
                  <i></i>
                </span>
                <span>settupper — packages.yaml</span>
                <IconCommand
                  size={14}
                  class="inline-block shrink-0 align-middle"
                  aria-hidden="true"
                />
              </div>
              <div class="px-3 pt-4 sm:px-4 sm:pt-[18px]">
                <div class="flex items-center gap-2.5 text-[9px] [&_strong]:text-[11px] [&_strong]:tracking-widest [&_strong]:text-[#f3efdd] [&>span:nth-child(2)]:rounded-sm [&>span:nth-child(2)]:border [&>span:nth-child(2)]:border-[#4b5749] [&>span:nth-child(2)]:px-[5px] [&>span:nth-child(2)]:py-0.5 [&>span:nth-child(2)]:text-[#a3b09f]">
                  <strong>SETTUPPER</strong>
                  <span>fedora</span>
                  <span class="ml-auto text-[#a9c58e]">● ready</span>
                </div>
                <div class="my-[18px] text-[8px] text-[#93a68d] sm:text-[9px] [&>span:first-child]:text-[#e7c98f] [&>span:last-child]:ml-2.5">
                  Group: <span>[All]</span>{" "}
                  <span class="text-[#8a9886]">tab to switch</span>
                </div>
                <div class="grid min-h-[220px] grid-cols-[1.4fr_1fr] sm:min-h-[240px]">
                  <div class="min-w-0 border-r border-[#414c40] pr-2.5 sm:pr-3 [&>div:not(:first-child):not(:last-child)]:my-[3px] [&>div:not(:first-child):not(:last-child)]:-mx-1 [&>div:not(:first-child):not(:last-child)]:grid [&>div:not(:first-child):not(:last-child)]:grid-cols-[12px_1fr_auto] [&>div:not(:first-child):not(:last-child)]:items-center [&>div:not(:first-child):not(:last-child)]:gap-1 [&>div:not(:first-child):not(:last-child)]:px-1 [&>div:not(:first-child):not(:last-child)]:py-2.5 [&>div:not(:first-child):not(:last-child)]:text-[8px] sm:[&>div:not(:first-child):not(:last-child)]:text-[10px] [&>div>span:last-child]:text-[7px] sm:[&>div>span:last-child]:text-[9px]">
                    <div class="mb-3.5 flex justify-between text-[7px] tracking-widest text-[#a0ac98] sm:text-[8px]">
                      YOUR APPS <span>STATUS</span>
                    </div>
                    <div>
                      <span class="text-[#b6cd94]">✓</span>
                      <span>Git</span>
                      <span class="text-[#b6cd94]">installed</span>
                    </div>
                    <div>
                      <span class="text-[#b6cd94]">✓</span>
                      <span>Fish Shell</span>
                      <span class="text-[#b6cd94]">installed</span>
                    </div>
                    <div class="border-l-2 border-[#edbd72] bg-[#374335] text-[#fcf4df]">
                      <span>›</span>
                      <span>VS Code</span>
                      <span class="text-[#e9bd78]">missing</span>
                    </div>
                    <div>
                      <span class="text-[#b6cd94]">✓</span>
                      <span>Node.js</span>
                      <span class="text-[#b6cd94]">installed</span>
                    </div>
                    <div>
                      <span class="text-[#8a9886]">○</span>
                      <span>NVM</span>
                      <span class="text-[#e9bd78]">missing</span>
                    </div>
                    <div class="flex gap-4 py-4 text-[7px] sm:text-[8px]">
                      <span class="text-[#b6cd94]">3 installed</span>
                      <span>2 to go</span>
                    </div>
                  </div>
                  <div class="pl-3 text-[8px] sm:pl-4 [&>strong]:text-xs [&>strong]:font-medium [&>strong]:text-[#f5e8ce] [&_p]:mt-2 [&_p]:text-[7px] sm:[&_p]:text-[9px] [&_dl]:mt-5 [&_dl]:grid [&_dl]:grid-cols-2 [&_dl]:gap-y-2.5 [&_dl]:text-[7px] sm:[&_dl]:text-[9px] [&_dt]:text-[#8a9886]">
                    <div class="mb-3.5 flex justify-between text-[7px] tracking-widest text-[#a0ac98] sm:text-[8px]">
                      DETAILS
                    </div>
                    <strong>VS Code</strong>
                    <p>Editor, ready for work.</p>
                    <dl>
                      <dt>Group</dt>
                      <dd>editors</dd>
                      <dt>Action</dt>
                      <dd class="text-[#e9bd78]">install</dd>
                    </dl>
                    <div class="mb-3.5 flex justify-between text-[7px] tracking-widest text-[#a0ac98] sm:text-[8px] mt-5 mb-1 border-t border-[#414c40] pt-3.5">
                      LOG
                    </div>
                    <p class="text-[#b6cd94]">✓ Status updated</p>
                    <p class="text-[#8a9886]">
                      Waiting for you<span class="text-[#e9bd78]">_</span>
                    </p>
                  </div>
                </div>
                <div class="mt-3 flex justify-between gap-1.5 border-t border-[#414c40] py-3 text-[7px] text-[#9dab96] sm:text-[8px] [&_b]:font-normal [&_b]:text-[#f4d59f]">
                  <span>
                    <b>i</b> install
                  </span>
                  <span>
                    <b>u</b> update
                  </span>
                  <span>
                    <b>A</b> smart all
                  </span>
                  <span>
                    <b>q</b> quit
                  </span>
                </div>
              </div>
            </div>
            <figcaption>
              <span class="text-base text-accent" aria-hidden="true">
                <IconPlus size={14} aria-hidden="true" />
              </span>{" "}
              YOUR SETUP, AT A GLANCE. <span>ILLUSTRATIVE PREVIEW</span>
            </figcaption>
          </figure>
        </section>

        <div class="border-y border-line bg-[#efeee6]">
          <div class="mx-auto w-[calc(100%-36px)] max-w-[1240px] sm:w-[calc(100%-64px)] xl:w-[calc(100%-96px)] flex flex-col items-center justify-between gap-4 py-6 lg:flex-row lg:gap-7 [&>.font-mono]:whitespace-nowrap [&>.font-mono]:text-[9px] [&>.font-mono]:text-[#787b6f] [&>div]:flex [&>div]:w-full [&>div]:flex-1 [&>div]:flex-wrap [&>div]:items-center [&>div]:justify-center [&>div]:gap-x-6 [&>div]:gap-y-3 [&>div]:font-heading [&>div]:text-xs [&>div]:font-medium [&>div]:text-[#5f6459] sm:[&>div]:justify-between sm:[&>div]:text-sm">
            <span class="font-mono text-[9px] font-medium leading-[1.8] tracking-[0.095em] sm:text-[10px]">
              SAME CONFIG. DIFFERENT HOMES.
            </span>
            <div>
              <span>Ubuntu / Debian</span>
              <span>Fedora / RHEL</span>
              <span>Arch Linux</span>
              <span>macOS</span>
              <span>Windows</span>
            </div>
          </div>
        </div>

        <section
          class="py-14 sm:py-16 lg:py-24 mx-auto w-[calc(100%-36px)] max-w-[1240px] sm:w-[calc(100%-64px)] xl:w-[calc(100%-96px)]"
          id="features"
        >
          <div class="mb-8 flex flex-col items-start justify-between gap-5 lg:mb-11 lg:flex-row lg:items-end lg:gap-11 [&_h2]:mt-4 [&>p]:max-w-[480px] [&>p]:pb-1 [&>p]:text-sm [&>p]:text-muted lg:[&>p]:max-w-[325px] lg:[&>p]:text-[15px]">
            <div>
              <p class="font-mono text-[9px] font-medium leading-[1.8] tracking-[0.095em] sm:text-[10px] text-accent">
                01 / LESS SETUP. MORE DOING.
              </p>
              <h2>
                A fresh start.
                <br />
                Without starting over.
              </h2>
            </div>
            <p>
              New laptop? Clean install? A whole team to onboard? Keep the setup
              you love, wherever you go.
            </p>
          </div>
          <div class="grid gap-3 sm:grid-cols-2 sm:gap-4 lg:grid-cols-6">
            {features.map((feature) => (
              <article
                class="rounded-md border border-line bg-[#f9f8f3] p-6 max-sm:col-span-1 max-lg:last:col-span-full [&_h3]:text-xl [&_p]:mt-3 [&_p]:max-w-[455px] [&_p]:text-sm [&_p]:text-muted"
                classList={{
                  "lg:col-span-3": feature.large,
                  "lg:col-span-2": !feature.large,
                }}
              >
                <div class="mb-5 flex items-center justify-between sm:mb-7">
                  <span
                    class="h-9 font-mono text-2xl font-medium text-accent"
                    aria-hidden="true"
                  >
                    <feature.icon size={28} stroke="1.7" aria-hidden="true" />
                  </span>
                  <span class="font-mono text-[10px] text-[#919483]">
                    /{feature.number}
                  </span>
                </div>
                <h3>{feature.title}</h3>
                <p>{feature.text}</p>
              </article>
            ))}
          </div>
        </section>

        <section
          class="border-y border-line bg-[#eeeee6] py-16 lg:py-20"
          id="install"
        >
          <div class="mx-auto w-[calc(100%-36px)] max-w-[1240px] sm:w-[calc(100%-64px)] xl:w-[calc(100%-96px)] grid items-center gap-8 lg:grid-cols-[0.9fr_1.25fr] lg:gap-16 [&_h2]:mt-4 [&>div:first-child>p:not(:first-child)]:mt-6 [&>div:first-child>p:not(:first-child)]:mb-5 [&>div:first-child>p:not(:first-child)]:max-w-[500px] [&>div:first-child>p:not(:first-child)]:text-sm [&>div:first-child>p:not(:first-child)]:text-muted lg:[&>div:first-child>p:not(:first-child)]:max-w-[360px]">
            <div>
              <p class="font-mono text-[9px] font-medium leading-[1.8] tracking-[0.095em] sm:text-[10px] text-accent">
                02 / MAKE YOURSELF AT HOME.
              </p>
              <h2>
                One command.
                <br />
                Then you’re in.
              </h2>
              <p>
                Pick your platform, open a terminal, and install. The script
                finds the latest release and the right binary for your machine.
              </p>
              <a
                class="inline-flex items-center gap-3 text-xs font-semibold hover:underline hover:underline-offset-4 sm:text-[13px]"
                href={`${github}/releases`}
              >
                Prefer a manual download?{" "}
                <IconArrowUpRight
                  size={14}
                  class="inline-block shrink-0 align-middle"
                  aria-hidden="true"
                />
              </a>
            </div>
            <InstallTabs />
          </div>
        </section>

        <section
          class="py-14 sm:py-16 lg:py-24 mx-auto w-[calc(100%-36px)] max-w-[1240px] sm:w-[calc(100%-64px)] xl:w-[calc(100%-96px)]"
          id="usage"
        >
          <div class="mb-8 flex flex-col items-start justify-between gap-5 lg:mb-11 lg:flex-row lg:items-end lg:gap-11 [&_h2]:mt-4 [&>p]:max-w-[480px] [&>p]:pb-1 [&>p]:text-sm [&>p]:text-muted lg:[&>p]:max-w-[325px] lg:[&>p]:text-[15px]">
            <div>
              <p class="font-mono text-[9px] font-medium leading-[1.8] tracking-[0.095em] sm:text-[10px] text-accent">
                03 / YOUR SETUP, IN PLAIN TEXT.
              </p>
              <h2>
                Declare it.
                <br />
                Run it. Make it yours.
              </h2>
            </div>
            <p>
              You choose the tools and commands. Settupper brings them together
              in one keyboard-friendly place.
            </p>
          </div>
          <div class="grid items-start gap-9 lg:grid-cols-2 lg:gap-16">
            <ol class="[&>li]:flex [&>li]:gap-4 [&>li]:border-b [&>li]:border-line [&>li]:py-6 [&>li:first-child]:pt-1 [&>li:last-child]:border-0 [&>li:last-child]:pb-0 [&_h3]:text-xl [&_p]:mt-3 [&_p]:text-sm [&_p]:text-muted [&_a]:mt-4 [&_a]:text-accent [&_code]:font-mono [&_code]:text-[0.87em]">
              <li>
                <span class="mt-0.5 grid size-7 shrink-0 place-content-center rounded-full border border-[#d9dacc] font-mono text-[11px] text-accent">
                  1
                </span>
                <div>
                  <h3>Write your wishlist.</h3>
                  <p>
                    Create a <code>packages.yaml</code> or JSON file. Add your
                    apps, an install check, and commands for each platform you
                    use.
                  </p>
                  <a
                    class="inline-flex items-center gap-3 text-xs font-semibold hover:underline hover:underline-offset-4 sm:text-[13px]"
                    href="/packages-example.yaml"
                    download="packages-example.yaml"
                  >
                    Download a starter config{" "}
                    <IconArrowDown
                      size={14}
                      class="inline-block shrink-0 align-middle"
                      aria-hidden="true"
                    />
                  </a>
                </div>
              </li>
              <li>
                <span class="mt-0.5 grid size-7 shrink-0 place-content-center rounded-full border border-[#d9dacc] font-mono text-[11px] text-accent">
                  2
                </span>
                <div>
                  <h3>Open your setup.</h3>
                  <p>
                    Point Settupper at your file. It detects your distro and
                    checks what’s already installed.
                  </p>
                  <div class="mt-4 flex items-center justify-between gap-2 rounded-sm bg-[#eaece1] p-3 text-[11px] leading-relaxed sm:text-[13px] [&_code]:min-w-0 [&_code]:wrap-anywhere [&_button]:border-[#c7cbbb] [&_button]:text-[#5e6754] [&_button]:hover:bg-[#d7ddcb] [&_button]:hover:text-ink">
                    <code>settupper packages.yaml</code>
                    <CopyButton
                      text="settupper packages.yaml"
                      label="Copy launch command"
                    />
                  </div>
                </div>
              </li>
              <li>
                <span class="mt-0.5 grid size-7 shrink-0 place-content-center rounded-full border border-[#d9dacc] font-mono text-[11px] text-accent">
                  3
                </span>
                <div>
                  <h3>You’re in control.</h3>
                  <p>
                    Select apps with <kbd>Space</kbd>. Press <kbd>a</kbd> for a
                    smart action, or <kbd>Shift</kbd> + <kbd>A</kbd> to work
                    through all visible apps in dependency order.
                  </p>
                  <p class="text-xs!">Try a preview first:</p>
                  <div class="mt-4 flex items-center justify-between gap-2 rounded-sm bg-[#eaece1] p-3 text-[11px] leading-relaxed sm:text-[13px] [&_code]:min-w-0 [&_code]:wrap-anywhere [&_button]:border-[#c7cbbb] [&_button]:text-[#5e6754] [&_button]:hover:bg-[#d7ddcb] [&_button]:hover:text-ink">
                    <code>settupper --dry-run packages.yaml</code>
                    <CopyButton
                      text="settupper --dry-run packages.yaml"
                      label="Copy dry run command"
                    />
                  </div>
                </div>
              </li>
            </ol>
            <div class="min-w-0 overflow-hidden rounded-md border border-[#42493d] bg-terminal text-[#d9e0ce]">
              <div class="flex items-center justify-between border-b border-[#454b3e] px-5 py-4 font-mono text-[11px]">
                <span>
                  <span class="text-accent" aria-hidden="true">
                    <IconFileCode
                      size={14}
                      class="inline-block align-middle"
                      aria-hidden="true"
                    />
                  </span>{" "}
                  packages.yaml
                </span>
                <CopyButton text={example} label="Copy example YAML" />
              </div>
              <pre class="overflow-x-auto p-5 font-mono text-[11px] leading-[1.85] text-[#efcd9b] sm:p-6">
                <code>{example}</code>
              </pre>
              <div class="border-t border-[#454b3e] px-5 py-4 text-[10px] leading-relaxed text-[#b5bda6] [&_span]:mr-2 [&_span]:text-[#e8ba7b]">
                <IconCornerDownRight
                  size={14}
                  class="inline-block shrink-0 align-middle"
                  aria-hidden="true"
                />{" "}
                One app. Platform-specific commands. Your rules.
              </div>
            </div>
          </div>
          <div class="mt-8 flex items-center gap-4 rounded-md border border-dashed border-[#c9cebc] p-5 text-muted sm:mt-11 sm:gap-5 sm:px-7 [&>span]:text-3xl [&>span]:text-[#778861] [&_p]:text-xs [&_p]:wrap-anywhere [&_strong]:mr-1 [&_strong]:font-semibold [&_strong]:text-[#414a36] [&_code]:font-mono [&_code]:text-[0.87em]">
            <IconKeyboard
              size={30}
              class="shrink-0 text-[#778861]"
              aria-hidden="true"
            />
            <p>
              <strong>Even fewer keystrokes.</strong> Save your file at{" "}
              <code>~/.config/settupper/packages.yaml</code> on Linux, then just
              run <code>settupper</code>. You can also load a config directly
              from a URL.
            </p>
          </div>
        </section>

        <section class="grid gap-7 border-t border-line py-14 lg:grid-cols-[0.7fr_1.3fr] lg:gap-14 lg:pt-16 lg:pb-20 mx-auto w-[calc(100%-36px)] max-w-[1240px] sm:w-[calc(100%-64px)] xl:w-[calc(100%-96px)]">
          <div class="[&_h2]:mt-4 [&_h2]:text-[2.1rem] sm:[&_h2]:text-[2.7rem] [&>p:not(:first-child)]:mt-5 [&>p:not(:first-child)]:text-sm [&>p:not(:first-child)]:text-muted">
            <p class="font-mono text-[9px] font-medium leading-[1.8] tracking-[0.095em] sm:text-[10px] text-accent">
              STAY IN YOUR FLOW.
            </p>
            <h2>
              Hands on
              <br />
              the keyboard.
            </h2>
            <p>
              No clicking through menus.
              <br />
              Just the keys you need.
            </p>
          </div>
          <dl class="mt-2 grid grid-cols-2 gap-x-4 sm:grid-cols-3 sm:gap-x-6 [&>div]:flex [&>div]:items-center [&>div]:gap-3 [&>div]:border-b [&>div]:border-line [&>div]:py-4 [&_dt]:flex [&_kbd]:min-w-7 [&_kbd]:px-1.5 [&_kbd]:py-1 [&_kbd]:text-[10px] [&_dd]:text-xs [&_dd]:text-muted">
            {keys.map(([key, action]) => (
              <div>
                <dt>
                  <kbd>{key}</kbd>
                </dt>
                <dd>{action}</dd>
              </div>
            ))}
          </dl>
        </section>

        <section class="relative mb-10 flex items-center justify-between overflow-hidden rounded-lg bg-[#e95d37] px-6 py-9 text-[#231f17] sm:px-10 sm:py-12 lg:mb-16 lg:px-16 lg:py-14 [&_h2]:mt-5 [&_h2]:text-[clamp(2.6rem,4.5vw,4rem)] [&>div]:relative [&>div]:z-10 [&>div>p]:text-[9px] mx-auto w-[calc(100%-36px)] max-w-[1240px] sm:w-[calc(100%-64px)] xl:w-[calc(100%-96px)]">
          <div>
            <p class="font-mono text-[9px] font-medium leading-[1.8] tracking-[0.095em] sm:text-[10px]">
              YOUR NEXT FRESH START STARTS HERE.
            </p>
            <h2>
              Bring your setup
              <br />
              with you.
            </h2>
            <div class="mt-7 flex flex-wrap items-center gap-5 lg:gap-7">
              <a
                href="#install"
                class="inline-flex items-center justify-center gap-6 rounded-md px-5 py-4 text-xs font-semibold transition duration-200 hover:-translate-y-0.5 motion-reduce:transform-none motion-reduce:transition-none sm:text-sm bg-[#25291f] text-[#f9f3df] hover:bg-[#3c422e]"
              >
                Install Settupper{" "}
                <IconArrowUpRight
                  size={14}
                  class="inline-block shrink-0 align-middle"
                  aria-hidden="true"
                />
              </a>
              <a
                href={github}
                class="inline-flex items-center gap-3 text-xs font-semibold hover:underline hover:underline-offset-4 sm:text-[13px]"
              >
                Explore the source{" "}
                <IconArrowUpRight
                  size={14}
                  class="inline-block shrink-0 align-middle"
                  aria-hidden="true"
                />
              </a>
            </div>
          </div>
          <IconTerminal2
            class="pointer-events-none absolute top-20 -right-2 size-24 shrink-0 text-[#6d321f] opacity-15 sm:static sm:size-40 sm:opacity-35 lg:size-56"
            stroke="1.2"
            aria-hidden="true"
          />
        </section>
      </main>

      <footer class="flex flex-wrap items-center justify-between gap-5 pb-7 lg:flex-nowrap lg:gap-7 lg:pb-10 [&>p]:ml-auto [&>p]:text-[10px] [&>p]:text-muted lg:[&>p]:ml-0 [&>div]:flex [&>div]:w-full [&>div]:justify-between [&>div]:gap-3 [&>div]:border-t [&>div]:border-line [&>div]:pt-5 [&>div]:text-[11px] [&>div]:text-muted lg:[&>div]:w-auto lg:[&>div]:gap-6 lg:[&>div]:border-0 lg:[&>div]:pt-0 [&_a]:hover:text-accent mx-auto w-[calc(100%-36px)] max-w-[1240px] sm:w-[calc(100%-64px)] xl:w-[calc(100%-96px)]">
        <a
          class="inline-flex items-center gap-2.5 font-heading text-xl font-bold tracking-[-0.07em] sm:text-2xl"
          href="#"
        >
          <span
            class="grid size-7 place-content-center rounded-[7px] bg-accent font-mono text-base font-semibold tracking-[-0.14em] text-[#fffaf4] sm:size-8"
            aria-hidden="true"
          >
            <IconTerminal2 size={20} stroke="1.8" aria-hidden="true" />
          </span>
          settupper<span class="-ml-[9px] text-accent">.</span>
        </a>
        <p>Made for fresh starts. Built with Rust.</p>
        <div>
          <a href={`${github}/blob/main/LICENSE`}>MIT License</a>
          <a
            class="inline-flex items-center gap-1"
            href={`${github}/blob/main/README.md`}
          >
            Documentation <IconArrowUpRight size={13} aria-hidden="true" />
          </a>
          <a class="inline-flex items-center gap-1" href={github}>
            GitHub <IconArrowUpRight size={13} aria-hidden="true" />
          </a>
        </div>
      </footer>
    </div>
  );
}
