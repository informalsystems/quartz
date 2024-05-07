use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    obligato_client::Client,
    types::{ObligatoObligation, ObligatoSetOff},
    BANK_DEBTOR_ID,
};

pub struct MockClient;

#[async_trait]
impl Client for MockClient {
    type Error = ();

    async fn get_obligations(&self) -> Result<Vec<ObligatoObligation>, Self::Error> {
        Ok(vec![
            // obligation: 1 --10--> 2
            ObligatoObligation {
                id: Uuid::from_u128(1),
                debtor_id: Uuid::from_u128(1),
                creditor_id: Uuid::from_u128(2),
                amount: 10,
            },
            // tender: $ --10--> 1
            ObligatoObligation {
                id: Uuid::from_u128(2),
                debtor_id: Uuid::parse_str(BANK_DEBTOR_ID).unwrap(),
                creditor_id: Uuid::from_u128(1),
                amount: 10,
            },
        ])
    }

    async fn set_setoffs(&self, setoffs: Vec<ObligatoSetOff>) -> Result<(), Self::Error> {
        println!("{:?}", setoffs);
        Ok(())
    }
}
