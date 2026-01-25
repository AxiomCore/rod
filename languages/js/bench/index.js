import { run, bench, group } from 'mitata';
import { z } from 'zod';
import { rod } from 'rod-js';

// Wait for WASM to load
await rod.init();

// --- SCENARIOS ---

// 1. Simple Object (Bridge Tax Test)
const simpleZod = z.object({ name: z.string(), age: z.number() });
const simpleRod = rod.object({ name: rod.string(), age: rod.number() });
const simpleData = { name: "John Doe", age: 30 };

// 2. Heavy Computation (Format & Logic)
const heavyZod = z.object({
    id: z.string().uuid(),
    email: z.string().email(),
    tags: z.array(z.string().min(3)).min(10)
});
const heavyRod = rod.object({
    id: rod.string().uuid(),
    email: rod.string().email(),
    tags: rod.array(rod.string().min(3)).min(10)
});
const heavyData = {
    id: "550e8400-e29b-41d4-a716-446655440000",
    email: "performance@rod.rs",
    tags: Array(12).fill("rust-lang")
};

// 3. Deeply Nested / Large Object (Lazy Mode Test)
const generateDeep = (depth) => {
    if (depth === 0) return { leaf: "value" };
    return { nest: generateDeep(depth - 1) };
};
const deepData = generateDeep(50);
// We only validate the top level and one middle field to test Lazy traversal
const deepRod = rod.object({ nest: rod.any() });
const deepZod = z.object({ nest: z.any() });

// 4. Large Primitive Array (Eager Mode Test)
const listData = Array(100000).fill(0).map(() => Math.random() * 100);
const listZod = z.array(z.number());
const listRod = rod.array(rod.number());

// --- BENCHMARKS ---

group('Simple Object (Should favor Zod)', () => {
    bench('zod', () => { simpleZod.parse(simpleData); });
    bench('rod (lazy)', () => { simpleRod.parse(simpleData, { mode: 'lazy' }); });
});

group('Heavy Logic (UUID/Email/Min)', () => {
    bench('zod', () => { heavyZod.parse(heavyData); });
    bench('rod (lazy)', () => { heavyRod.parse(heavyData, { mode: 'lazy' }); });
    bench('rod (eager)', () => { heavyRod.parse(heavyData, { mode: 'eager' }); });
});

group('Deep Nesting (Any)', () => {
    bench('zod', () => { deepZod.parse(deepData); });
    bench('rod (lazy)', () => { deepRod.parse(deepData, { mode: 'lazy' }); });
});

group('Massive Number Array (100000 items)', () => {
    bench('zod', () => { listZod.parse(listData); });
    bench('rod (eager)', () => { listRod.parse(listData, { mode: 'eager' }); });
});

const manyItems = Array.from({ length: 1000 }, () => ({ ...heavyData, tags: [...heavyData.tags] }));

group('Batch Validation (1000 heavy items)', () => {
    bench('zod', () => { manyItems.map(i => heavyZod.parse(i)); });
    // We can implement a .parseBatch() in builders.ts
    bench('rod (batch)', () => { heavyRod.parseBatch(manyItems); });
});

group('Validity Filtering (1000 items)', () => {
    // Zod has to create 1000 result objects even if we only want the boolean
    bench('zod (safeParse.success)', () => {
        manyItems.map(i => heavyZod.safeParse(i).success);
    });

    // Rod writes 1000 bytes into a single pre-allocated buffer
    bench('rod (checkBatch)', () => {
        heavyRod.checkBatch(manyItems);
    });

    bench('rod (checkBatchEager)', () => {
        heavyRod.checkBatchEager(manyItems);
    });
});


const megaShape = {};
const megaData = {};
for (let i = 0; i < 50; i++) {
    megaShape[`field_${i}`] = z.string();
    megaData[`field_${i}`] = "value";
}
const megaZod = z.object(megaShape);
const megaRod = rod.object(Object.fromEntries(
    Object.keys(megaShape).map(k => [k, rod.string()])
));

// 6. Computationally Expensive Refine
const expensiveCheck = (val) => {
    // Simulate a heavy check (e.g., custom checksum or complex logic)
    let hash = 0;
    for (let i = 0; i < 1000; i++) {
        hash = (hash + val.length + i) % 1000000;
    }
    return hash > 0;
};

// Note: To run this in Rod, we'd need to register a Rust function, 
// but for the sake of the "Computation" test, let's use a long regex.
const complexRegex = /^(?=(.*[a-z]){3})(?=(.*[A-Z]){2})(?=(.*[0-9]){2})(?=(.*[!@#$%^&*()\-__+.]){1}).{8,}$/;
const computeZod = z.string().regex(complexRegex);
const computeRod = rod.string().regex(complexRegex.source);
const computeData = "Pass123!wordLonger";

// --- Add to groups ---
group('Mega Object (50 Fields)', () => {
    bench('zod', () => { megaZod.parse(megaData); });
    bench('rod (eager)', () => { megaRod.parse(megaData, { mode: 'eager' }); });
});

group('Computational Complexity (Complex Regex)', () => {
    const dataArray = Array(1000).fill(computeData);
    bench('zod', () => { dataArray.map(i => computeZod.parse(i)); });
    bench('rod (parseBatch)', () => { computeRod.parseBatch(dataArray); });
});


await run();