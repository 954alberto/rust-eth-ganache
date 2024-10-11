use ethers::{
    contract::{Contract, ContractFactory}, // Import for interacting with and deploying Ethereum smart contracts
    middleware::SignerMiddleware,          // Middleware to sign transactions using a wallet
    prelude::{LocalWallet, Middleware, Provider, Signer, U256}, // Types for wallet, Ethereum provider, and other utilities
    types::BlockNumber, // Used for referencing Ethereum block numbers
    utils::Ganache,     // Utility to spin up a local Ethereum testnet (Ganache)
};

use ethers_solc::{Artifact, Project, ProjectPathsConfig}; // Import for Solidity project and artifact management
use ethers_solc::{ConfigurableArtifacts, ProjectCompileOutput}; // Solidity compilation outputs and configuration
use eyre::{eyre, ContextCompat, Ok, Result}; // For error handling and contextual errors
use hex::ToHex; // Utility to convert addresses and other data to hexadecimal
use std::{
    path::{Path, PathBuf}, // Used for file system path management
    time::Duration,        // Duration utility used to set intervals
};

// Type alias for a contract deployed using a wallet and signing middleware
pub type SignerDeployedContract<T> = Contract<SignerMiddleware<Provider<T>, LocalWallet>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Define a mnemonic (12-word seed) to generate private keys for the wallet
    let mnemonic = "brisk usual burst upper buddy female library dial rifle mercy globe nurse";

    // Launch a local Ganache Ethereum testnet instance using the mnemonic
    let ganache = Ganache::new().mnemonic(mnemonic).spawn();
    println!("HTTP Endpoint: {}", ganache.endpoint()); // Print the Ganache instance's HTTP endpoint

    // Generate a local wallet using the first private key derived from the mnemonic
    let wallet: LocalWallet = ganache.keys()[0].clone().into();
    let first_address = wallet.address(); // Get the wallet's address (derived from the private key)
    println!(
        "wallet first address: {}",
        first_address.encode_hex::<String>() // Convert the address to hexadecimal and print it
    );

    // Create a provider to interact with the Ethereum network (Ganache in this case)
    let provider = Provider::try_from(ganache.endpoint())?.interval(Duration::from_millis(10)); // Set polling interval
    let chain_id = provider.get_chainid().await?; // Get the chain ID for the Ethereum network
    println!("Ganache started with chain id {}", chain_id); // Print the chain ID

    // Define the folder containing Solidity contract files
    let contracts_folder = "examples/";

    // Compile the Solidity contracts located in the folder
    let project = compile(contracts_folder).await?;

    // Print the details of the compiled project, including ABI and functions
    print_project(project.clone()).await?;

    // Get the wallet's balance from the Ganache provider
    let balance = provider.get_balance(wallet.address(), None).await?;
    println!(
        "Wallet first address {} balance: {}",
        wallet.address().encode_hex::<String>(), // Encode the address to hexadecimal for printing
        balance
    );

    // Contract deployment begins here
    let contract_name = "BUSDImplementation"; // The name of the contract to deploy
    let extension = ".sol"; // File extension for Solidity contracts
    let binding = contracts_folder.to_string() + contract_name + extension; // Construct the full contract path
    let contract_path = binding.as_str(); // Convert to string reference
    let contract_relative_path = Path::new(contract_path); // Create a path object for the contract

    // Convert the relative contract path to an absolute path on the filesystem
    let contract_absolute_path = std::fs::canonicalize(contract_relative_path)?;

    // Convert the absolute path to a string for further use (required for the `find` function)
    let contract_absolute_str = contract_absolute_path
        .to_str()
        .context("Failed to convert path to string")?; // Error handling for failed conversion

    // Locate the compiled contract using the project object
    let contract = project
        .find(contract_absolute_str, contract_name) // Find the contract by its name and path
        .context("Contract not found")? // Handle the case where the contract is not found
        .clone(); // Clone the contract (ownership handling)

    // Extract ABI (Application Binary Interface) and bytecode from the compiled contract
    let (abi, bytecode, _) = contract.into_parts();
    let abi = abi.context("Missing abi from contract")?; // Ensure that ABI is available
    let bytecode = bytecode.context("Missing bytecode from contract")?; // Ensure that bytecode is available

    // Rebuild the wallet with the correct chain ID (required to sign transactions on the correct chain)
    let wallet = wallet.with_chain_id(chain_id.as_u64());
    // Create a client to interact with the blockchain (includes the signing wallet)
    let client = SignerMiddleware::new(provider.clone(), wallet).into();

    // Create a factory for deploying the contract using the ABI and bytecode
    let factory = ContractFactory::new(abi.clone(), bytecode, client);

    // Initialize the deployment process (passing constructor arguments if any, here it is empty `()`)
    let mut deployer = factory.deploy(())?;

    // Get the latest block information to determine gas pricing
    let block = provider
        .clone()
        .get_block(BlockNumber::Latest)
        .await?
        .context("Failed to get block");

    // Get the base fee for the next block and set the gas price for the transaction
    let gas_price = block?
        .next_block_base_fee()
        .context("Failed to get the base fee for the next block")?;
    deployer.tx.set_gas_price::<U256>(gas_price); // Set gas price for the transaction

    // Send the transaction to deploy the contract and await its completion
    let contract = deployer.clone().legacy().send().await?;
    println!(
        "BUSDImpl contract address {}",
        contract.address().encode_hex::<String>() // Print the deployed contract's address
    );

    Ok(()) // Indicate that the process completed successfully
}

