export interface RodWasmModule {
    RodSchema: {
        new(spec: any): RodSchemaInstance;
    };
}

export interface RodSchemaInstance {
    validate_lazy(data: any): any;
    validate_eager(data: any): any;
    validate_batch(data: any): any;
    check_batch(data: any): Uint8Array;
    check_batch_eager(data: any): Uint8Array;
    free?(): void; // Wasm memory management
}

export interface Bridge {
    init(): Promise<void>;
    getSchema(spec: object): RodSchemaInstance;
}