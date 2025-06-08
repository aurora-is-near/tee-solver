export NEAR_ENV=testnet
export SOLVER_REGISTRY_CONTRACT=solver-registry-dev.testnet
export SOLVER_GOV_ACCOUNT=solver-gov.testnet
export WORKER_CODEHASH=

# approve worker codehash
near call $SOLVER_REGISTRY_CONTRACT approve_codehash '{"codehash":"'$WORKER_CODEHASH'"}' --accountId $SOLVER_GOV_ACCOUNT
