// Starts the Chromium that playwright-core finds (PLAYWRIGHT_BROWSERS_PATH, or `npx playwright-core install chromium`).
import { chromium, type Browser } from "playwright-core";

export interface LaunchOptions {
  /** A real window, for the hidden-tab runs. Needs a display (`xvfb-run` on a server). */
  headed?: boolean;
  /** Leave the browser's background throttling as it is, which the hidden-tab runs need. Tests turn it off. */
  realisticBackground?: boolean;
}

export async function launch(options: LaunchOptions = {}): Promise<Browser> {
  const path = process.env["CHROMIUM_PATH"];
  return chromium.launch({
    headless: !options.headed,
    ...(path ? { executablePath: path } : {}),
    // Audio may start without a click. A tab in the background is throttled by default; tests switch that
    // off so they measure the page and not the browser's power saving, and the hidden-tab runs leave it on.
    args: [
      "--autoplay-policy=no-user-gesture-required",
      ...(options.realisticBackground ? [] : ["--disable-background-timer-throttling", "--disable-renderer-backgrounding"]),
    ],
  });
}
