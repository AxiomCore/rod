const { rod } = require('../dist/index-node.js');

async function test() {
    await rod.init();
    console.log("--- Testing Dates ---");

    const cutoff = new Date("2006-01-01").getTime();

    const AgeSchema = rod.object({
        birthday: rod.date().max(cutoff)
    });

    // Person born in 2020 (Younger than 18) -> Should Fail
    const child = { birthday: new Date("2020-01-01").getTime() };
    const result = AgeSchema.safeParse(child);

    if (!result.success) {
        console.log("✅ Successfully blocked child born in 2020");
    } else {
        console.log("❌ Failed: Allowed a child to pass!");
    }
}

test();