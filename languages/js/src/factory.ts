import { Bridge } from "./bridge-types";
import * as b from "./builders";

export function createRod(bridge: Bridge) {
    return {
        init: () => bridge.init(),

        fromJSON: (spec: object) => {
            return new b.RodDynamic(spec, bridge);
        },

        // Primitives
        string: () => new b.RodString(bridge),
        number: () => new b.RodNumber(bridge),
        boolean: () => new b.RodBoolean(bridge),
        date: () => new b.RodDate(bridge),
        literal: (val: any) => new b.RodLiteral(val, bridge),
        enum: (values: string[]) => new b.RodEnum(values, bridge),

        // Collections
        array: (schema: b.RodType) => new b.RodArray(schema, bridge),
        object: (shape: Record<string, b.RodType>) => new b.RodObject(shape, bridge),
        tuple: (items: b.RodType[]) => new b.RodTuple(items, bridge),
        record: (key: b.RodType, value: b.RodType) => new b.RodRecord(key, value, bridge),
        map: (key: b.RodType, value: b.RodType) => new b.RodMap(key, value, bridge),
        set: (value: b.RodType) => new b.RodSet(value, bridge),

        // Logic
        union: (options: b.RodType[]) => new b.RodUnion(options, bridge),
        discriminatedUnion: (disc: string, options: b.RodType[]) =>
            new b.RodDiscriminatedUnion(disc, options, bridge),
        any: () => new b.RodAny(bridge),
        never: () => new b.RodNever(bridge),
    };
}