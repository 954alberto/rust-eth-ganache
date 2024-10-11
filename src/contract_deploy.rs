use ethers::{
    contract::{Contract, ContractFactory},
    middleware::SignerMiddleware,
    prelude::{LocalWallet, Middleware, Provider, Signer, U256},
    types::BlockNumber,
    utils::Ganache,
};

use ethers_solc::{Artifact, Project, ProjectPathsConfig};
use ethers_solc::{ConfigurableArtifacts, ProjectCompileOutput};
use eyre::{eyre, ContextCompat, Ok, Result};
use hex::ToHex;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

pub type SignerDeployedContract<T> = Contract<SignerMiddleware<Provider<T>, LocalWallet>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Define a mnemonic for a wallet (used to derive private keys)
    let mnemonic = "brisk usual burst upper buddy female library dial rifle mercy globe nurse";

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
    let chain_id = provider.get_chainid().await?;
    println!("Ganache started with chain id {}", chain_id);
    let contracts_folder = "examples/";
    let project = compile(contracts_folder).await?;

    print_project(project.clone()).await?;

    let balance = provider.get_balance(wallet.address(), None).await?;
    println!(
        "Wallet first address {} balance: {}",
        wallet.address().encode_hex::<String>(),
        balance
    );

    // Deploying contradct
    let contract_name = "BUSDImplementation";
    let extension = ".sol";
    let binding = contracts_folder.to_string() + contract_name + extension;
    let contract_path = binding.as_str();
    let contract_relative_path = Path::new(contract_path);

    // Convert the relative path to an absolute path
    let contract_absolute_path = std::fs::canonicalize(contract_relative_path)?;

    // Convert the PathBuf to a string if needed for the `find` function
    let contract_absolute_str = contract_absolute_path
        .to_str()
        .context("Failed to convert path to string")?;

    //let contract = project.find("/home/papi/Projects/rust-eth-ganache/examples/BUSDImplementation.sol", "BUSDImplementation").unwrap();

    let contract = project
        .find(contract_absolute_str, contract_name)
        .context("Contract not found")?
        .clone();
    //println!("contract: {contract:?}");

    //get abi and bytecode, which are only available in a compiled contract
    let (abi, bytecode, _) = contract.into_parts();
    let abi = abi.context("Missing abi from contract")?;
    let bytecode = bytecode.context("Missing bytecode from contract")?;
    //create signer client
    let wallet = wallet.with_chain_id(chain_id.as_u64());
    let client = SignerMiddleware::new(provider.clone(), wallet).into();
    //deploy contract
    let factory = ContractFactory::new(abi.clone(), bytecode, client);

    let mut deployer = factory.deploy(())?;
    let block = provider
        .clone()
        .get_block(BlockNumber::Latest)
        .await?
        .context("Failed to get block");

    let gas_price = block?
        .next_block_base_fee()
        .context("Failed to get the base fee for the next block")?;
    deployer.tx.set_gas_price::<U256>(gas_price);

    let contract = deployer.clone().legacy().send().await?;
    println!(
        "BUSDImpl contract address {}",
        contract.address().encode_hex::<String>()
    );

    Ok(()) // Return Ok if everything succeeds
}

pub async fn compile(root: &str) -> Result<ProjectCompileOutput<ConfigurableArtifacts>> {
    let root = PathBuf::from(root);
    if !root.exists() {
        return Err(eyre!("Project root {root:?} does not exist!"));
    }

    let paths = ProjectPathsConfig::builder()
        .root(&root)
        .sources(&root)
        .build()?;

    let project = Project::builder()
        .paths(paths)
        .set_auto_detect(true)
        .no_artifacts()
        .build()?;

    let output = project.compile()?;

    if output.has_compiler_errors() {
        Err(eyre!(
            "Compiling solidity project failed: {:?}",
            output.output().errors
        ))
    } else {
        Ok(output.clone())
    }
}

pub async fn print_project(project: ProjectCompileOutput<ConfigurableArtifacts>) -> Result<()> {
    let artifacts = project.into_artifacts();
    for (id, artifact) in artifacts {
        let name = id.name;
        let abi = artifact.abi.context("No ABI found for artifact {name}")?;

        println!("{}", "=".repeat(80));
        println!("CONTRACT: {:?}", name);

        let contract = &abi.abi;
        let functions = contract.functions();
        let functions = functions.cloned();
        let constructor = contract.constructor();

        if let Some(constructor) = constructor {
            let args = &constructor.inputs;
            println!("CONSTRUCTOR args: {:?}", args);
        }
        for func in functions {
            let name = &func.name;
            let params = &func.inputs;
            println!("FUNCTION {name} {params:?}");
        }
    }
    Ok(())
}
