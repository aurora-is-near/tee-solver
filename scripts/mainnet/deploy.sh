export NEAR_ENV=mainnet
export INTENTS_CONTRACT=intents.near
export SOLVER_REGISTRY_CONTRACT=solver-registry-dev.near
export SOLVER_GOV_ACCOUNT=solver-gov.near

# deploy solver registry contract
near deploy $SOLVER_REGISTRY_CONTRACT ../../contracts/solver-registry/res/solver_registry.wasm --initFunction new --initArgs '{"owner_id":"'$SOLVER_GOV_ACCOUNT'","intents_contract_id":"'$INTENTS_CONTRACT'"}'
