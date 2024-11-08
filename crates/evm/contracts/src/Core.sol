// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

contract Core {
    Config public config;
    bytes32 public enclavePublicKey;
    uint256 public sequenceNumber;
    // nonce is no longer needed
    // TODO - is session and session key needed?

    struct Config {
        bytes32 mrEnclave;
        LightClientOpts lightClientOpts;
        address pccs; // is both the tcbinfo_contract and dcap_verifier_contract
    }

    struct LightClientOpts {
        string chainID;
        uint256 trustedHeight;
        bytes32 trustedHash;
        // etc.
    } // TODO Shoaib to figure out real values

    // TODO - must take the quote and the collateral from the enclave
    // i.e. the enclave calls this (I guess it has the ETH to pay gas)
    // the enclave wants to see these are stored
    // .... maybe it just checks the quoote, and not the mr enclave (yet)
    constructor(Config memory _config) {
        config = _config;
    }

    // TODO - import pcss
    modifier onlyEnclave(bytes memory quote) {
        require(quote == config.pccs.verifyQuote(), "Only enclave can call this function");
        _;
    }

    // needs to verify the handshake, and store the enclave public key
    // TODO - restrict to onlyEnclave 
    // Note - without the nonce, the entire sessoin just becomes teh pub key. I chould rename to storeSession
    function storePubKey(bytes32 _enclavePublicKey, bytes32 quote) public onlyEnclave(quote) {
        enclavePublicKey = _enclavePublicKey;
    }

    // TODO - set config in the next envlace handshake msg

    // TODO - might also need session create, on top of session set pub key

    // TODO - recreate message types from all rust to here (they are pretty simple messages)

    // TODO - all the function calls and the instantiate call needs to emit events, so the host can listen to them
    // and communicate back and forth between the enclave 

}

