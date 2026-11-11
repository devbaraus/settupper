export type Commands = string | string[] | Record<string, string | string[]>;
export type Application = {
  id: string;
  name: string;
  description?: string;
  group?: string;
  depends_on?: string[];
  check?: Commands;
  install?: Commands;
  update?: Commands;
  uninstall?: Commands;
  actions?: { install?: Commands; update?: Commands; uninstall?: Commands };
  reboot_on?: Record<string, boolean>;
};
export type Catalog = {
  version: number;
  groups: { id: string; name: string }[];
  apps: Application[];
};

export const starterPresets = [
  { name: "Developer", description: "Git, npm with Node.js, and Python.", ids: ["git", "npm", "python"] },
  { name: "Terminal Essentials", description: "Git and curl for the command line.", ids: ["git", "curl"] },
  { name: "Everyday Apps", description: "Firefox for browsing. VLC for media.", ids: ["firefox", "vlc"] },
];

export function validateCatalog(catalog: Catalog) {
  if (!Number.isInteger(catalog.version) || catalog.version < 1 || !Array.isArray(catalog.apps) || !Array.isArray(catalog.groups)) {
    throw new Error("Invalid catalog structure");
  }
  const ids = new Set<string>();
  function validateCommands(value: unknown): boolean {
    if (typeof value === "string") return value.trim().length > 0;
    if (Array.isArray(value)) return value.length > 0 && value.every(command => typeof command === "string" && command.trim());
    return !!value && typeof value === "object" && Object.values(value).every(commands =>
      typeof commands === "string" ? commands.trim().length > 0 : Array.isArray(commands) && validateCommands(commands));
  }
  for (const app of catalog.apps) {
    if (typeof app.id !== "string" || !app.id.trim() || typeof app.name !== "string" || !app.name.trim() || ids.has(app.id)) {
      throw new Error(`Missing or duplicate application ID/name: ${app.id}`);
    }
    if (app.depends_on !== undefined && (!Array.isArray(app.depends_on) || app.depends_on.some(id => typeof id !== "string"))) {
      throw new Error(`Invalid dependencies: ${app.id}`);
    }
    if ((app.description !== undefined && typeof app.description !== "string") ||
        (app.group !== undefined && typeof app.group !== "string") ||
        (app.actions !== undefined && (!app.actions || typeof app.actions !== "object" || Array.isArray(app.actions))) ||
        (app.reboot_on !== undefined && (!app.reboot_on || typeof app.reboot_on !== "object" || Array.isArray(app.reboot_on) || Object.values(app.reboot_on).some(value => typeof value !== "boolean")))) {
      throw new Error(`Invalid application fields: ${app.id}`);
    }
    for (const action of [app.check, app.install, app.update, app.uninstall, ...Object.values(app.actions ?? {})]) {
      if (action !== undefined && !validateCommands(action)) throw new Error(`Invalid commands: ${app.id}`);
    }
    ids.add(app.id);
  }
  for (const group of catalog.groups) {
    if (typeof group.id !== "string" || !group.id || typeof group.name !== "string" || !group.name) {
      throw new Error("Invalid catalog group");
    }
  }
  resolveSelection(catalog, [...ids]);
}

export function catalogGroups(catalog: Catalog) {
  const groups = new Map(catalog.groups.map(group => [group.id, group]));
  for (const app of catalog.apps) {
    if (app.group && !groups.has(app.group)) {
      groups.set(app.group, { id: app.group, name: app.group });
    }
  }
  return [...groups.values()];
}

export function filterApplications(catalog: Catalog, query: string, group: string) {
  const term = query.trim().toLocaleLowerCase();
  return catalog.apps.filter(app => (!group || app.group === group) &&
    `${app.name} ${app.description ?? ""} ${app.id}`.toLocaleLowerCase().includes(term));
}

export function toggleSelection(ids: string[], id: string) {
  return ids.includes(id) ? ids.filter(value => value !== id) : [...ids, id];
}

export function resolveSelection(catalog: Catalog, ids: string[]) {
  const apps = new Map(catalog.apps.map(app => [app.id, app]));
  const resolved = new Map<string, Application>();
  const visiting = new Set<string>();
  function visit(id: string) {
    if (visiting.has(id)) throw new Error(`Dependency cycle: ${id}`);
    if (resolved.has(id)) return;
    const app = apps.get(id);
    if (!app) throw new Error(`Unknown application: ${id}`);
    visiting.add(id);
    for (const dependency of app.depends_on ?? []) visit(dependency);
    visiting.delete(id);
    resolved.set(id, app);
  }
  for (const id of ids) visit(id);
  return [...resolved.values()];
}

export function createPreset(catalog: Catalog, ids: string[]) {
  const apps = resolveSelection(catalog, ids);
  if (!apps.length) throw new Error("Select at least one application");
  const groups = catalogGroups(catalog).filter(group => apps.some(app => app.group === group.id));
  return { version: catalog.version, groups, apps };
}

export function presetFilename(name: string) {
  const base = name.trim().replace(/\.json$/i, "").toLowerCase()
    .replace(/[^a-z0-9_-]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 80);
  return `${base || "my-preset"}.json`;
}

export function applicationPlatforms(app: Application) {
  const install = app.install ?? app.actions?.install;
  if (!install) return "No install commands";
  if (typeof install === "string" || Array.isArray(install)) return "Default commands";
  const labels: Record<string, string> = { ubuntu: "Ubuntu / Debian", fedora: "Fedora / RHEL", arch: "Arch", macos: "macOS", windows: "Windows", default: "Default commands", all: "Default commands", "*": "Default commands" };
  return Object.keys(install).map(key => labels[key] ?? key).join(" · ");
}
