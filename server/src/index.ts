import { deploySolvers } from "./tasks/deploy-solvers";
import { fundSolvers } from "./tasks/fund-solvers";
import { setupAuth } from "./tasks/setup-auth";

async function main() {
  // 0. Check environment, ensure Phala Cloud CLI is installed and auth is configured
  // Use environment variables to configure Phala auth. Create API_KEY_FILE if needed.
  await setupAuth();

  // 1. Query the get_workers() and get_pool_len() functions in solver-registry-dev.testnet contract in NEAR Testnet
  // If there's no existing workers for any given pool, create a worker on Phala Cloud using the Docker image at https://hub.docker.com/repository/docker/robortyan/intents-tee-amm-solver
  // A docker-compose.yaml file needs to be created, which will be used by the Phala Cloud CLI.
  // The details about how to use Phala Cloud CLI can be found at: https://github.com/Phala-Network/phala-cloud-cli/blob/main/README.md

  deploySolvers();

  // 2. After a new CVM is created via Phala Cloud CLI, the CVM needs to be managed with the server.
  // Monitor the status of the CVM's implicit account using https://cvm-host-url/address to fetch the address of the account.
  // Fund the address with 0.05 NEAR to make sure it can register the worker on solver-registry-dev.testnet.

  fundSolvers();
}

main().catch(console.error);
