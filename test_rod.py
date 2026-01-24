import rod
import json

def run_test():
    print("🚀 Starting Rod Python Integration Test...")

    # 1. Define a complex schema using the Zod-like API
    UserSchema = rod.object({
        "name": rod.string().min(3).trim(),
        "age": rod.number().int().min(18),
        "email": rod.string().email(),
        "tags": rod.array(rod.string()).min(1),
        "role": rod.enum("admin", "user", "guest")
    }).strict()

    # --- Test Case 1: Valid Data ---
    valid_user = {
        "name": "  Yash Makan  ", # Will be trimmed by Rust
        "age": 25,
        "email": "yash@axiom.rs",
        "tags": ["rust", "python", "ffi"],
        "role": "admin"
    }

    print("\n🧪 Testing with valid data...")
    try:
        parsed = UserSchema.parse(valid_user)
        # Check if trimming worked
        assert parsed["name"] == "Yash Makan"
        print(f"✅ Validation Passed! Result:")
        print(json.dumps(parsed, indent=2))
    except ValueError as e:
        print(f"❌ Test Failed! Should have passed. Error: {e}")

    # --- Test Case 2: Invalid Data ---
    invalid_user = {
        "name": "Ya",        # Too short (min 3)
        "age": 17,           # Too young (min 18)
        "email": "not-email", # Invalid format
        "tags": [],          # Too few items (min 1)
        "role": "super-man", # Not in enum
        "extra": "intruder"  # Fails strict mode
    }

    print("\n🧪 Testing with invalid data...")
    result = UserSchema.safe_parse(invalid_user)
    
    if not result["success"]:
        print("✅ Validation Correctly Failed. Errors found:")
        # The error is a structured dict from RodError
        print(json.dumps(result["error"], indent=2))
    else:
        print("❌ Test Failed! Should have failed validation.")

    print("\n🎉 All tests completed successfully!")

if __name__ == "__main__":
    run_test()