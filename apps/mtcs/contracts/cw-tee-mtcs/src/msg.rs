use std::collections::BTreeMap;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, HexBinary, Uint128, Uint64};
use quartz_common::contract::{
    msg::execute::attested::{RawAttested, RawAttestedMsgSansHandler, RawDefaultAttestation},
    prelude::*,
};

use crate::state::{LiquiditySource, RawHash, SettleOff};

pub type AttestedMsg<M, RA> = RawAttested<RawAttestedMsgSansHandler<M>, RA>;

#[cw_serde]
pub struct InstantiateMsg<RA = RawDefaultAttestation>(pub QuartzInstantiateMsg<RA>);

#[cw_serde]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg<RA = RawDefaultAttestation> {
    Quartz(QuartzExecuteMsg),
    FaucetMint(execute::FaucetMintMsg),
    Transfer(execute::Cw20Transfer),
    SubmitObligation(execute::SubmitObligationMsg),
    SubmitObligations(execute::SubmitObligationsMsg),
    SubmitSetoffs(AttestedMsg<execute::SubmitSetoffsMsg, RA>),
    InitClearing,
    SetLiquiditySources(execute::SetLiquiditySourcesMsg),
}

// TODO: Added this back here because adding overdraft contract as a dependency is causing errors. Overdraft isn't correctly disabling entrypoints when acting as a dependency
#[cw_serde]
pub enum OverdraftExecuteMsg {
    DrawCredit {
        receiver: Addr,
        amount: Uint128,
    },
    DrawCreditFromTender {
        debtor: Addr,
        amount: Uint128,
    },
    TransferCreditFromTender {
        sender: Addr,
        receiver: Addr,
        amount: Uint128,
    },
    IncreaseBalance {
        receiver: Addr,
        amount: Uint128,
    },
    DecreaseBalance {
        receiver: Addr,
        amount: Uint128,
    },
    Lock {},
    Unlock {},
    AddOwner {
        new: Addr,
    },
}

pub mod execute {
    use cosmwasm_std::Uint128;
    use quartz_common::contract::{msg::execute::attested::HasUserData, state::UserData};
    use sha2::{Digest, Sha256};

    use super::*;
    use crate::state::LiquiditySource;

    #[cw_serde]
    pub struct FaucetMintMsg {
        pub recipient: String,
        pub amount: u64,
    }

    #[cw_serde]
    pub struct Cw20Transfer {
        pub recipient: String,
        pub amount: u64,
    }

    #[cw_serde]
    pub struct SubmitObligationMsg {
        pub ciphertext: HexBinary,
        pub digest: HexBinary,
        // pub signatures: [HexBinary; 2],
        // pub proof: π
    }

    #[cw_serde]
    pub struct SubmitObligationsMsg {
        pub obligations: Vec<SubmitObligationMsg>,
        pub liquidity_sources: Vec<LiquiditySource>,
    }

    #[cw_serde]
    pub struct SubmitTenderMsg {
        pub ciphertext: HexBinary,
        pub digest: HexBinary,
        // pub proof: π
    }

    #[cw_serde]
    pub struct SubmitSetoffsMsg {
        pub setoffs_enc: BTreeMap<RawHash, SettleOff>,
        // pub proof: π,
    }

    impl HasUserData for SubmitSetoffsMsg {
        fn user_data(&self) -> UserData {
            let mut hasher = Sha256::new();
            hasher.update(serde_json::to_string(&self).expect("infallible serializer"));
            let digest: [u8; 32] = hasher.finalize().into();

            let mut user_data = [0u8; 64];
            user_data[0..32].copy_from_slice(&digest);
            user_data
        }
    }

    #[cw_serde]
    pub struct SetLiquiditySourcesMsg {
        pub liquidity_sources: Vec<LiquiditySource>,
    }

    #[cw_serde]
    pub enum EscrowExecuteMsg {
        ExecuteSetoff {
            payer: String,
            payee: String,
            amount: Vec<(String, Uint128)>,
        },
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetAllSetoffsResponse)]
    GetAllSetoffs,
    #[returns(GetLiquiditySourcesResponse)]
    GetLiquiditySources { epoch: Option<Uint64> }, // `None` means latest
    #[returns(cw20::BalanceResponse)]
    Balance { address: String },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetAllSetoffsResponse {
    pub setoffs: Vec<(HexBinary, SettleOff)>,
}

