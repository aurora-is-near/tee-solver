export NEAR_ENV=mainnet
export SOLVER_REGISTRY_CONTRACT=solver-registry-stg.near
export SOLVER_GOV_ACCOUNT=solver-gov.near
export WORKER_COMPOSE_HASH=abf86c9f6c42e4b63766c62063cb308a7b403336bb92716c7567f4415f59f968

# approve worker compose hash
near call $SOLVER_REGISTRY_CONTRACT approve_compose_hash '{"compose_hash":"'$WORKER_COMPOSE_HASH'"}' --accountId $SOLVER_GOV_ACCOUNT
