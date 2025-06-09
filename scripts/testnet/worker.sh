export NEAR_ENV=testnet
export SOLVER_REGISTRY_CONTRACT=solver-registry-dev.testnet
export SOLVER_GOV_ACCOUNT=solver-gov.testnet
export WORKER_CODEHASH=a19e0247bff656a3857b12f90ccf4d053e713089608ebfcb8c8951ea52c6392d

# approve worker codehash
near call $SOLVER_REGISTRY_CONTRACT approve_codehash '{"codehash":"'$WORKER_CODEHASH'"}' --accountId $SOLVER_GOV_ACCOUNT
