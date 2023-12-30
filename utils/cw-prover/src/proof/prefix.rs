pub struct PrefixWasm;

pub trait ConstPrefix {
    const PREFIX: &'static str;
}

impl ConstPrefix for PrefixWasm {
    const PREFIX: &'static str = "wasm";
}
