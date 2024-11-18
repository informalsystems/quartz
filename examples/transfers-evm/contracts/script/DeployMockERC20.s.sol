// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../test/MockERC20.sol";

contract DeployMockERC20 is Script {
    function run() external {
        // Specify the token name and symbol
        string memory tokenName = "Cycles";
        string memory tokenSymbol = "CYC";

        // Load the mint address from .env
        address mintAddress = vm.envAddress("CYC_MINT_ADDRESS");

        vm.startBroadcast();

        // Deploy CYC contract
        MockERC20 mockToken = new MockERC20(tokenName, tokenSymbol);

        // Mint 1 billion tokens to the specified mint address
        mockToken.mint(mintAddress, 1000000000 ether);

        console.log("CYC deployed at:", address(mockToken));

        vm.stopBroadcast();
    }
}
