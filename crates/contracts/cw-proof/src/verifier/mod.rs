pub mod cw;
pub mod ics23;
pub mod multi;

trait Verifier {
    type Proof;
    type Root: Eq;
    type Key;
    type Value;
    type Error;

    fn verify(
        &self,
        proof: &Self::Proof,
        key: &Self::Key,
        value: &Self::Value,
    ) -> Result<Self::Root, Self::Error>;

    fn verify_against_root(
        &self,
        proof: &Self::Proof,
        key: &Self::Key,
        value: &Self::Value,
        root: &Self::Root,
    ) -> Result<bool, Self::Error> {
        let found_root = self.verify(proof, key, value)?;
        Ok(root == &found_root)
    }
}
