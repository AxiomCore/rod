import { Bridge } from "./bridge-types";
import { RodString, RodNumber, RodBoolean, RodArray, RodObject, RodType, RodAny, RodNever } from "./builders";

export function createRod(bridge: Bridge) {
    return {
        init: () => bridge.init(),
        string: () => new RodString(bridge),
        number: () => new RodNumber(bridge),
        boolean: () => new RodBoolean(bridge),
        array: (schema: RodType) => new RodArray(schema, bridge), // RodArray needs ctor update
        object: (shape: Record<string, RodType>) => new RodObject(shape, bridge), // RodObject needs ctor update
        any: () => new RodAny(bridge),
        never: () => new RodNever(bridge),
    };
}