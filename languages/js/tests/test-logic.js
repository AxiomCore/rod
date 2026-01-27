const { rod } = require('../dist/index-node.js');

async function test() {
    await rod.init();
    console.log("--- Testing Logic Types ---");

    // Literals
    const version = rod.literal("v1");
    version.parse("v1");
    console.log("✅ Literals passed");

    // Enums
    const Role = rod.enum(["ADMIN", "USER"]);
    Role.parse("ADMIN");
    console.log("✅ Enums passed");

    // Discriminated Union (Polymorphism)
    const Circle = rod.object({ type: rod.literal("circle"), r: rod.number() });
    const Square = rod.object({ type: rod.literal("square"), s: rod.number() });
    const Shape = rod.discriminatedUnion("type", [Circle, Square]);

    Shape.parse({ type: "circle", r: 5 });
    Shape.parse({ type: "square", s: 10 });
    console.log("✅ Discriminated Unions passed");
}

test().catch(e => { console.error(e); process.exit(1); });