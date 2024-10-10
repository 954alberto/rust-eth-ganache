use std::{iter::Filter, time::Duration};

use ethers::{
    prelude::{Address, LocalWallet, Middleware, Provider, Signer, TransactionRequest, U256},
    signers::coins_bip39::mnemonic,
    utils::Ganache,
};
use eyre::{ContextCompat, Result};
use hex::ToHex;

/// Main asynchronous function which sets up a local blockchain using Ganache,
/// queries balances, and sends a transaction from one account to another.
#[tokio::main]
async fn main() -> Result<()> {
    // Define a mnemonic for a wallet (used to derive private keys)
    let mnemonic = "gas monster ski craft below illegal discover limit dog bundle bus artefact";

    // Create and launch a Ganache instance (local Ethereum test blockchain) with the mnemonic
    let ganache = Ganache::new().mnemonic(mnemonic).spawn();
    println!("HTTP Endpoint: {}", ganache.endpoint()); // Print the HTTP endpoint for Ganache

    // Create a local wallet from the first generated key from Ganache
    let wallet: LocalWallet = ganache.keys()[0].clone().into();
    let first_address = wallet.address(); // Extract the first address from the wallet
    println!(
        "wallet first address: {}",
        first_address.encode_hex::<String>() // Encode the address to a hexadecimal string
    );

    // Connect to the Ganache provider using the Ganache endpoint, set polling interval to 10ms
    let provider = Provider::try_from(ganache.endpoint())?.interval(Duration::from_millis(10));

    // Query and print the balance of the wallet's first address
    let first_balance = provider.get_balance(first_address, None).await?;
    println!("wallet first address balance: {}", first_balance); // Display the balance

    // Query the balance of a random Ethereum address (external to this wallet)
    let other_address_hex = "0xB794F5eA0ba39494cE839613fffBA74279579268"; // Random address in hex format
    let other_address = other_address_hex.parse::<Address>()?; // Parse the hex string into an Address type
    let other_balance = provider.get_balance(other_address, None).await?; // Get the balance of the random address
    println!(
        "Balance for address {}: {}",
        other_address_hex,
        other_balance // Display the balance
    );

    // Create a transaction request to send 1000 units of Wei (smallest denomination of Ether)
    // from the wallet's first address to the random address
    let tx = TransactionRequest::pay(other_address, U256::from(1000u64)).from(first_address);

    // Send the transaction and wait for it to be mined (with at least 1 confirmation)
    let receipt = provider
        .send_transaction(tx, None) // Send the transaction
        .await? // Wait for the transaction to be processed
        .log_msg("Pending transfer") // Log a message for the pending transaction
        .confirmations(1) // Wait for 1 confirmation
        .await? // Await for confirmation
        .context("Missing receipt")?; // Ensure the receipt is not missing

    // Print the block number in which the transaction was mined
    println!(
        "TX mined in block {}",
        receipt.block_number.context("cannot get block number")? // Handle potential error if block number is unavailable
    );

    // Query and print the balance of the random address after the transaction
    println!(
        "Balance of {} after TX: {}",
        other_address_hex,
        provider.get_balance(other_address, None).await? // Fetch and display updated balance
    );

    Ok(()) // Return Ok if everything succeeds
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::signers::LocalWallet;

    #[tokio::test]
    async fn test_wallet_generation() -> Result<()> {
        let mnemonic = "gas monster ski craft below illegal discover limit dog bundle bus artefact";

        // Create and launch a Ganache instance (local Ethereum test blockchain) with the mnemonic
        let ganache = Ganache::new().mnemonic(mnemonic).spawn();

        // Create a local wallet from the first generated key from Ganache
        let wallet: LocalWallet = ganache.keys()[0].clone().into();

        let address = wallet.address();
        let address_hex = address.encode_hex::<String>();
        let prefix = "0x".to_string();
        let address_hex = prefix + &address_hex;
        println!(
            "wallet first address: {}",
            address.encode_hex::<String>() // Encode the address to a hexadecimal string
        );

        // Assertions to check if the address is valid
        assert!(
            address.is_zero() == false,
            "Wallet address should not be zero"
        );
        assert_eq!(
            address_hex.to_string().len(),
            42,
            "Wallet address should be 42 characters long"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_get_balance() {
        let mnemonic = "gas monster ski craft below illegal discover limit dog bundle bus artefact";
        let ganache = Ganache::new().mnemonic(mnemonic).spawn();
        let provider = Provider::try_from(ganache.endpoint()).unwrap();

        let wallet: LocalWallet = ganache.keys()[0].clone().into();
        let address = wallet.address();

        // Check initial balance
        let balance = provider.get_balance(address, None).await.unwrap();
        assert!(
            balance > U256::zero(),
            "Initial balance should be greater than zero"
        );
    }

    #[tokio::test]
    async fn test_send_transaction() -> Result<()> {
        // Change return type to Result
        let mnemonic = "gas monster ski craft below illegal discover limit dog bundle bus artefact";
        let ganache = Ganache::new().mnemonic(mnemonic).spawn();
        let provider = Provider::try_from(ganache.endpoint()).unwrap();

        let wallet: LocalWallet = ganache.keys()[0].clone().into();
        let first_address = wallet.address();
        let other_address = "0xB794F5eA0ba39494cE839613fffBA74279579268"
            .parse::<Address>()
            .unwrap();

        let initial_balance = provider.get_balance(other_address, None).await.unwrap();

        // Send transaction
        let tx = TransactionRequest::pay(other_address, U256::from(1000u64)).from(first_address);
        let pending_tx = provider.send_transaction(tx, None).await.unwrap();

        // Await the transaction receipt
        let receipt = pending_tx.await?.context("Missing receipt")?; // This will now compile correctly

        // Check that the transaction was mined successfully
        assert!(
            receipt.block_number.is_some(),
            "Transaction should have been mined"
        );

        // Check that the balance of the recipient has increased
        let new_balance = provider.get_balance(other_address, None).await.unwrap();
        assert!(
            new_balance > initial_balance,
            "Recipient's balance should have increased"
        );

        Ok(()) // Return Ok to indicate success
    }

    #[tokio::test]
    async fn test_get_balance_nonexistent_address() {
        let mnemonic = "gas monster ski craft below illegal discover limit dog bundle bus artefact";
        let ganache = Ganache::new().mnemonic(mnemonic).spawn();
        let provider = Provider::try_from(ganache.endpoint()).unwrap();

        let non_existent_address = "0x0000000000000000000000000000000000000000"
            .parse::<Address>()
            .unwrap();

        // Expect an error or a balance of zero
        let balance = provider
            .get_balance(non_existent_address, None)
            .await
            .unwrap();
        assert_eq!(
            balance,
            U256::zero(),
            "Balance for non-existent address should be zero"
        );
    }
}
