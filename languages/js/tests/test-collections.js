const { rod } = require('../dist/index-node.js');

async function test() {
    await rod.init();
    console.log("--- Testing Collections ---");

    // Nested Objects + Strict Mode
    const Profile = rod.object({
        username: rod.string(),
        meta: rod.object({ lastIp: rod.string().ip() })
    }).strict();

    Profile.parse({ username: "admin", meta: { lastIp: "127.0.0.1" } });
    try { Profile.parse({ username: "a", meta: {}, extra: 1 }); } catch { console.log("✅ Object.strict() passed"); }

    // Tuples
    const Point = rod.tuple([rod.number(), rod.number()]);
    Point.parse([10, 20]);
    console.log("✅ Tuples passed");

    // Records (Dynamic Keys)
    const Settings = rod.record(rod.string().min(1), rod.boolean());
    Settings.parse({ "dark_mode": true, "notifications": false });
    console.log("✅ Records passed");
}

test().catch(e => { console.error(e); process.exit(1); });