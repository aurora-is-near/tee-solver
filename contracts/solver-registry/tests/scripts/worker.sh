export NEAR_ENV=testnet
export SOLVER_REGISTRY_CONTRACT=solver-registry-dev.testnet
export SOLVER_GOV_ACCOUNT=solver-gov.testnet
export WORKER_CODEHASH=b08c929607a344db24191b59c388abdca2bc9721d8eab7fb740b19de03a7d690

# approve worker codehash
near call $SOLVER_REGISTRY_CONTRACT approve_codehash '{"codehash":"'$WORKER_CODEHASH'"}' --accountId $SOLVER_GOV_ACCOUNT
