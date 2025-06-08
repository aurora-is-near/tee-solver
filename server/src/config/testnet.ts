import { optionalEnv, requiredEnv } from "./env";
import { Config } from './types';

const config: Config = {
  near: {
    networkId: 'testnet',
    rpcUrl: optionalEnv('NEAR_RPC_URL') || 'https://neart.lava.build',
    contract: {
      intents: 'mock-intents.testnet',
      solverRegistry: 'solver-registry-dev.testnet',
    },
    account: {
      operatorAddress: optionalEnv('OPERATOR_ACCOUNT_ID') || 'solver-master.testnet',
      operatorPrivateKey: requiredEnv('OPERATOR_PRIVATE_KEY') as `ed25519:${string}`,
    },
  },
  phala: {
    apiKey: optionalEnv('PHALA_API_KEY') || '',
  },
  worker: {
    minimumBalance: 0.1, // NEAR
  }
};

export default config;
