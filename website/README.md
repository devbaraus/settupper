# Settupper landing page

A static Astro site with a SolidJS page template, Tailwind CSS utilities, and Tabler icons from `@tabler/icons-solidjs`. Includes platform-specific installation commands, a starter YAML download, usage instructions, keyboard shortcuts, and accessible copy controls.

Requires Node.js 22.23.4+ (or a supported newer even-numbered release).

```sh
cd website
npm ci
npm run dev
```

Create the production site with `npm run build`. Serve `dist/` with any static host, or run `npm run preview` to inspect the build locally.

`src/pages/index.astro` owns the document metadata and hydrates `src/components/LandingPage.tsx`. Solid signals power `InstallTabs.tsx` and `CopyButton.tsx`. Tailwind runs through its Vite plugin; `src/styles/global.css` contains only its import and theme tokens. Apply styles with utility classes in the templates.

The downloadable config is `public/packages-example.yaml`. Installation commands follow the repository's installers and always fetch the latest release. Without JavaScript, all installation commands remain visible and copy buttons are hidden.
