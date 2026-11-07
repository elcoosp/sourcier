import { chromium } from "playwright";

const browser = await chromium.launch();
const failures = [];

// 1. Mobile hamburger nav
const mob = await browser.newPage({ viewport: { width: 390, height: 844 } });
await mob.goto("http://localhost:4321/guides/installation/", { waitUntil: "networkidle" });
const burger = mob.locator("button.sl-menu-button").first();
if (await burger.count()) {
	await burger.click();
	await mob.waitForTimeout(400);
	await mob.waitForTimeout(200);
	const sidebarVisible = await mob.evaluate(() => {
		// The menu button toggles the starlight__sidebar popover; after the
		// click the sidebar nav list becomes visible.
		const panel = document.querySelector("nav.sidebar ul");
		return {
			popoverOpen: document.querySelector("nav.sidebar")?.matches(":popover-open") ?? false,
			panelDisplay: panel ? getComputedStyle(panel).display : null,
		};
	});
	console.log("hamburger clicked:", JSON.stringify(sidebarVisible));
	if (sidebarVisible.panelDisplay === "none") failures.push("mobile sidebar did not open after hamburger click");
	const shot = await mob.screenshot({ path: ".qa/screenshots/manual-hamburger-mobile.png" });
	console.log("hamburger screenshot saved");
} else {
	failures.push("no hamburger button found");
}
await mob.close();

// 2. Pagefind search modal
const desk = await browser.newPage({ viewport: { width: 1440, height: 900 } });
await desk.goto("http://localhost:4321/", { waitUntil: "networkidle" });
await desk.evaluate(() => document.fonts.ready);
// Open search with Ctrl+K (Starlight default shortcut)
await desk.keyboard.press("ControlOrMeta+k");
await desk.waitForTimeout(600);
let dialog = desk.locator("dialog[open], starlight-search-modal[open], .search-modal");
let dialogCount = await dialog.count();
if (!dialogCount) {
	// fall back: click the search button in the header
	const searchBtn = desk.locator("button[aria-keyshortcuts='Control+K'], starlight-search button, button[aria-label='Search']").first();
	if (await searchBtn.count()) {
		await searchBtn.click();
		await desk.waitForTimeout(600);
		dialogCount = await dialog.count();
	}
}
console.log("search dialog open:", dialogCount > 0);
await desk.keyboard.type("finalize", { delay: 60 });
await desk.waitForTimeout(1200);
// pagefind renders results inside the dialog
const results = await desk.evaluate(() => {
	const dlg = document.querySelector("dialog[open]");
	if (!dlg) return { open: false };
	const items = [...dlg.querySelectorAll("a, .pagefind-ui__result")];
	const texts = items.slice(0, 5).map((i) => i.textContent?.trim().slice(0, 80));
	return { open: true, count: items.length, texts };
});
console.log("pagefind results:", JSON.stringify(results, null, 1));
if (!results.open) failures.push("search modal did not open");
else if (!results.count) failures.push("pagefind returned no results for 'finalize'");
await desk.screenshot({ path: ".qa/screenshots/manual-search-desktop.png" });

await browser.close();
if (failures.length) {
	console.error("MANUAL CHECKS FAILED:", failures);
	process.exit(1);
}
console.log("MANUAL CHECKS PASSED");
process.exit(0);
