import * as dotenv from "dotenv";
import { deploySolvers } from "./tasks/deploy-solvers";
import { setupAuth } from "./tasks/setup-auth";

dotenv.config();

async function main() {
  // configure auths (e.g. Phala Cloud)
  await setupAuth();

  // Find out the pools without solvers inside CVMs, create solvers for the pools.
  // After a new solver CVM instance is created, fund its address with some NEAR to make sure it can register the worker.
  deploySolvers();
}

main().catch(console.error);
