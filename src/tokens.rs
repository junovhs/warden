use std::sync::LazyLock;
use tiktoken_rs::CoreBPE;

// We use cl100k_base (GPT-4/3.5 turbo encoding) as the standard
static BPE: LazyLock<CoreBPE> = LazyLock::new(|| tiktoken_rs::cl100k_base().unwrap());

pub struct Tokenizer;

impl Tokenizer {
    #[must_use]
    pub fn count(text: &str) -> usize {
        // EncodeOrdinary is faster as it ignores special tokens, which is fine for code
        BPE.encode_ordinary(text).len()
    }

    /// Returns true if the file exceeds the token limit
    #[must_use]
    pub fn exceeds_limit(text: &str, limit: usize) -> bool {
        Self::count(text) > limit
    }
}
