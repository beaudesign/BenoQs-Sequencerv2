// Writes dist/tokens.css from contracts/design.tokens.json. Run by `npm run build` before tsc.
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseTokens } from "../src/tokens.ts";
import { tokensCss } from "../src/tokens-css.ts";

const here = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(here, "..");
const tokens = parseTokens(JSON.parse(readFileSync(join(webRoot, "../../contracts/design.tokens.json"), "utf8")));
mkdirSync(join(webRoot, "dist"), { recursive: true });
writeFileSync(join(webRoot, "dist/tokens.css"), tokensCss(tokens));
