#!/bin/bash
# chaos.sh - Generates violations to test Warden

# Use a directory name that Warden won't ignore by default
TARGET_DIR="chaos_examples"
rm -rf "$TARGET_DIR"
mkdir -p "$TARGET_DIR"

echo "⚡ Generating chaos in $TARGET_DIR/..."

# 1. HUGE FILE (Law of Atomicity)
echo "// This file is too big" > "$TARGET_DIR/bloat.rs"
# Generate 2500 lines to ensure it breaks the 2000 token limit
for i in {1..2500}; do
    echo "fn function_$i() { println!(\"waste of space\"); }" >> "$TARGET_DIR/bloat.rs"
done

# 2. DEEP NESTING (Law of Complexity)
# Nesting level 5 (Limit is 3)
cat <<EOF > "$TARGET_DIR/deep.rs"
fn deep_trouble() {
    if true {
        if true {
            if true {
                if true {
                    if true {
                        println!("Help me");
                    }
                }
            }
        }
    }
}
EOF

# 3. COMPLEXITY (Law of Complexity)
# Cyclomatic complexity 9 (Limit is 8)
cat <<EOF > "$TARGET_DIR/complex.rs"
fn complex_logic(x: i32) {
    if x == 1 { println!("1"); }
    else if x == 2 { println!("2"); }
    else if x == 3 { println!("3"); }
    else if x == 4 { println!("4"); }
    else if x == 5 { println!("5"); }
    else if x == 6 { println!("6"); }
    else if x == 7 { println!("7"); }
    else if x == 8 { println!("8"); }
    else { println!("9"); }
}
EOF

# 4. BANNED FUNCTIONS (Law of Paranoia)
cat <<EOF > "$TARGET_DIR/unsafe.rs"
fn risky_business() {
    let x: Option<i32> = None;
    x.unwrap();                 // Violation
    x.expect("Crash");          // Violation
}
EOF

echo "😈 Chaos generated."
echo "Run 'warden' now!"
