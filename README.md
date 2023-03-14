# butter-core-near

The project includes 2 types of contracts, which are:
1. **core factory contract**: factory contract to create butter core contract
2. **core contract**: butter core contract which will do swap in ref finance dex

## Pre-requisites

**1. rust**

Follow [these instructions](https://doc.rust-lang.org/book/ch01-01-installation.html) for setting up Rust.
Then, add the **wasm32-unknown-unknown** toolchain which enables compiling Rust to Web Assembly (wasm), the low-level language used by the NEAR platform.

```shell
# Get Rust in linux and MacOS
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env

# Add the wasm toolchain
rustup target add wasm32-unknown-unknown
```

**2. near-cli**

The NEAR Command Line Interface (CLI) is a tool that enables to interact with the NEAR network directly from the shell.
Follow [here](https://docs.near.org/tools/near-cli) for installing near-cli.
Then, select the network and login with your master account.

```shell
# Install near-cli in linux and McsOS
npm install -g near-cli

# The default network for near-cli is testnet, change the network by setting NEAR_ENV
# export NEAR_ENV=mainnet

# login with your master account
near login
```

## Build the contracts

Run below script to build:

```shell
./scripts/build.sh
```

2 wasm files will be generated in directory ./script/res, which are: 
1. **butter_core_factory.wasm**: factory contract to deploy and initialize the butter core contract.
2. **butter_core.wasm**: butter core contract which will do swap in ref finance dex


## Deploy the contracts
**1. Configure below parameters in ./scripts/config.sh**

```shell
MASTER_ACCOUNT="map002.testnet" # make sure the account is already created on NEAR blockchain
CONTROLLER=mos.mfac.$MASTER_ACCOUNT    # the MOS account which will call core contract
REF_EXCHANGE=ref-finance-101.testnet   # the ref finance dex on which the core contract will do swap, v2.ref-finance.near for mainnet
WRAPPED_TOKEN=wrap.testnet             # wrap.near for mainnet
OWNER=multisig.mfac.$MASTER_ACCOUNT    # the multisig contract account
FACTORY_NAME=corefac # the name of core factory contract to be created, the account ID will be $MFACTORY_NAME.$MASTER_ACCOUNT
```

**2. Deploy the factory contract:**
```shell
    ./scripts/deploy.sh deploy_factory
```

**3. Deploy and initialize butter core contract, you may need more than one butter core contracts:**
```shell
    ./scripts/deploy.sh deploy_core  core0
    ./scripts/deploy.sh deploy_core  core1
    ./scripts/deploy.sh deploy_core  core2
```
