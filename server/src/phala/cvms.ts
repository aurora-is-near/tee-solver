import { execSync } from 'child_process';
import { writeFileSync } from 'fs';
import { join } from 'path';
import { getConfig } from '../config';
import { getApiKey, saveApiKey } from './credentials';
import { logger } from '../utils/logger';

export class PhalaCloudService {
  async setupPhalaAuth(): Promise<void> {
    const config = await getConfig();
    
    const localApiKey = await getApiKey();
    if (!localApiKey) {
        if (config.phala.apiKey) {
            await saveApiKey(config.phala.apiKey);
        } else {
            throw new Error('PHALA_API_KEY is not set');
        }
    }

    logger.info('Phala auth configured successfully');
  }

  async createCVM(poolId: number, tokenIds: string[]): Promise<void> {
    const config = await getConfig();

    // const composePath = join(process.cwd(), `docker-compose.yaml`);
    const envPath = join(process.cwd(), `.env.phala`);
    const envContent = `
NEAR_NETWORK_ID=${config.near.networkId}
INTENTS_CONTRACT=${config.near.contract.intents}
SOLVER_REGISTRY_CONTRACT=${config.near.contract.solverRegistry}
SOLVER_POOL_ID=${poolId}
AMM_TOKEN1_ID=${tokenIds[0]}
AMM_TOKEN2_ID=${tokenIds[1]}
    `;
    writeFileSync(envPath, envContent);

    // const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const output = execSync(`npx phala cvms create -n solver-pool-${poolId} -c docker-compose.yaml -e .env.phala`, { encoding: 'utf-8' });

    logger.info(output);

    // const match = output.match(/CVM host URL: (https:\/\/[^\s]+)/);
    // if (!match) {
    //   throw new Error('Failed to extract CVM host URL from output');
    // }

    // return match[1];
  }

  async getCVMAddress(cvmHostUrl: string): Promise<string> {
    try {
      const response = await fetch(`${cvmHostUrl}/address`);
      const data = await response.json() as { address: string };
      return data.address;
    } catch (error) {
      console.error('Failed to get CVM address:', error);
      throw error;
    }
  }
}
