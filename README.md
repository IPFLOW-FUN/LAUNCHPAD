# Launchpad Protocol

[![License](https://img.shields.io/badge/license-Apache--2.0-green)](LICENSE)
[![Contributing](https://img.shields.io/badge/contributing-welcome-brightgreen.svg)](CONTRIBUTING.md)

The Launchpad Protocol is a suite of smart contracts on the Solana blockchain for creating and launching new tokens via a bonding curve mechanism. It provides a complete lifecycle for token launches, from initial sale to liquidity pool creation on Raydium.

## Features

- **Token Creation**: Instantly create a new SPL token with metadata.
- **Bonding Curve**: Automated price discovery and token distribution during the initial offering.
- **Whitelist Support**: Securely manage pre-sale access using a Merkle tree for whitelisted addresses.
- **Automated Liquidity Migration**: Seamlessly transfers collected SOL and remaining tokens to a Raydium CP (Constant Product) swap pool.
- **Post-Migration Claims**: Allows initial buyers to claim their tokens after the liquidity pool is established.
- **Configurable Parameters**: Flexible control for project owners to set fees, token reserves, and sale timelines.
- **On-Chain Events**: Emits detailed events for every critical action, such as trades, migration, and claims.

## Program

This repository contains the primary `launchpad` program.

| Program | Description |
| --- | --- |
| `launchpad` | The core protocol for managing token launches on a bonding curve. |

## Core Instructions

The protocol exposes several key instructions to manage the token launch lifecycle:

- `initialize`: Initializes the global state for the protocol.
- `create_token`: Creates a new token and its associated bonding curve with specified parameters.
- `buy`: Allows users to purchase tokens with SOL from the bonding curve during the sale period.
- `sell`: Allows users to sell their purchased tokens back to the curve before it completes.
- `withdraw`: Executed after the sale ends to distribute creator tokens and platform fees.
- `migrate_liquidity`: Migrates the assets from the bonding curve to a Raydium CP swap pool.
- `claim`: Allows users to claim their purchased tokens after liquidity has been migrated.

## Audits

The smart contracts have been audited to ensure security and reliability.

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request.

## License

This project is licensed under the Apache-2.0. See the [LICENSE](LICENSE) file for details.