import { SolverRegistry } from "../near/solver-registry";
import { logger } from "../utils/logger";

export async function fundSolvers() {
  logger.info('Funding solvers');

  setTimeout(async () => {
    await fundSolvers();
  }, 60 * 1000);
}
