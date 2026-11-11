# Application presets

Each `.yaml` or `.yml` file contains **one application object**, with the same fields as a single entry in a Settupper configuration's `apps` array. Do not wrap the object in `apps:` or add configuration-level fields such as `version` or `groups`.

The website builds its catalog exclusively from these files. The root demo configuration is independent.

Use a unique, stable `id`, a `name`, a short `description`, and a `group`. Include `check` commands and `actions.install`, `actions.update`, and `actions.uninstall` maps for the platforms you support. `depends_on` references other application IDs in this folder. The builder includes dependencies automatically, preserves the complete objects, and exports them under `apps` in a version 1 JSON configuration.

Current groups are `development`, `terminal`, and `everyday`. Additional group IDs also appear in the category filter. Files are loaded at build time: rebuild the website after adding or editing a preset. Invalid IDs, commands, missing dependencies, dependency cycles, and missing starter-collection references fail the build.

Commands assume the platform's package manager and package sources are configured. Only list explicit platform commands you can support; a fallback command does not imply universal OS compatibility. These files contain executable commands and should be reviewed before use.

Starter collections reference these application IDs in `website/src/lib/presets.ts`. After intentionally changing npm or Node.js definitions, regenerate `website/tests/fixtures/node-package-manager.json` with `createPreset(catalog, ["npm"])`; the website tests compare it with the export, and the Rust tests load that same fixture.
