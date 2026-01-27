const { rod } = require('../dist/index-node.js');

async function test() {
    await rod.init();
    console.log("--- Testing Primitives ---");

    const UserSchema = rod.object({
        email: rod.string().email().trim(),
        id: rod.string().uuid()
    });
    const user = UserSchema.parse({ email: "  test@rod.rs  ", id: "550e8400-e29b-41d4-a716-446655440000" });
    if (user.email !== "test@rod.rs") throw new Error("Trim failed");
    console.log("✅ Strings passed");

    const score = rod.number().int().min(0).max(100);
    score.parse(50);
    try { score.parse(50.5); } catch { console.log("✅ Number.int() passed"); }

    const birthDate = rod.date().min(0);
    birthDate.parse(Date.now());
    console.log("✅ Dates passed");
}

test().catch(e => { console.error(e); process.exit(1); });