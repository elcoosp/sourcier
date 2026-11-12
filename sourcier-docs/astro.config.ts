import { defineDocsConfig } from '@elcoosp/docs-toolkit/config';
import starlightLinksValidator from "starlight-links-validator";
import starlightImageZoom from "starlight-image-zoom";
import starlightHeadingBadges from "starlight-heading-badges";
import starlightLlmsTxt from "starlight-llms-txt";

export default await defineDocsConfig({
  llmsTxt: {},
  changelog: { repo: 'elcoosp/sourcier' },
  site: 'https://sourcier.vercel.app',
  domain: 'sourcier.vercel.app',
  title: "Sourcier",
  description: "Compact and efficient source file mapping and position tracking for Rust: register files once, pack every span into a single u64.",
  sidebar: [],
  extraStarlight: {
			title: "Sourcier",
			description:
				"Compact and efficient source file mapping and position tracking for Rust: register files once, pack every span into a single u64.",

			expressiveCode: {
				themes: ['github-light', 'github-dark']
			},
			favicon: '/favicon.png',
			social: [
				{
					icon: "github",
					label: "GitHub",
					href: "https://github.com/elcoosp/sourcier"
				},
			],
			editLink: {
				baseUrl:
					"https://github.com/elcoosp/sourcier/edit/main/docs-site/src/content/docs/"
			},
			// Single-locale site: the root locale avoids an i18n
			// UI-translation collection, which this site does not need.

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
					items: [{ label: "Overview", translations: { fr: 'Aperçu' }, link: "/" }]
				},
				{
					label: "Tutorials", translations: { fr: 'Tutoriels' },
					items: [
						{
							label: "Your first source map",
							link: "/tutorials/getting-started/"
						},
					]
				},
				{
					label: "How-to guides", translations: { fr: 'Guides pratiques' },
					items: [
						{ label: "Installation", translations: { fr: "Installation" }, link: "/guides/installation/" },
						{
							label: "Registering source files",
							link: "/guides/registering-files/"
						},
						{
							label: "Working with positions",
							link: "/guides/working-with-positions/"
						},
						{
							label: "Views and navigation",
							link: "/guides/navigating-source/"
						},
						{
							label: "Sharing usage statistics",
							link: "/guides/usage-statistics/"
						},
						{ label: "Handling errors", link: "/guides/handling-errors/" },
						{
							label: "Tuning performance",
							link: "/guides/tuning-performance/"
						},
					]
				},
				{
					label: "Reference", translations: { fr: 'Référence' },
					items: [
						{ label: "API surface", link: "/reference/api/" },
						{
							label: "Position encoding",
							link: "/reference/position-encoding/"
						},
						{ label: "Feature flags", link: "/reference/feature-flags/" },
						{ label: "Errors", link: "/reference/errors/" },
						{ label: "Changelog", translations: { fr: 'Journal des modifications' }, link: "/reference/changelog/" },
					]
				},
				{
					label: "Explanation", translations: { fr: 'Explication' },
					items: [
						{
							label: "Why Sourcier exists",
							link: "/explanation/why-sourcier/"
						},
						{
							label: "The two-phase lifecycle",
							link: "/explanation/two-phase-lifecycle/"
						},
						{
							label: "Performance design",
							link: "/explanation/performance-design/"
						},
						{
							label: "Safety and stability",
							link: "/explanation/safety-and-stability/"
						},
					]
				},
			]
		},
});
