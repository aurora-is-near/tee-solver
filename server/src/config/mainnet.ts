import { optionalEnv, requiredEnv } from "./env";
import { Config } from './types';

const config: Config = {
  near: {
    networkId: 'mainnet',
    rpcUrl: optionalEnv('NEAR_RPC_URL') || 'https://near.lava.build',
    contract: {
      intents: 'intents.near',
      solverRegistry: 'solver-registry.near',
    },
    account: {
      operatorAddress: requiredEnv('OPERATOR_ACCOUNT_ID'),
      operatorPrivateKey: requiredEnv('OPERATOR_PRIVATE_KEY') as `ed25519:${string}`,
    },
  },
  phala: {
    apiKey: optionalEnv('PHALA_API_KEY') || 'phala-api-key.txt',
  },
  worker: {
    minimumBalance: 0.1, // NEAR
  }
};

export default config;
