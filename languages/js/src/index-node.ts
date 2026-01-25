import { nodeBridge } from "./bridge-node";
import { createRod } from "./factory";

export const rod = createRod(nodeBridge);
export * from "./builders";