# CCTP v2 Contract ABIs

## Status: PLACEHOLDER - Needs Update

These ABI files are currently copies of the v1 contracts and serve as placeholders to enable initial v2 implementation work. They need to be replaced with the actual v2 contract ABIs.

## v2 Contract Differences

Based on Circle's documentation, v2 contracts add these methods:

### TokenMessengerV2
- `depositForBurn()` - Enhanced with finality threshold and fee parameters
- `depositForBurnWithCaller()` - Specifies authorized destination caller
- `getMinFeeAmount()` - Returns minimum fee for fast transfer (on chains with fee switch)

### MessageTransmitterV2
- `sendMessage()` - Generic message sending with finality control
- `receiveMessage()` - Enhanced to handle different finality levels
- Methods to query finality thresholds and message states

## TODO

1. Fetch actual v2 ABIs from Circle's deployed contracts:
   - Ethereum: [Address TBD]
   - Arbitrum: [Address TBD]
   - Base: [Address TBD]
   - Linea: [Address TBD]
   - Sonic: [Address TBD]
   - Sei: [Address TBD]
   - BNB Chain: [Address TBD]

2. Update ABI files with v2-specific methods and events

3. Regenerate Rust bindings with actual v2 interfaces

## Sources

- Circle CCTP GitHub: https://github.com/circlefin/evm-cctp-contracts
- Circle Docs: https://developers.circle.com/cctp/technical-guide
- Block explorers (once addresses are known)
