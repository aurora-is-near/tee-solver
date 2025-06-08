export type Config = {
  near: {
    networkId: 'mainnet' | 'testnet';
    rpcUrl: string;
    contract: {
      intents: string;
      solverRegistry: string;
    };
    account: {
      operator: string;
    };
  };
  phala: {
    apiKey: string;
  };
  worker: {
    minimumBalance: number;
  };
};
