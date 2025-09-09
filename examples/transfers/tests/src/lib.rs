use anyhow::Result;
use cosmwasm_std::{Addr, Coin};
use cw_multi_test::{App, AppResponse, ContractWrapper, Executor};
use quartz_common::{
    contract::{
        msg::{
            execute::attested::{Attested, MockAttestation, RawMockAttestation},
            instantiate::CoreInstantiate,
        },
        state::{Config, LightClientOpts, MrEnclave},
    },
    enclave::attestor::{Attestor, MockAttestor},
};
use transfers_contract::{
    contract::{execute, instantiate, query},
    msg::{execute::Request, ExecuteMsg, InstantiateMsg, QueryMsg},
};

#[cfg(test)]
mod mock_tests;

#[cfg(test)]
mod multi_tests;

pub struct Context {
    app: App,
    contract_addr: Addr,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        // init app
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("user1"),
                    vec![Coin::new(1000000u128, "ucosm")],
                )
                .unwrap();
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("user2"),
                    vec![Coin::new(1000000u128, "ucosm")],
                )
                .unwrap();
        });
        let contract = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(contract));

        // quartz config
        let mr_enclave = MrEnclave::default();
        let light_client_opts = LightClientOpts::new(
            "testing".to_string(),
            20148,
            [
                192, 50, 26, 51, 185, 223, 21, 42, 49, 53, 232, 74, 4, 184, 42, 88, 235, 236, 15,
                97, 171, 19, 8, 40, 39, 184, 36, 126, 21, 68, 240, 101,
            ]
            .to_vec()
            .try_into()
            .unwrap(),
            (2, 3),
            1209600,
            5,
            5,
        )
        .unwrap();
        let config = Config::new(mr_enclave, light_client_opts, None, None);

        // instantiate
        let msg = CoreInstantiate::new(config);
        let attestation = <MockAttestor as Attestor>::Attestation::from(
            MockAttestor.attestation(msg.clone()).unwrap(),
        );
        let attested_msg: Attested<CoreInstantiate, MockAttestation> =
            Attested::new(msg, attestation);
        let instantiate: quartz_common::contract::msg::InstantiateMsg<MockAttestation> =
            quartz_common::contract::msg::instantiate::Instantiate(attested_msg);
        let msg = InstantiateMsg::<RawMockAttestation> {
            quartz: instantiate.into(),
            denom: "ucosm".to_string(),
        };
        let contract_addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &msg,
                &[],
                "Quartz transfers contract",
                None,
            )
            .unwrap();

        Context { app, contract_addr }
    }

    pub fn execute_msg(
        &mut self,
        sender: Addr,
        msg: &ExecuteMsg,
        funds: &[Coin],
    ) -> Result<AppResponse> {
        self.app
            .execute_contract(sender, self.contract_addr.clone(), msg, funds)
    }

    pub fn query_requests(&self) -> Result<Vec<Request>> {
        let result = self
            .app
            .wrap()
            .query_wasm_smart(self.contract_addr.clone(), &QueryMsg::GetRequests {})?;
        Ok(result)
    }
}
