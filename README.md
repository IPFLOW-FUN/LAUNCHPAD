# LAUNCHPAD

**Audited Smart Contracts for Token Launchpad**

This repository contains a suite of audited smart contracts for building a decentralized token launchpad on the Solana blockchain.

## Features

-   **Secure by Design:** All contracts have undergone a security audit to ensure robustness and safety of funds.
-   **Customizable Sale Rounds:** Configure different phases for your token sale, such as seed, private, and public rounds.
-   **Vesting Schedules:** Built-in support for token vesting to manage token distribution over time.
-   **Whitelist & KYC:** Easily integrate with identity verification services to manage participant access.

## Getting Started

### Prerequisites

-   [Rust](https://www.rust-lang.org/tools/install)
-   [Solana Tool Suite](https://docs.solana.com/cli/install-solana-cli-tools)
-   [Anchor Framework](https://www.anchor-lang.com/docs/installation)

### Installation & Deployment

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/your-username/LAUNCHPAD.git
    cd LAUNCHPAD
    ```

2.  **Build the smart contracts:**
    ```bash
    anchor build
    ```

3.  **Run tests:**
    ```bash
    anchor test
    ```

4.  **Deploy to a local validator:**
    ```bash
    solana-test-validator
    anchor deploy
    ```

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.