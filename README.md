# CosmWasm Crypto Upwork

Smart contract for managing profiles and funds in a decentralized crypto upwork

## Set up

- Install Rust
- Install cargo-wasm
- Install Node and npm

## Build project

```sh
    cargo wasm
```

## Test smart contract

- Update .env file with your freelancer and client wallet mnemonics 

```sh
    npm i @cosmjs/cosmwasm-stargate @cosmjs/proto-signing @cosmjs/stargate dotenv
```

```sh
    node scripts/test.js
```

## Deploy smart contract on Neutron

- Update .env file with your freelancer and client wallet mnemonics 

```sh
    npm i @cosmjs/cosmwasm-stargate @cosmjs/proto-signing @cosmjs/stargate dotenv
```

```sh
    node scripts/deploy.js
```
