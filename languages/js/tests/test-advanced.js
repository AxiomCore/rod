const { rod } = require('../dist/index-node.js');

async function test() {
    await rod.init();
    console.log("--- Testing Advanced Features ---");

    const schema = rod.string().email();
    const result = schema.safeParse("not-an-email");
    if (!result.success) {
        console.log("✅ safeParse returned error structure");
    }

    const eagerSchema = rod.object({ a: rod.number() });
    const eagerResult = eagerSchema.parse({ a: 1 }, { mode: 'eager' });
    if (eagerResult.a === 1) console.log("✅ Eager mode passed");

    const numberArr = rod.number().min(10);
    const inputs = [5, 15, 20, 2];
    const mask = numberArr.checkBatch(inputs);

    if (mask[0] === 0 && mask[1] === 1) {
        console.log("✅ checkBatch (Bitmask) passed");
    }
}

test().catch(e => { console.error(e); process.exit(1); });