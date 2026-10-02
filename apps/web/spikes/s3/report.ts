// Reads a saved S3 file (what `page.html` saves, or what `run.ts --write` wrote) and says what is in it, from the raw records:
//
//   node spikes/s3/report.ts <file.json> [--settle 3000] [--nominal 120]
//
// `--settle` is how long after a Start or a Continue the phase is not counted (ms); `--nominal` is the tempo the sender was set to, to
// compare the tempo the follower read with. Both are for a follow file and are ignored for a send file.
import { readFileSync } from "node:fs";
import { formatFollow, formatSend, readSaved } from "./format.ts";
import type { FollowResult, SendResult } from "./page.ts";

const argv = process.argv.slice(2);
const file = argv.find((a, i) => !a.startsWith("--") && !(i > 0 && argv[i - 1]?.startsWith("--")));
const opt = (name: string): number | undefined => {
  const i = argv.indexOf(name);
  return i >= 0 ? Number(argv[i + 1]) : undefined;
};
if (!file) {
  console.error("usage: node spikes/s3/report.ts <file.json> [--settle ms] [--nominal bpm]");
  process.exit(2);
}
try {
  const saved = readSaved(readFileSync(file, "utf8"));
  const settleMs = opt("--settle");
  const nominalBpm = opt("--nominal");
  const options = { ...(settleMs === undefined ? {} : { settleMs }), ...(nominalBpm === undefined ? {} : { nominalBpm }) };
  const lines =
    saved.result.schema === "wenge.s3.follow/1"
      ? formatFollow({ ...saved, result: saved.result as FollowResult }, options)
      : formatSend({ ...saved, result: saved.result as SendResult });
  console.log(lines.join("\n"));
} catch (e) {
  console.error(e instanceof Error ? e.message : String(e));
  process.exit(1);
}
