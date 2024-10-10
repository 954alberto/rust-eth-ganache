
# Ethereum Local Blockchain Interaction with Ganache

This Rust project demonstrates how to set up a local Ethereum blockchain using [Ganache](https://www.trufflesuite.com/ganache) and interact with it by querying wallet balances and sending transactions using the [ethers-rs](https://github.com/gakonst/ethers-rs) library.

## Features

- Launches a local Ethereum blockchain using Ganache.
- Queries the balance of a wallet generated from a mnemonic.
- Sends a transaction from one address to another.
- Retrieves and displays updated balance information after the transaction.

## Prerequisites

Before running the project, ensure you have the following installed:

- **Rust** (and `cargo`) - [Installation guide](https://www.rust-lang.org/tools/install)
- **Ganache** - A local blockchain for Ethereum development. It is included as part of the `ethers-rs` utilities.

## Dependencies

This project uses several crates from the Rust ecosystem, listed in `Cargo.toml`:

- `ethers` – For Ethereum blockchain interaction.
- `eyre` – For error handling and context management.
- `tokio` – For asynchronous execution.
- `hex` – For hex encoding of addresses.

## Setup Instructions

1. **Clone the Repository**:

   ```bash
   git clone <repository-url>
   cd <repository-directory>
   ```

2. **Install Dependencies**:

   Run the following command to install all the dependencies specified in `Cargo.toml`:

   ```bash
   cargo build
   ```

3. **Run the Program**:

   You can execute the program using `cargo run`. This will spin up a local Ethereum blockchain, query balances, send a transaction, and display results.

   ```bash
   cargo run
   ```

## Code Overview

The main code logic is located in `main.rs` and does the following:

1. **Ganache Setup**:
    - A local blockchain is launched using a predefined mnemonic.
    - An Ethereum wallet is generated from the mnemonic, and its first address is queried.

2. **Balance Query**:
    - The program queries the balance of the generated wallet and a random Ethereum address.

3. **Transaction**:
    - It sends a small transaction (1000 Wei) from the wallet's first address to the random address.

4. **Receipt & Balance Update**:
    - After the transaction is mined, the program retrieves the receipt and checks the updated balance of the random address.

## Example Output

```bash
HTTP Endpoint: http://127.0.0.1:8545
wallet first address: 0x627306090abaB3A6e1400e9345bC60c78a8BEf57
wallet first address balance: 100000000000000000000
Balance for address 0xaf206dCE72A0ef76643dfeDa34DB764E2126E646: 0
Pending transfer
TX mined in block 3
Balance of 0xaf206dCE72A0ef76643dfeDa34DB764E2126E646 after TX: 1000
```

## Customization

- **Changing Mnemonic**:
   If you want to use a different mnemonic, update the `mnemonic` variable in the `main.rs` file with your own mnemonic phrase.
   
- **Adjusting Transaction Value**:
   The value of the transaction can be adjusted in this line:

   ```rust
   let tx = TransactionRequest::pay(other_address, U256::from(1000u64)).from(first_address);
   ```

   Change `1000u64` to your desired transaction amount in Wei.

## License

This project is licensed under the MIT License.
