import init, { RodSchema } from "../wasm/web/rod_wasm";
import { Bridge, RodSchemaInstance } from "./bridge-types";

// Cache map
const schemaCache = new Map<string, RodSchemaInstance>();
let initialized = false;

export const browserBridge: Bridge = {
    async init() {
        if (!initialized) {
            await init(); // Fetches and compiles wasm
            initialized = true;
        }
    },

    getSchema(spec: object): RodSchemaInstance {
        if (!initialized) throw new Error("Rod not initialized. Call rod.init()");

        const key = JSON.stringify(spec);
        if (schemaCache.has(key)) return schemaCache.get(key)!;

        const schema = new RodSchema(spec);
        schemaCache.set(key, schema);
        return schema;
    }
};