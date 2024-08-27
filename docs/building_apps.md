# Building Applications

Quartz application developers need to write code for two kinds of environments: smart contracts and TEEs. 

For now, smart contract code is written in CosmWasm Rust and deployed on Cosmos-SDK chains that support CosmWasm.
TEE code is written in Rust and compiled via Gramine to run on Intel SGX enclaves.

App devs need to design their smart contracts and their enclave code in tandem to work together. 
Note that enclave code is not restricted to be CosmWasm as well, but can be (practically) arbitrary Rust.

## Enclave Code

... 

## Smart Contract Code

The logic of a Quartz smart contract is divided roughly between the following domains:

- public business logic - normal public business logic you would write if you weren't using Quartz, or for things that can't be done privately (eg. transfers from native non-private accounts)
- private business logic - this is business logic that executes in the TEE on data that is encrypted on-chain.
- public control logic - this is the logic that controls the enclave - who runs it, what code it runs, when it runs, etc.



Each of these three logic written by application developers in the smart contract. Only the private business logic additionally requires app devs to write code that wil

## Public Business Logic

Certain parts of contract logic will always remain independent of Quartz and written as normal in the given smart contract environment. 
This might include certain deployment, ownership, or contract upgrade logic. 
It could be certain governance or administrative logic that is intended to be public regardless. 
Or it could be logic that cannot be handled via Quartz because it involves interaction with components of the blockchain that are already outside the Quartz environment.
The most common example of this is existing transparent account balances, which can only be brought into a Quartz enabled smart contract 
in the first place via transparent on-chain logic. That said, these actions may trigger storage updates in the contract which are expected to be read by the enclave.

## Private Business Logic

This is the core business logic that must be executed privately. It comprises roughly 3 components:

- on-chain queieing of private inputs
- off-chain execution of code in an enclave
- on-chain processing of results


## Public Control Logic
