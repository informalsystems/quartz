// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import "./Quartz.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

contract Transfers is Quartz {
    using EnumerableSet for EnumerableSet.AddressSet;

    IERC20 public token;
    address public owner;

    struct Request {
        address user;
        uint256 amount; // unencrypted deposits and withdraws
        string action; // "deposit", "withdraw", "transfer"
        bytes32 encryptedEnclaveMsg; // an encrypted enclave msg
    }

    event Deposit(address indexed user, uint256 amount);
    event Withdraw(address indexed user, uint256 amount);
    event TransferRequest(address indexed sender, address indexed receiver, bytes32 ciphertext);
    event BalanceStored(address indexed user, bytes32 encryptedBalance);
    event StateUpdated(bytes32 newEncryptedState);

    mapping(address => bytes) public balances;
    Request[] public requests;
    bytes public encryptedState;

    /**
     * @notice Initializes the Transfers contract with the Quartz configuration and token address.
     * @param _config The configuration object for Quartz
     * @param _quote The attestation quote for Quartz setup
     * @param _token The ERC20 token used for deposits and withdrawals
     */
    constructor(Config memory _config, bytes memory _quote, address _token) Quartz(_config, _quote) {
        token = IERC20(_token);
        owner = msg.sender;
    }

    /**
     * @notice Deposits tokens to the contract and updates the sender's balance.
     * Emits a {Deposit} event.
     */
    function deposit(uint256 amount) external {
        require(token.transferFrom(msg.sender, address(this), amount), "Transfer failed");

        balances[msg.sender] += amount;
        
        emit Deposit(msg.sender, amount);
        
        requests.push(Request(msg.sender, amount, "deposit", bytes32(0)));
    }

    /**
     * @notice Withdraws tokens from the caller's balance.
     * Emits a {Withdraw} event.
     */
    function withdraw(uint256 amount) external {
        require(balances[msg.sender] >= amount, "Insufficient balance");

        balances[msg.sender] -= amount;
        require(token.transfer(msg.sender, amount), "Transfer failed");

        emit Withdraw(msg.sender, amount);
        
        requests.push(Request(msg.sender, amount, "withdraw", bytes32(0)));
    }

    /**
     * @notice Requests a transfer by storing an encrypted balance and emits a transfer event.
     * @dev Only enclave can authorize this transfer with a valid quote.
     * @param receiver The recipient address of the transfer
     * @param ciphertext The encrypted transfer data
     * @param quote The attestation quote for enclave verification
     */
    function transferRequest(address receiver, bytes32 ciphertext, bytes memory quote) external {
        emit TransferRequest(msg.sender, receiver, ciphertext);
        requests.push(Request(msg.sender, 0, "transfer", ciphertext));
    }

    /**
     * @notice Updates the contract state with a new encrypted state, clears requests, and processes withdrawals.
     * @dev Only enclave can call this function
     * @param newEncryptedState The new encrypted state to be stored.
     * @param withdrawalAddresses The list of addresses requesting withdrawals.
     * @param withdrawalAmounts The corresponding list of withdrawal amounts for each address.
     * @param quote The attestation quote for enclave verification.
     */
    function update(
        bytes32 newEncryptedState,
        address[] calldata withdrawalAddresses,
        uint256[] calldata withdrawalAmounts,
        bytes memory quote
    ) external onlyEnclave(quote) {
        require(withdrawalAddresses.length == withdrawalAmounts.length, "Mismatched withdrawals");

        // 1. Store the new encrypted state
        encryptedState = newEncryptedState;
        emit StateUpdated(newEncryptedState);

        // 2. Clear stored requests
        delete requests;

        // 3. Process each withdrawal
        for (uint256 i = 0; i < withdrawalAddresses.length; i++) {
            address user = withdrawalAddresses[i];
            uint256 amount = withdrawalAmounts[i];

            require(token.transfer(user, amount), "Transfer failed");
            emit Withdraw(user, amount);
        }
    }

    /**
     * @notice User calls this to notify the enclave they want their state updated in the contract
     * @param ephemeralPubley The pubkey used to decrypt the stored balance
     */
    function queryEncryptedBalance(bytes ephemeralPubley) public {
        emit QueryBalance(msg.sender, ephemeralPubley);
    }

    /**
     * @notice Stores an encrypted balance for a user, restricted to enclave calls.
     * Emits a {BalanceStored} event.
     * @param user The address of the user whose balance is being stored
     * @param encryptedBalance The encrypted balance data
     * @param quote The attestation quote for enclave verification
     */
    function storeEncryptedBalance(address user, bytes32 encryptedBalance, bytes memory quote) external onlyEnclave(quote) {
        balances[user] = encryptedBalance;
        emit BalanceStored(user, encryptedBalance);
    }
}
