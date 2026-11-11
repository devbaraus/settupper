import { parse } from "yaml";
import { resolveSelection, starterPresets, validateCatalog, type Catalog } from "./presets.ts";

// Read only at build time; installation commands are never executed by the website.
export function loadCatalog(files: Record<string, string>): Catalog {
  const catalog: Catalog = {
    version: 1,
    groups: [
      { id: "development", name: "Development" },
      { id: "terminal", name: "Terminal" },
      { id: "everyday", name: "Everyday apps" },
    ],
    apps: Object.keys(files).sort().map(file => {
      const app = parse(files[file]);
      if (!app || typeof app !== "object" || Array.isArray(app) || "apps" in app) {
        throw new Error(`${file} must contain one application object`);
      }
      return app;
    }),
  };
  validateCatalog(catalog);
  for (const preset of starterPresets) resolveSelection(catalog, preset.ids);
  return catalog;
}
