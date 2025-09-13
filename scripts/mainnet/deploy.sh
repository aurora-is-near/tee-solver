export NEAR_ENV=mainnet
export INTENTS_CONTRACT=intents.near
export SOLVER_REGISTRY_CONTRACT=solver-registry-stg.near
export SOLVER_GOV_ACCOUNT=solver-gov.near
export WORKER_PING_TIMEOUT_MS=600000

# deploy solver registry contract
near deploy $SOLVER_REGISTRY_CONTRACT ../../contracts/solver-registry/res/solver_registry.wasm --initFunction new --initArgs '{"owner_id":"'$SOLVER_GOV_ACCOUNT'","intents_contract_id":"'$INTENTS_CONTRACT'","worker_ping_timeout_ms":'$WORKER_PING_TIMEOUT_MS'}'
