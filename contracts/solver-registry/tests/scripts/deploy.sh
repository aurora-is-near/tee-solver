export NEAR_ENV=testnet
export INTENTS_CONTRACT=mock-intents.testnet
export SOLVER_REGISTRY_CONTRACT=solver-registry-dev.testnet
export SOLVER_GOV_ACCOUNT=solver-gov.testnet

# deploy mock intents contract
near deploy $INTENTS_CONTRACT ../../../../target/near/mock_intents/mock_intents.wasm --initFunction new --initArgs '{}'

# deploy solver registry contract
near deploy $SOLVER_REGISTRY_CONTRACT ../../../../target/near/solver_registry/solver_registry.wasm --initFunction new --initArgs '{"owner_id":"'$SOLVER_GOV_ACCOUNT'","intents_contract_id":"'$INTENTS_CONTRACT'"}'
near deploy $SOLVER_REGISTRY_CONTRACT ../../../../target/near/solver_registry/solver_registry.wasm
