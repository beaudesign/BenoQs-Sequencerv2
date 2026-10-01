// Starts the Chromium that playwright-core finds (PLAYWRIGHT_BROWSERS_PATH, or `npx playwright-core install chromium`).
import { chromium, type Browser } from "playwright-core";

export interface LaunchOptions {
  /** A real window, for the hidden-tab runs. Needs a display (`xvfb-run` on a server). */
  headed?: boolean;
}

export async function launch(options: LaunchOptions = {}): Promise<Browser> {
  const path = process.env["CHROMIUM_PATH"];
  return chromium.launch({
    headless: !options.headed,
    ...(path ? { executablePath: path } : {}),
    // Audio may start without a click, and a throttled background timer would blur what the spikes measure.
    args: ["--autoplay-policy=no-user-gesture-required", "--disable-background-timer-throttling", "--disable-renderer-backgrounding"],
  });
}
