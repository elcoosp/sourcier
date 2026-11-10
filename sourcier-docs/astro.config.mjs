// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import astroTakumi from 'astro-takumi';
import fs from 'node:fs';
import { makeRender } from './src/og-render.mjs';
import starlightLinksValidator from "starlight-links-validator";
import starlightImageZoom from "starlight-image-zoom";
import starlightHeadingBadges from "starlight-heading-badges";
import starlightLlmsTxt from "starlight-llms-txt";

const __logoB64 = fs.readFileSync("./src/assets/logo.png").toString("base64");
const __render = makeRender({
  logoDataUrl: "data:image/png;base64," + __logoB64,
  accentHex: "#36417d",
  accentHexA12: "#36417d1a",
  domain: "sourcier.vercel.app",
  projectName: "Sourcier",
});

export default defineConfig({
	vite: {
		ssr: { external: ["jsdom", "astro-takumi"] },
	},
	// Canonical deployment origin. Starlight always registers its own
	// sitemap integration, which requires `site`; starlight-llms-txt uses
	// the same origin for the absolute URLs in llms.txt. Change this single
	// line when the docs get their real domain. See
	// docs/decisions/0006-site-and-llms-txt.md.
	site: 'https://sourcier.vercel.app',
	integrations: [
		starlight({
			routeMiddleware: "./src/routeData.ts",
			title: "Sourcier",
			description:
				"Compact and efficient source file mapping and position tracking for Rust: register files once, pack every span into a single u64.",


			expressiveCode: {
				themes: ['github-light', 'github-dark'],
			},			logo: { src: './src/assets/logo.png' },
			favicon: '/favicon.png',
			customCss: ["./src/styles/custom.css"],
			social: [
				{
					icon: "github",
					label: "GitHub",
					href: "https://github.com/elcoosp/sourcier",
				},
			],
			editLink: {
				baseUrl:
					"https://github.com/elcoosp/sourcier/edit/main/docs-site/src/content/docs/",
			},
			// Single-locale site: the root locale avoids an i18n
			// UI-translation collection, which this site does not need.
			locales: {
				root: {
					label: "English",
					lang: "en",
				},
			},
			defaultLocale: "root",
			plugins: [
				// Fails the build on broken internal links — release blocker.
				starlightLinksValidator(),
				// Click-to-zoom for documentation images.
				starlightImageZoom(),
				// :badge[...] directives in headings for feature-gated APIs.
				starlightHeadingBadges(),
				// Generates llms.txt, llms-small.txt and llms-full.txt.
				starlightLlmsTxt(),
			],
			sidebar: [
				{
					label: "Introduction",
					items: [{ label: "Overview", link: "/" }],
				},
				{
					label: "Tutorials",
					items: [
						{
							label: "Your first source map",
							link: "/tutorials/getting-started/",
						},
					],
				},
				{
					label: "How-to guides",
					items: [
						{ label: "Installation", link: "/guides/installation/" },
						{
							label: "Registering source files",
							link: "/guides/registering-files/",
						},
						{
							label: "Working with positions",
							link: "/guides/working-with-positions/",
						},
						{
							label: "Views and navigation",
							link: "/guides/navigating-source/",
						},
						{
							label: "Sharing usage statistics",
							link: "/guides/usage-statistics/",
						},
						{ label: "Handling errors", link: "/guides/handling-errors/" },
						{
							label: "Tuning performance",
							link: "/guides/tuning-performance/",
						},
					],
				},
				{
					label: "Reference",
					items: [
						{ label: "API surface", link: "/reference/api/" },
						{
							label: "Position encoding",
							link: "/reference/position-encoding/",
						},
						{ label: "Feature flags", link: "/reference/feature-flags/" },
						{ label: "Errors", link: "/reference/errors/" },
						{ label: "Changelog", link: "/reference/changelog/" },
					],
				},
				{
					label: "Explanation",
					items: [
						{
							label: "Why Sourcier exists",
							link: "/explanation/why-sourcier/",
						},
						{
							label: "The two-phase lifecycle",
							link: "/explanation/two-phase-lifecycle/",
						},
						{
							label: "Performance design",
							link: "/explanation/performance-design/",
						},
						{
							label: "Safety and stability",
							link: "/explanation/safety-and-stability/",
						},
					],
				},
			],
		}),
	
		astroTakumi({
			options: {
				domBackend: "linkedom",
				fonts: [fs.readFileSync("./node_modules/@fontsource/roboto/files/roboto-latin-400-normal.woff2")],
				width: 1200,
				height: 630,
				format: "png",
			},
			render: __render,
		}),
	],
});
