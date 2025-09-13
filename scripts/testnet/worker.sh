export NEAR_ENV=testnet
export SOLVER_REGISTRY_CONTRACT=solver-registry-stg.testnet
export SOLVER_GOV_ACCOUNT=solver-gov.testnet
export WORKER_COMPOSE_HASH=1248c4379f6d9825b6d3ccbb1ecb001af16761086285be9edc1f40291041e196

# approve worker compose hash
near call $SOLVER_REGISTRY_CONTRACT approve_compose_hash '{"compose_hash":"'$WORKER_COMPOSE_HASH'"}' --accountId $SOLVER_GOV_ACCOUNT
