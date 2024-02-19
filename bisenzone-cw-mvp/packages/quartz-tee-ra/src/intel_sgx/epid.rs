use hex_literal::hex;
use thiserror::Error;

pub const INTEL_ROOT_MODULUS: &[u8] = &hex!("a97a2de0e66ea6147c9ee745ac0162686c7192099afc4b3f040fad6de093511d74e802f510d716038157dcaf84f4104bd3fed7e6b8f99c8817fd1ff5b9b864296c3d81fa8f1b729e02d21d72ffee4ced725efe74bea68fbc4d4244286fcdd4bf64406a439a15bcb4cf67754489c423972b4a80df5c2e7c5bc2dbaf2d42bb7b244f7c95bf92c75d3b33fc5410678a89589d1083da3acc459f2704cd99598c275e7c1878e00757e5bdb4e840226c11c0a17ff79c80b15c1ddb5af21cc2417061fbd2a2da819ed3b72b7efaa3bfebe2805c9b8ac19aa346512d484cfc81941e15f55881cc127e8f7aa12300cd5afb5742fa1d20cb467a5beb1c666cf76a368978b5");

pub const INTEL_ROOT_EXPONENT: &[u8] =
    &hex!("0000000000000000000000000000000000000000000000000000000000010001");

pub mod types;

pub mod verifier;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Recovered digest from signature does not match the specified report")]
    RecoveredDigestMismatch,
}
