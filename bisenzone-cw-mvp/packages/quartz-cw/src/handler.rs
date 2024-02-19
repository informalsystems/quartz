pub mod execute;
pub mod instantiate;

use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::error::Error;
use crate::msg::HasDomainType;

pub trait Handler {
    fn handle(self, deps: DepsMut<'_>, env: &Env, info: &MessageInfo) -> Result<Response, Error>;
}

pub trait RawHandler: HasDomainType {
    fn handle_raw(
        self,
        deps: DepsMut<'_>,
        env: &Env,
        info: &MessageInfo,
    ) -> Result<Response, Error>;
}

impl<RM> RawHandler for RM
where
    RM: HasDomainType,
    RM::DomainType: Handler,
{
    fn handle_raw(
        self,
        deps: DepsMut<'_>,
        env: &Env,
        info: &MessageInfo,
    ) -> Result<Response, Error> {
        let execute: RM::DomainType = self.try_into()?;
        execute.handle(deps, env, info)
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::DepsMut;
    use serde::Deserialize;

    use crate::handler::Handler;
    use crate::msg::{HasDomainType, RawExecuteMsg, RawInstantiateMsg};
    use crate::state::SESSION;

    fn parse_msg<'a, R>(msg_str: &'a str) -> R::DomainType
    where
        R: HasDomainType + Deserialize<'a>,
    {
        let raw_msg: R =
            serde_json::from_str(msg_str).expect("deserialization failure for hard-coded RawMsg");
        raw_msg.try_into().expect("invalid hard-coded RawMsg")
    }

    fn handle_msg<'a, R>(mut deps: DepsMut<'_>, msg_str: &'a str)
    where
        R: HasDomainType + Deserialize<'a>,
        R::DomainType: Handler,
    {
        let msg = parse_msg::<R>(msg_str);
        let info = mock_info("creator", &[]);
        let env = mock_env();
        let res = msg
            .handle(deps.branch(), &env, &info)
            .expect("msg handler failure");

        assert_eq!(0, res.messages.len());
    }

    fn instantiate(deps: DepsMut<'_>) {
        handle_msg::<RawInstantiateMsg>(
            deps,
            r#"{
                "msg": {
                    "mr_enclave": "e3c2f2a5b840d89e069acaffcadb6510ef866a73d3a9ee57100ed5f8646ee4bb"
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
                    },
                    "mr_enclave": "e3c2f2a5b840d89e069acaffcadb6510ef866a73d3a9ee57100ed5f8646ee4bb",
                    "user_data": "9113b0be77ed5d0d68680ec77206b8d587ed40679b71321ccdd5405e4d54a6820000000000000000000000000000000000000000000000000000000000000000"
                }
            }"#,
        );
    }

    fn session_create(deps: DepsMut<'_>) {
        handle_msg::<RawExecuteMsg>(
            deps,
            r#"{
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
                        },
                        "mr_enclave": "e3c2f2a5b840d89e069acaffcadb6510ef866a73d3a9ee57100ed5f8646ee4bb",
                        "user_data": "425d87f8620e1dedeee70590cc55b164b8f01480ee59e0b1da35436a2f7c27770000000000000000000000000000000000000000000000000000000000000000"
                    }
                }
            }"#,
        );
    }

    fn session_set_pub_key(deps: DepsMut<'_>) {
        handle_msg::<RawExecuteMsg>(
            deps,
            r#"{
                "session_set_pub_key": {
                    "msg": {
                        "nonce": "425d87f8620e1dedeee70590cc55b164b8f01480ee59e0b1da35436a2f7c2777"
                        "pub_key": "03E67EF09213633074FB4FBF338643F4F0C574ED60EF11D03422EEB06FA38C8F3F"
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
                        },
                        "mr_enclave": "e3c2f2a5b840d89e069acaffcadb6510ef866a73d3a9ee57100ed5f8646ee4bb",
                        "user_data": "425d87f8620e1dedeee70590cc55b164b8f01480ee59e0b1da35436a2f7c27770000000000000000000000000000000000000000000000000000000000000000"
                    }
                }
            }"#,
        );
    }

    #[test]
    #[ignore]
    fn test_instantiate_handler() {
        let mut deps = mock_dependencies();
        instantiate(deps.as_mut());
    }

    #[test]
    #[ignore]
    fn test_session_create_handler() {
        let mut deps = mock_dependencies();
        instantiate(deps.as_mut());
        session_create(deps.as_mut());
        SESSION.load(&deps.storage).expect("Session not created");
    }

    #[test]
    #[ignore]
    fn test_session_set_pub_key_handler() {
        let mut deps = mock_dependencies();
        instantiate(deps.as_mut());
        session_create(deps.as_mut());
        session_set_pub_key(deps.as_mut());
        SESSION.load(&deps.storage).expect("Session not created");
        // TODO(hu55a1n1): check that nonce & pub_key match, etc.
    }
}
