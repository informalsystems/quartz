pub trait ConstPrefix {
    const PREFIX: &'static str;
}

#[derive(Clone, Debug)]
pub struct PrefixWasm;

impl ConstPrefix for PrefixWasm {
    const PREFIX: &'static str = "neutron";
}
