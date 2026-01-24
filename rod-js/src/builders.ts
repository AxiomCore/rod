import { Bridge } from "./bridge-types";

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

    // Inject the bridge strategy
    constructor(type: string, protected bridge: Bridge) {
        this.def = { type };
    }

    // Returns the raw JSON spec required by Rust
    toSpec(): object {
        return this.def;
    }

    /**
     * Parse data against the schema. Throws error if validation fails.
     * @param data The input data (unknown)
     * @param opts Configuration for this run
     */
    parse(data: unknown, opts?: ParseOptions): any {
        const schema = this.bridge.getSchema(this.toSpec());
        const mode = opts?.mode ?? "lazy";

        let result;
        if (mode === "lazy") {
            result = schema.validate_lazy(data);
        } else {
            result = schema.validate_eager(data);
        }

        if (!result.success) {
            // Throw error to match Zod behavior
            throw new Error(JSON.stringify(result.error, null, 2));
        }

        return result.data;
    }

    /**
     * Parse data safely without throwing.
     */
    safeParse(data: unknown, opts?: ParseOptions): ParseResult<any> {
        try {
            const res = this.parse(data, opts);
            return { success: true, data: res };
        } catch (e: any) {
            // RodError is structured JSON, so we parse the error message if it was stringified
            let error;
            try {
                error = JSON.parse(e.message);
            } catch {
                error = e.message;
            }
            return { success: false, error };
        }
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