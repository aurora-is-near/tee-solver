import { SolverRegistry } from "../near/solver-registry";
import { PhalaCloudService } from "../phala/cvms";
import { logger } from "../utils/logger";

export async function deploySolvers() {
  const solverRegistry = new SolverRegistry();
  const poolsWithoutWorkers = await solverRegistry.getPoolsWithoutWorkers();
  const phala = new PhalaCloudService();

  logger.info(`Pools without workers: [${poolsWithoutWorkers.join(', ')}]`);

  for (const poolId of poolsWithoutWorkers) {
    const pool = await solverRegistry.getPool(poolId);
    logger.info(`Deploying solver for pool ${poolId}`, pool, pool);
    await phala.createCVM(poolId, pool.token_ids);
  }

  setTimeout(async () => {
    await deploySolvers();
  }, 60 * 1000);
}
