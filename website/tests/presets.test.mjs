import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import test from "node:test";
import { loadCatalog } from "../src/lib/catalog.ts";
import {
  applicationPlatforms, createPreset, filterApplications, presetFilename,
  resolveSelection, starterPresets, toggleSelection, validateCatalog,
} from "../src/lib/presets.ts";

const directory = new URL("../../presets/", import.meta.url);
const catalog = loadCatalog(Object.fromEntries(readdirSync(directory).filter(file => /\.ya?ml$/.test(file))
  .map(file => [file, readFileSync(new URL(file, directory), "utf8")])));

test("application files and starter collections form a valid catalog", () => {
  validateCatalog(catalog);
  for (const preset of starterPresets) {
    const exported = createPreset(catalog, preset.ids);
    assert.ok(exported.apps.length >= preset.ids.length);
    for (const app of exported.apps) {
      assert.ok(app.actions.install);
      for (const dependency of app.depends_on ?? []) {
        assert.ok(exported.apps.some(app => app.id === dependency));
      }
    }
  }
});

test("each preset file contains one application, without an apps wrapper", () => {
  assert.throws(() => loadCatalog({ "bad.yaml": "apps:\n  - id: git\n    name: Git\n" }), /one application object/);
  assert.throws(() => loadCatalog({ "bad.yaml": "- id: git\n  name: Git\n" }), /one application object/);
});

test("search matches names, descriptions, and IDs alongside category filters", () => {
  assert.deepEqual(filterApplications(catalog, "  NODE.JS  ", "development").map(app => app.id), ["nodejs", "npm"]);
  assert.deepEqual(filterApplications(catalog, "distributed", "development").map(app => app.id), ["git"]);
  assert.deepEqual(filterApplications(catalog, "Git", "everyday"), []);
  assert.deepEqual(filterApplications(catalog, "no match exists", ""), []);
  assert.equal(filterApplications(catalog, "", "").length, catalog.apps.length);
});

test("selection toggles without duplicates, and removing a parent drops unused dependencies", () => {
  let ids = toggleSelection([], "npm");
  assert.deepEqual(resolveSelection(catalog, ids).map(app => app.id), ["nodejs", "npm"]);
  ids = toggleSelection(ids, "git");
  ids = toggleSelection(ids, "npm");
  assert.deepEqual(resolveSelection(catalog, ids).map(app => app.id), ["git"]);
  assert.deepEqual(resolveSelection(catalog, ["npm", "nodejs", "npm"]).map(app => app.id), ["nodejs", "npm"]);
  assert.deepEqual(resolveSelection(catalog, []), []);
});

test("shared and transitive dependencies appear once, before their consumers", () => {
  const graph = { version: 1, groups: [], apps: [
    { id: "app", name: "App", depends_on: ["a", "b"] },
    { id: "a", name: "A", depends_on: ["shared"] },
    { id: "b", name: "B", depends_on: ["shared"] },
    { id: "shared", name: "Shared" },
  ] };
  assert.deepEqual(resolveSelection(graph, ["app"]).map(app => app.id), ["shared", "a", "b", "app"]);
  graph.apps[3].depends_on = ["app"];
  assert.throws(() => validateCatalog(graph), /Dependency cycle/);
});

test("invalid IDs, command definitions, and dependency references fail the build", () => {
  const invalid = { version: 1, groups: [], apps: [{ id: "bad", name: "Bad", depends_on: ["missing"] }] };
  assert.throws(() => validateCatalog(invalid), /Unknown application/);
  assert.throws(() => validateCatalog({ ...catalog, apps: [...catalog.apps, catalog.apps[0]] }), /duplicate/);
  assert.throws(() => validateCatalog({ ...catalog, apps: [{ id: "bad", name: "Bad", install: { ubuntu: [42] } }] }), /Invalid commands/);
  assert.throws(() => resolveSelection(catalog, ["missing"]), /Unknown application/);
  assert.throws(() => createPreset(catalog, []), /Select at least one/);
});

test("export preserves commands and metadata, with only relevant groups", () => {
  const result = createPreset(catalog, ["npm"]);
  assert.deepEqual(result.groups, [{ id: "development", name: "Development" }]);
  for (const app of result.apps) assert.deepEqual(app, catalog.apps.find(source => source.id === app.id));
  assert.deepEqual(result, JSON.parse(readFileSync(new URL("./fixtures/node-package-manager.json", import.meta.url), "utf8")));
});

test("filenames stay safe and predictable", () => {
  assert.equal(presetFilename(" My Work Setup.JSON "), "my-work-setup.json");
  assert.equal(presetFilename("../../my setup"), "my-setup.json");
  assert.equal(presetFilename("💻"), "my-preset.json");
  assert.ok(presetFilename("a".repeat(500)).length <= 85);
});

test("platform labels distinguish explicit support from fallback commands", () => {
  assert.match(applicationPlatforms(catalog.apps.find(app => app.id === "git")), /Windows/);
  assert.doesNotMatch(applicationPlatforms(catalog.apps.find(app => app.id === "npm")), /Windows|macOS/);
  assert.equal(applicationPlatforms({ id: "test", name: "Test", install: "install test" }), "Default commands");
});
