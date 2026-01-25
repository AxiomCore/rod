import { browserBridge } from "./bridge-browser";
import { createRod } from "./factory";

export const rod = createRod(browserBridge);
export * from "./builders";