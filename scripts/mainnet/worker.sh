export NEAR_ENV=mainnet
export SOLVER_REGISTRY_CONTRACT=solver-registry-dev.near
export SOLVER_GOV_ACCOUNT=solver-gov.near
export WORKER_CODEHASH=91d6e0b2173f9a512bd92ca7ee7260f1c2bc2f5b88c7c92cc7cbcb7263e8e68e

# approve worker codehash
near call $SOLVER_REGISTRY_CONTRACT approve_codehash '{"codehash":"'$WORKER_CODEHASH'"}' --accountId $SOLVER_GOV_ACCOUNT