#[cw_serde]
pub struct GetLiquiditySourcesResponse {
    pub liquidity_sources: Vec<LiquiditySource>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_instantiate_msg() {
        let _: InstantiateMsg = serde_json::from_str(
            r#"{
                "msg": {
                    "config": {
                        "mr_enclave": "1bfb949d235f61e5dc40f874ba3e9c36adef1e7a521b4b5f70e10fb1dc803251",
                        "epoch_duration": {
                            "secs": 43200,
                            "nanos": 0
                        },
                        "light_client_opts": {
                            "chain_id": "testing",
                            "trusted_height": 1,
                            "trusted_hash": "a1d115ba3a5e9fcc12ed68a9d8669159e9085f6f96ec26619f5c7ceb4ee02869",
                            "trust_threshold": [
                                2,
                                3
                            ],
                            "trusting_period": 1209600,
                            "max_clock_drift": 5,
                            "max_block_lag": 5
                        }
                    }
                },
                "attestation": {
                    "report": {
                        "report": {
                            "id": "5246688123689513540899231107533660789",
                            "timestamp": "2024-02-07T17:06:23.913745",
                            "version": 4,
                            "epidPseudonym": "+CUyIi74LPqS6M0NF7YrSxLqPdX3MKs6D6LIPqRG/ZEB4WmxZVvxAJwdwg/0m9cYnUUQguLnJotthX645lAogfJgO8Xg5/91lSegwyUKvHmKgtjOHX/YTbVe/wmgWiBdaL+KmarY0Je459Px/FqGLWLsAF7egPAJRd1Xn88Znrs=",
                            "advisoryURL": "https://security-center.intel.com",
                            "advisoryIDs": [
                                "INTEL-SA-00161",
                                "INTEL-SA-00219",
                                "INTEL-SA-00289",
                                "INTEL-SA-00334",
                                "INTEL-SA-00615"
                            ],
                            "isvEnclaveQuoteStatus": "CONFIGURATION_AND_SW_HARDENING_NEEDED",
                            "platformInfoBlob": "150200650000080000141402040180070000000000000000000D00000C000000020000000000000CB0F08115F3DE71AE97980FE5E10B042054930ACE356C79EC44603D3F890756EC6ED73927A7C58CDE9AF1E754AEC77E335E8D80294407936BEB6404F27669FF7BB1",
                            "isvEnclaveQuoteBody": "AgABALAMAAAPAA8AAAAAAFHK9aSLRQ1iSu/jKG0xSJQAAAAAAAAAAAAAAAAAAAAAFBQCBwGAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABQAAAAAAAAAHAAAAAAAAAOPC8qW4QNieBprK/8rbZRDvhmpz06nuVxAO1fhkbuS7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAc8uUpEUEPvz8ZkFapjVh5WlWaLoAJM/f80T0EhGInHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACRE7C+d+1dDWhoDsdyBrjVh+1AZ5txMhzN1UBeTVSmggAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                        },
                        "reportsig": "YcY4SPvkfR4P2E8A5huutCeS+vY/ir+xq6disalNfNtAcUyOIOqTPVXhAZgY1M5B47Hjj1oYWf2qC2w+dnj7VcZjzO9oR0pJYdA+A7jaVrNzH2eXA79yICkuU8WE/x58I0j5vjXLoHXahaKlpZkMeTphqBY8u+FTVSdP3cWPho4viPapTfQRuEWmYq4KIq2zSr6wLg3Pz+yQ+G3e9BASVkLYxdYGTDFH1pMmfas9SEI7V4I+j8DaXmL8bucSRakmcQdmDMPGiA7mvIhSAlprzCrdxM7CHeUC6MPLN1fmFFcc9kyO/ved69j/651MWC83GgxSJ15L80U+DQzmrSW8xg=="
                    }
                }
            }"#,
        ).expect("failed to deserialize hardcoded quartz instantiate msg");
    }

    #[test]
    fn test_serde_execute_msg() {
        let _: ExecuteMsg = serde_json::from_str(
            r#"{
                "quartz": {
                    "session_create": {
                        "msg": {
                            "nonce": "425d87f8620e1dedeee70590cc55b164b8f01480ee59e0b1da35436a2f7c2777"
                        },
                        "attestation": {
                            "report": {
                                "report": {
                                    "id": "5246688123689513540899231107533660789",
                                    "timestamp": "2024-02-07T17:06:23.913745",
                                    "version": 4,
                                    "epidPseudonym": "+CUyIi74LPqS6M0NF7YrSxLqPdX3MKs6D6LIPqRG/ZEB4WmxZVvxAJwdwg/0m9cYnUUQguLnJotthX645lAogfJgO8Xg5/91lSegwyUKvHmKgtjOHX/YTbVe/wmgWiBdaL+KmarY0Je459Px/FqGLWLsAF7egPAJRd1Xn88Znrs=",
                                    "advisoryURL": "https://security-center.intel.com",
                                    "advisoryIDs": [
                                        "INTEL-SA-00161",
                                        "INTEL-SA-00219",
                                        "INTEL-SA-00289",
                                        "INTEL-SA-00334",
                                        "INTEL-SA-00615"
                                    ],
                                    "isvEnclaveQuoteStatus": "CONFIGURATION_AND_SW_HARDENING_NEEDED",
                                    "platformInfoBlob": "150200650000080000141402040180070000000000000000000D00000C000000020000000000000CB0F08115F3DE71AE97980FE5E10B042054930ACE356C79EC44603D3F890756EC6ED73927A7C58CDE9AF1E754AEC77E335E8D80294407936BEB6404F27669FF7BB1",
                                    "isvEnclaveQuoteBody": "AgABALAMAAAPAA8AAAAAAFHK9aSLRQ1iSu/jKG0xSJQAAAAAAAAAAAAAAAAAAAAAFBQCBwGAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABQAAAAAAAAAHAAAAAAAAAOPC8qW4QNieBprK/8rbZRDvhmpz06nuVxAO1fhkbuS7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAc8uUpEUEPvz8ZkFapjVh5WlWaLoAJM/f80T0EhGInHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACRE7C+d+1dDWhoDsdyBrjVh+1AZ5txMhzN1UBeTVSmggAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                                },
                                "reportsig": "YcY4SPvkfR4P2E8A5huutCeS+vY/ir+xq6disalNfNtAcUyOIOqTPVXhAZgY1M5B47Hjj1oYWf2qC2w+dnj7VcZjzO9oR0pJYdA+A7jaVrNzH2eXA79yICkuU8WE/x58I0j5vjXLoHXahaKlpZkMeTphqBY8u+FTVSdP3cWPho4viPapTfQRuEWmYq4KIq2zSr6wLg3Pz+yQ+G3e9BASVkLYxdYGTDFH1pMmfas9SEI7V4I+j8DaXmL8bucSRakmcQdmDMPGiA7mvIhSAlprzCrdxM7CHeUC6MPLN1fmFFcc9kyO/ved69j/651MWC83GgxSJ15L80U+DQzmrSW8xg=="
                            }
                        }
                    }
                }
            }"#,
        ).expect("failed to deserialize hardcoded quartz msg");
    }
}
