const { rod } = require('../dist/index-node.js');

async function runTest() {
    console.log("🚀 Starting Rod.js Node.js Test...");

    // Init WASM
    await rod.init();

    const UserSchema = rod.object({
        name: rod.string().min(3),
        age: rod.number().int().min(18),
        email: rod.string().email(),
    }).strict();

    // --- Test Case 1: Valid Data ---
    const validUser = {
        name: "Rod User",
        age: 30,
        email: "test@rod.rs"
    };
    console.log("\n🧪 Testing with valid data...");
    try {
        const parsedData = UserSchema.parse(validUser);
        console.log("✅ Validation Passed (Lazy Mode):", parsedData);

        const parsedDataEager = UserSchema.parse(validUser, { mode: 'eager' });
        console.log("✅ Validation Passed (Eager Mode):", parsedDataEager);
    } catch (e) {
        console.error("❌ Test Failed! Should have passed.", e);
        process.exit(1);
    }

    // --- Test Case 2: Invalid Data ---
    const invalidUser = {
        name: "R", // Too short
        age: 17.5, // Not an integer and too young
        extraField: true // FIX: lowercase 'true'
    };
    console.log("\n🧪 Testing with invalid data...");
    const result = UserSchema.safeParse(invalidUser);
    if (!result.success) {
        console.log("✅ Validation Correctly Failed. Errors:");
        console.log(JSON.stringify(result.error, null, 2));
    } else {
        console.error("❌ Test Failed! Should have failed.");
        process.exit(1);
    }

    console.log("\n🎉 JS Integration Test Complete.");
}

runTest().catch(err => {
    console.error(err);
    process.exit(1);
});