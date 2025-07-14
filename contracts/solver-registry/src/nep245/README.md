# NEP-245 Multi-Token Standard Implementation

This directory contains the implementation of the NEP-245 multi-token standard for the TEE Solver Registry's liquidity pool shares.

## Overview

The NEP-245 implementation allows liquidity pool shares to be represented as transferable multi-tokens, enabling:

- **Transferable Liquidity**: Users can transfer their liquidity pool shares to other accounts
- **Standard Compliance**: Full compliance with the NEP-245 multi-token standard
- **Backward Compatibility**: Existing pool functionality remains unchanged
- **Event Tracking**: Comprehensive event emission for all operations
- **Efficient Storage**: Uses existing pool.shares structure without additional storage

## Architecture

### Token Structure

Each liquidity pool has a corresponding share token with ID format: `{pool_id}`

- **Token ID**: `0`, `1`, `2`, etc. (pool ID as string)
- **Total Supply**: Uses existing `pool.shares_total_supply`
- **Balances**: Uses existing `pool.shares` LookupMap

### Storage Structure

The implementation leverages the existing pool structure:

```rust
pub struct Pool {
    // ... existing fields ...
    pub shares: LookupMap<AccountId, Balance>, // account_id -> shares
    pub shares_total_supply: Balance,
    // ... other fields ...
}
```

**No additional storage fields are required** - the NEP-245 interface is implemented as a wrapper around the existing pool.shares structure.

## Core Features

### 1. Multi-Token Core (NEP-245)

- `mt_transfer`: Transfer single token
- `mt_batch_transfer`: Transfer multiple tokens
- `mt_transfer_call`: Transfer with callback
- `mt_batch_transfer_call`: Batch transfer with callback
- `mt_token`: Get token information
- `mt_balance_of`: Get balance for specific token
- `mt_batch_balance_of`: Get balances for multiple tokens
- `mt_supply`: Get total supply for token
- `mt_batch_supply`: Get total supplies for multiple tokens

### 2. Multi-Token Enumeration

- `mt_tokens`: List all tokens with pagination
- `mt_tokens_for_owner`: List tokens owned by specific account

### 3. Integration with Liquidity Pools

#### Adding Liquidity
1. User deposits tokens into the pool
2. Pool calculates shares to mint
3. Shares are added to `pool.shares[account_id]`
4. Event `PoolShareMinted` is emitted

#### Removing Liquidity
1. User removes liquidity from pool
2. Shares are deducted from `pool.shares[account_id]`
3. Tokens are returned to user
4. Event `PoolShareBurned` is emitted

#### Transferring Shares
1. User transfers NEP-245 share tokens to another account
2. Balances are updated in `pool.shares`
3. Event `PoolShareTransferred` is emitted

## Events

### NEP-245 Standard Events
- `MtMint`: When tokens are minted
- `MtBurn`: When tokens are burned
- `MtTransfer`: When tokens are transferred

### Custom Pool Share Events
- `PoolShareMinted`: When pool shares are minted
- `PoolShareBurned`: When pool shares are burned
- `PoolShareTransferred`: When pool shares are transferred

## View Methods

### Standard NEP-245 Views
- All standard NEP-245 view methods are implemented

### Custom Views
- `get_account_pool_shares`: Get all pool shares for an account
- `get_pool_share_total_supply`: Get total supply for specific pool
- `get_all_pool_share_tokens`: Get all pool share tokens and supplies

## Usage Examples

### Transfer Pool Shares
```bash
near call <contract_id> mt_transfer '{
  "receiver_id": "alice.near",
  "token_id": "0",
  "amount": "1000000000000000000000000",
  "approval": null,
  "memo": "Transfering liquidity shares"
}' --accountId bob.near
```

### Check Balance
```bash
near view <contract_id> mt_balance_of '{
  "account_id": "alice.near",
  "token_id": "0"
}'
```

### List All Tokens
```bash
near view <contract_id> mt_tokens '{
  "from_index": "0",
  "limit": 10
}'
```

## Benefits

1. **Liquidity**: Users can now transfer their liquidity positions
2. **Composability**: Pool shares can be used in DeFi protocols
3. **Standardization**: Follows established NEAR token standards
4. **Transparency**: All operations are tracked via events
5. **Flexibility**: Supports both single and batch operations
6. **Efficiency**: No additional storage overhead

## Implementation Details

### Token ID Parsing
- Token IDs are pool IDs as strings: `0`, `1`, `2`, etc.
- Pool ID is extracted by parsing the token ID as `u32`
- Direct mapping to existing pool structure

### Balance Management
- `mt_balance_of` directly queries `pool.shares[account_id]`
- `mt_transfer` updates `pool.shares` for both sender and receiver
- No separate balance tracking needed

### Supply Tracking
- `mt_supply` returns `pool.shares_total_supply`
- Supply is managed by existing pool operations

### Internal Methods
The implementation uses internal helper methods in `core.rs`:
- `internal_mt_balance_of`: Get balance for a specific token
- `internal_mt_mint`: Mint tokens to an account
- `internal_mt_burn`: Burn tokens from an account
- `internal_mt_transfer`: Transfer tokens between accounts
- `internal_mt_batch_transfer`: Batch transfer multiple tokens
- `internal_mt_batch_transfer_call`: Batch transfer with callback
- `get_pool_share_token_id`: Convert pool ID to token ID
- `get_pool_id_from_token_id`: Parse token ID to get pool ID

## Security Considerations

- All transfers require proper authorization
- Balance checks prevent overflows
- Events provide audit trail
- Backward compatibility maintained
- Uses existing, tested pool logic

## Future Enhancements

- Support for approvals and delegated transfers
- Integration with DEX aggregators
- Cross-chain bridge support
- Advanced reward distribution mechanisms 