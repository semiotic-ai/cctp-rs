# CCTP v2 Contract ABIs

These are the actual v2 contract ABIs fetched from Circle's deployed contracts.

## Contract Addresses (Unified)

v2 uses the same contract addresses across all supported chains within each environment:

**Mainnet:**

- TokenMessengerV2: `0x28b5a0e9C621a5BadaA536219b3a228C8168cf5d`
- MessageTransmitterV2: `0x81D40F21F12A8F0E3252Bccb954D722d4c464B64`

**Testnet:**

- TokenMessengerV2: `0x8FE6B999Dc680CcFDD5Bf7EB0974218be2542DAA`
- MessageTransmitterV2: `0xE737e5cEBEEBa77EFE34D4aa090756590b1CE275`

## Key v2 Methods

### TokenMessengerV2

| Method | Description |
|--------|-------------|
| `depositForBurn` | Burn USDC with finality threshold and max fee |
| `depositForBurnWithHook` | Burn with programmable hook data for destination actions |
| `handleReceiveFinalizedMessage` | Handle standard (threshold 2000) transfers |
| `handleReceiveUnfinalizedMessage` | Handle fast (threshold 1000) transfers |

### MessageTransmitterV2

| Method | Description |
|--------|-------------|
| `sendMessage` | Generic cross-chain messaging with finality control |
| `receiveMessage` | Receive and verify attested messages |
| `usedNonces` | Check if a message has been processed (replay protection) |

## Differences from v1

- **Finality thresholds**: 1000 (fast/confirmed) vs 2000 (standard/finalized)
- **Fee support**: `maxFee` parameter for fast transfer fees
- **Hooks**: `hookData` parameter for programmable destination actions
- **Removed**: `replaceDepositForBurn` and `replaceMessage` methods

## Sources

- Circle CCTP GitHub: <https://github.com/circlefin/evm-cctp-contracts>
- Circle Docs: <https://developers.circle.com/stablecoins/cctp-getting-started>
