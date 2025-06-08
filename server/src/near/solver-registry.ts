import { getConfig } from "../config";
import { PoolInfo, WorkerInfo } from "../types/solver-registry";
import { range } from "../utils/array";
import { viewFunction } from "./utils";

export class SolverRegistry {
  private solverRegistryContract: string | null = null;

  async init() {
    if (this.solverRegistryContract) {
      return;
    }
    const config = await getConfig();
    this.solverRegistryContract = config.near.contract.solverRegistry;
  }

  public async getPoolLen(): Promise<number> {
    await this.init();
    return await viewFunction({
      contractId: this.solverRegistryContract!,
      methodName: 'get_pool_len',
      args: {},
    });
  }

  public async getPool(poolId: number): Promise<PoolInfo> {
    await this.init();
    return await viewFunction({
      contractId: this.solverRegistryContract!,
      methodName: 'get_pool',
      args: { pool_id: poolId },
    });
  }

  public async getWorkerLen(): Promise<number> {
    await this.init();
    return await viewFunction({
      contractId: this.solverRegistryContract!,
      methodName: 'get_worker_len',
      args: {},
    });
  }

  public async getWorkers(): Promise<WorkerInfo[]> {
    await this.init();

    const workerLen = await this.getWorkerLen();
    const limit = 100;
    const workers = [];
    for (let i = 0; i < workerLen; i += limit) {
      const _workers = await viewFunction({
        contractId: this.solverRegistryContract!,
        methodName: 'get_workers',
        args: {
          offset: i,
          limit,
        },
      });
      workers.push(..._workers);
    }
    return workers;
  }

  public async getPoolsWithoutWorkers(): Promise<number[]> {
    await this.init();

    const poolLen = await this.getPoolLen();

    const workers = await this.getWorkers();
    const poolIds = new Set(workers.map((worker) => worker.pool_id));

    return range(poolLen).filter((poolId) => !poolIds.has(poolId));
  }
}
