// Serves the built app for a person: `npm run app`, then open the address it prints.
import { serve } from "./serve.ts";

const port = Number(process.env["PORT"] ?? 8080);
const server = await serve(port);
console.log(`${server.url}/pages/app.html`);
