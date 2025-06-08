import { optionalEnv, requiredEnv } from "./env";
import { Config } from '../types/config';

const config: Config = {
  near: {
    networkId: 'mainnet',
    rpcUrl: optionalEnv('NEAR_RPC_URL') || 'https://near.lava.build',
    contract: {
      intents: 'intents.near',
      solverRegistry: 'solver-registry.near',
    },
    account: {
      operator: requiredEnv('OPERATOR_ACCOUNT_ID'),
    },
  },
  phala: {
    apiKey: optionalEnv('PHALA_API_KEY') || 'phala-api-key.txt',
  },
  worker: {
    minimumBalance: 0.05,
  }
};

export default config;
