# Agave SWQOS Metrics

A command-line utility for analyzing Solana network Quality of Service (QoS) parameters for a given validator node,
focusing on QUIC stream and receive window metrics.

## Description

This tool helps validators understand their network QoS parameters based on their stake weight in the network. It
calculates:

- Maximum allowed concurrent uni-streams ([agave-implementation](https://github.com/anza-xyz/agave/blob/8854e65d81b8d74c6e55e292b93e3a66e72a4c81/streamer/src/nonblocking/quic.rs#L465))
- Receive window size ([agave-implementation](https://github.com/anza-xyz/agave/blob/8854e65d81b8d74c6e55e292b93e3a66e72a4c81/streamer/src/nonblocking/quic.rs#L663))
- Maximum transaction capacity

## Usage Example

```bash
cargo r -- --rpc-url https://api.mainnet-beta.solana.com --validator-key YOUR_VALIDATOR_PUBKEY

