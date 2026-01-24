export interface RodWasmModule {
    RodSchema: {
        new(spec: any): RodSchemaInstance;
    };
}

export interface RodSchemaInstance {
    validate_lazy(data: any): any;
    validate_eager(data: any): any;
    free?(): void; // Wasm memory management
}

export interface Bridge {
    init(): Promise<void>;
    getSchema(spec: object): RodSchemaInstance;
}