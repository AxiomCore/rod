import { Bridge, RodSchemaInstance } from "./bridge-types";

export type ParseOptions = {
    /**
     * 'lazy': Uses JS reflection (Reflect.get) to traverse data. 
     * Best for large objects where only a few fields are validated.
     * 
     * 'eager': Serializes entire input to Rust JSON structure first. 
     * Best for heavy validation logic on smaller objects or massive arrays of primitives.
     * 
     * @default "lazy"
     */
    mode?: "lazy" | "eager";
};

export type ParseResult<T> =
    | { success: true; data: T }
    | { success: false; error: any };

// Base class for all validators
export abstract class RodType {
    protected def: Record<string, any>;
    private _compiled: RodSchemaInstance | null = null;

    constructor(type: string, protected bridge: Bridge) {
        this.def = { type };
    }

    protected getCompiled(): RodSchemaInstance {
        if (!this._compiled) {
            this._compiled = this.bridge.getSchema(this.toSpec());
        }
        return this._compiled;
    }

    toSpec(): object { return this.def; }

    parse(data: unknown, opts?: ParseOptions): any {
        const schema = this.getCompiled();
        const mode = opts?.mode ?? "lazy";

        const result = mode === "lazy"
            ? schema.validate_lazy(data)
            : schema.validate_eager(data);

        if (!result.success) {
            throw new Error(JSON.stringify(result.error));
        }

        // result.data is the raw handle we returned in the Rust fast-path
        return result.data;
    }

    safeParse(data: unknown, opts?: { mode?: "lazy" | "eager" }): any {
        try {
            return { success: true, data: this.parse(data, opts) };
        } catch (e: any) {
            return { success: false, error: JSON.parse(e.message) };
        }
    }

    /**
     * The fastest possible batch validation.
     * Returns a Uint8Array bitmask (1 = valid, 0 = invalid).
     * Bypasses all JS object creation overhead.
     */
    checkBatch(items: any[]): Uint8Array {
        const schema = this.getCompiled();
        return schema.check_batch(items);
    }

    checkBatchEager(items: any[]): Uint8Array {
        return this.getCompiled().check_batch_eager(items);
    }

    parseBatch(items: any[]): any[] {
        const schema = this.getCompiled();
        // validate_batch returns an array where null = failure
        return schema.validate_batch(items);
    }
}

// --- Primitives ---

export class RodString extends RodType {
    constructor(bridge: Bridge) { super("string", bridge); }

    min(val: number) { this.def.min = val; return this; }
    max(val: number) { this.def.max = val; return this; }
    length(val: number) { this.def.length = val; return this; }
    email() { this.def.email = true; return this; }
    url() { this.def.url = true; return this; }
    uuid() { this.def.uuid = true; return this; }
    cuid() { this.def.cuid = true; return this; }
    datetime() { this.def.datetime = true; return this; }
    ip() { this.def.ip = true; return this; }
    regex(pattern: string) { this.def.regex = pattern; return this; }
    startsWith(val: string) { this.def.starts_with = val; return this; }
    endsWith(val: string) { this.def.ends_with = val; return this; }
    includes(val: string) { this.def.includes = val; return this; }
    trim() { this.def.trim = true; return this; }
}

export class RodNumber extends RodType {
    constructor(bridge: Bridge) { super("number", bridge); }

    min(val: number) { this.def.min = val; return this; }
    max(val: number) { this.def.max = val; return this; }
    int() { this.def.int = true; return this; }
}

export class RodBoolean extends RodType {
    constructor(bridge: Bridge) { super("boolean", bridge); }
}

export class RodDate extends RodType {
    constructor(bridge: Bridge) { super("date", bridge); }

    min(timestamp: number) { this.def.min = timestamp; return this; }
    max(timestamp: number) { this.def.max = timestamp; return this; }
}

export class RodLiteral extends RodType {
    constructor(value: any, bridge: Bridge) {
        super("literal", bridge);
        this.def.value = value;
    }
}

export class RodEnum extends RodType {
    constructor(values: string[], bridge: Bridge) {
        super("enum", bridge);
        this.def.values = values;
    }
}

// --- Collections ---

export class RodArray extends RodType {
    constructor(itemSchema: RodType, bridge: Bridge) {
        super("array", bridge);
        this.def.items = itemSchema.toSpec();
    }

    min(val: number) { this.def.min = val; return this; }
    max(val: number) { this.def.max = val; return this; }
}

export class RodObject extends RodType {
    constructor(shape: Record<string, RodType>, bridge: Bridge) {
        super("object", bridge);
        const props: Record<string, any> = {};
        for (const k in shape) {
            props[k] = shape[k].toSpec();
        }
        this.def.properties = props;
    }

    strict() { this.def.strict = true; return this; }
    passthrough() {
        // Note: passthrough support might need explicit logic in your Parser logic in Rust 
        // to map 'strict: false' or a specific flag. Assuming default (strip) vs strict.
        // If Rust parser supports 'passthrough' flag, add it here.
        // For now, based on previous code, we only had strict option exposed in parser.
        // If needed: this.def.passthrough = true; 
        return this;
    }
}

export class RodTuple extends RodType {
    constructor(items: RodType[], bridge: Bridge) {
        super("tuple", bridge);
        this.def.items = items.map(i => i.toSpec());
    }
}

export class RodRecord extends RodType {
    constructor(keySchema: RodType, valueSchema: RodType, bridge: Bridge) {
        super("record", bridge);
        this.def.key = keySchema.toSpec();
        this.def.value = valueSchema.toSpec();
    }
}

export class RodMap extends RodType {
    constructor(keySchema: RodType, valueSchema: RodType, bridge: Bridge) {
        super("map", bridge);
        this.def.key = keySchema.toSpec();
        this.def.value = valueSchema.toSpec();
    }
}

export class RodSet extends RodType {
    constructor(valueSchema: RodType, bridge: Bridge) {
        super("set", bridge);
        this.def.value = valueSchema.toSpec();
    }

    min(val: number) { this.def.min = val; return this; }
}

// --- Logic ---

export class RodUnion extends RodType {
    constructor(options: RodType[], bridge: Bridge) {
        super("union", bridge);
        this.def.options = options.map(o => o.toSpec());
    }
}

export class RodDiscriminatedUnion extends RodType {
    constructor(discriminator: string, options: RodType[], bridge: Bridge) {
        super("discriminatedUnion", bridge);
        this.def.discriminator = discriminator;
        this.def.options = options.map(o => o.toSpec());
    }
}

export class RodAny extends RodType {
    constructor(bridge: Bridge) {
        super("any", bridge);
    }
}

export class RodNever extends RodType {
    constructor(bridge: Bridge) {
        super("never", bridge);
    }
}