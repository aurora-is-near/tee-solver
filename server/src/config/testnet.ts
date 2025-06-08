import { optionalEnv, requiredEnv } from "./env";
import { Config } from '../types/config';

const config: Config = {
  near: {
    networkId: 'testnet',
    rpcUrl: optionalEnv('NEAR_RPC_URL') || 'https://neart.lava.build',
    contract: {
      intents: 'mock-intents.testnet',
      solverRegistry: 'solver-registry-dev.testnet',
    },
    account: {
      operator: optionalEnv('OPERATOR_ACCOUNT_ID') || 'solver-master.testnet',
    },
  },
  phala: {
    apiKey: optionalEnv('PHALA_API_KEY') || '',
  },
  worker: {
    minimumBalance: 0.05,
  }
};

export default config;
