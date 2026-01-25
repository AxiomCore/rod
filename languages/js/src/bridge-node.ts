// @ts-ignore - Types might not be generated perfectly for require
import * as wasmNode from "../wasm/node/rod_wasm";
import { Bridge, RodSchemaInstance } from "./bridge-types";

const schemaCache = new Map<string, RodSchemaInstance>();

export const nodeBridge: Bridge = {
    async init() {
        // Node.js loads WASM synchronously via 'fs' in the generated bindings.
        // No async init needed, but kept for API consistency.
        return Promise.resolve();
    },

    getSchema(spec: object): RodSchemaInstance {
        const key = JSON.stringify(spec);
        if (schemaCache.has(key)) return schemaCache.get(key)!;

        // wasmNode.RodSchema is the class constructor
        const schema = new wasmNode.RodSchema(spec);
        schemaCache.set(key, schema);
        return schema;
    }
};