#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::InstantiateMsg;
    use cosmwasm_std::testing::MockApi;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    const USER: &str = "USER";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "denom";
    const TCB_SIGNER: &str = include_str!("../data/tcb_signer.pem");
    const ROOT_CA: &str = include_str!("../data/root_ca.pem");
    const TCB_INFO: &str = include_str!("../data/tcbinfo.json");
    const TIME: &str = "2024-07-11T15:19:13Z";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &MockApi::default().addr_make(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1),
                    }],
                )
                .unwrap();
        })
    }
    fn proper_instantiate() -> (App, CwTemplateContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());

        let user = app.api().addr_make(USER);
        assert_eq!(
            app.wrap().query_balance(user, NATIVE_DENOM).unwrap().amount,
            Uint128::new(1)
        );

        let msg = InstantiateMsg {
            root: ROOT_CA.to_string(),
        };
        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        let cw_template_contract = CwTemplateContract(cw_template_contract_addr);

        (app, cw_template_contract)
    }

    mod add_tcbinfo {
        use super::*;
        use crate::msg::ExecuteMsg;

        #[test]
        fn add_tcbinfo() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let msg = ExecuteMsg {
                tcb_info: TCB_INFO.to_string(),
                certificate: TCB_SIGNER.to_string(),
                time: TIME.to_string(),
            };
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
        }
    }
}
