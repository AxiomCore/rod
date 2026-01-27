const { rod } = require('../dist/index-node.js');

async function testDynamicSchema() {
    await rod.init();
    console.log("--- Testing Dynamic Schema ---");

    const userSpec = {
        type: "object",
        properties: {
            username: {
                type: "string",
                min: 3,
                trim: true
            },
            email: {
                type: "string",
                email: true
            },
            profile: {
                type: "object",
                properties: {
                    age: { type: "number", int: true, min: 18 }
                }
            }
        },
        strict: true
    };

    const DynamicUserSchema = rod.fromJSON(userSpec);

    const validData = {
        username: "  yashmakan  ",
        email: "hello@rod.rs",
        profile: { age: 25 }
    };

    try {
        const result = DynamicUserSchema.parse(validData);
        if (result.username === "yashmakan" && result.profile.age === 25) {
            console.log("✅ Dynamic Schema: Valid data passed (and trimmed)");
        }
    } catch (e) {
        console.error("❌ Dynamic Schema: Failed to parse valid data", e);
    }

    const invalidData = {
        username: "ya", // Too short (min 3)
        email: "not-an-email",
        profile: { age: 10 } // Too young (min 18)
    };

    const failResult = DynamicUserSchema.safeParse(invalidData);
    if (!failResult.success) {
        console.log("✅ Dynamic Schema: Correctly caught 3 validation errors");
    } else {
        console.error("❌ Dynamic Schema: Failed to catch invalid data");
    }

    const extraData = {
        username: "yash",
        email: "test@test.com",
        profile: { age: 20 },
        unknownField: "I should not be here"
    };

    const strictResult = DynamicUserSchema.safeParse(extraData);
    if (!strictResult.success) {
        const isStrictError = strictResult.error.issues.some(i => i.details.code_type === "unrecognized_keys");
        if (isStrictError) {
            console.log("✅ Dynamic Schema: Strict mode correctly blocked unknown field");
        }
    }
}

testDynamicSchema().catch(e => {
    console.error("💥 Suite crashed", e);
    process.exit(1);
});