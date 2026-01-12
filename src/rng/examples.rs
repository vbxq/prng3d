pub struct RngExample {
    pub name: &'static str,
    pub description: &'static str,
    pub code: &'static str,
}

pub const RNG_EXAMPLES: &[RngExample] = &[
    RngExample {
        name: "Xorshift32",
        description: "Marsaglia's classic, fast and high-quality.",
        code: r#"fn rng(state: int) -> int {
    let x = state ^ (state << 13)
    let y = x ^ (x >> 17)
    let z = y ^ (y << 5)
    z & 0x7FFFFFFF
}"#,
    },
    RngExample {
        name: "LCG MINSTD",
        description: "Park-Miller standard, acceptable for simple uses.",
        code: r#"fn rng(state: int) -> int {
    let next = (state * 48271) % 2147483647
    if next == 0 { 1 } else { next }
}"#,
    },
    RngExample {
        name: "LCG Numerical Recipes",
        description: "From Numerical Recipes, common but flawed.",
        code: r#"fn rng(state: int) -> int {
    (state * 1103515245 + 12345) & 0x7FFFFFFF
}"#,
    },
    RngExample {
        name: "RANDU (Bad)",
        description: "IBM 1968. Famous for visible 3D hyperplanes.",
        code: r#"fn rng(state: int) -> int {
    let next = (state * 65539) % 2147483648
    if next == 0 { 1 } else { next }
}"#,
    },
    RngExample {
        name: "Counter (Worst)",
        description: "Just increments. Perfect diagonal line in 3D.",
        code: r#"fn rng(state: int) -> int {
    state + 1
}"#,
    },
    RngExample {
        name: "Multiply-3 (Awful)",
        description: "Tiny multiplier, visible patterns.",
        code: r#"fn rng(state: int) -> int {
    (state * 3) % 2147483648
}"#,
    },
];
