export NEAR_ENV=testnet
export SOLVER_REGISTRY_CONTRACT=solver-registry-dev.testnet
export SOLVER_GOV_ACCOUNT=solver-gov.testnet
export WORKER_CODEHASH=73056c3446210e747b2ed3d5dfad5f842b31e4a9155c0feaeeb45a3f515816fc

# approve worker codehash
near call $SOLVER_REGISTRY_CONTRACT approve_codehash '{"codehash":"'$WORKER_CODEHASH'"}' --accountId $SOLVER_GOV_ACCOUNT