// Function to compile a Solidity project from the given root folder path
pub async fn compile(root: &str) -> Result<ProjectCompileOutput<ConfigurableArtifacts>> {
    let root = PathBuf::from(root); // Convert the root folder path to a PathBuf object
    if !root.exists() {
        return Err(eyre!("Project root {root:?} does not exist!")); // Error handling for non-existent project root
    }

    // Define the paths to be used for the Solidity project
    let paths = ProjectPathsConfig::builder()
        .root(&root)
        .sources(&root)
        .build()?; // Build the project path configuration

    // Build the project object, enabling auto-detection of the Solidity compiler
    let project = Project::builder()
        .paths(paths)
        .set_auto_detect(true) // Automatically detect Solidity compiler
        .no_artifacts() // Avoid writing artifacts to disk
        .build()?;

    // Compile the Solidity project
    let output = project.compile()?;

    // Check if there were any compiler errors
    if output.has_compiler_errors() {
        Err(eyre!(
            "Compiling solidity project failed: {:?}",
            output.output().errors // Print compilation errors
        ))
    } else {
        Ok(output.clone()) // Return the compiled output if successful
    }
}

// Function to print the details of the compiled contracts, including ABI and functions
pub async fn print_project(project: ProjectCompileOutput<ConfigurableArtifacts>) -> Result<()> {
    let artifacts = project.into_artifacts(); // Extract the compiled artifacts (contracts)
    for (id, artifact) in artifacts {
        let name = id.name; // Get the contract's name
        let abi = artifact.abi.context("No ABI found for artifact {name}")?; // Get the ABI and ensure it exists

        println!("{}", "=".repeat(80)); // Print a separator
        println!("CONTRACT: {:?}", name); // Print the contract name

        let contract = &abi.abi;
        let functions = contract.functions(); // Get the list of functions from the contract
        let functions = functions.cloned(); // Clone the function list for iteration
        let constructor = contract.constructor(); // Get the constructor if available

        // If the contract has a constructor, print its arguments
        if let Some(constructor) = constructor {
            let args = &constructor.inputs;
            println!("CONSTRUCTOR args: {:?}", args); // Print the constructor arguments
        }

        // Print each function's name and parameters
        for func in functions {
            let name = &func.name; // Get the function name
            let params = &func.inputs; // Get the function parameters
            println!("FUNCTION {name} {params:?}"); // Print function details
        }
    }
    Ok(())
}
