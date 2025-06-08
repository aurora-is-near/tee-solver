export interface WorkerInfo {
  account_id: string;
  pool_id: number;
  checksum: string;
  codehash: string;
}

export interface PoolInfo {
  /// List of tokens in the pool.
  token_ids: string[],
  /// How much NEAR this contract has.
  amounts: string[],
  /// Fee charged for swap in basis points
  fee: number,
  /// Total number of shares.
  shares_total_supply: string,
}
