export NEAR_ENV=testnet
export INTENTS_CONTRACT=mock-intents-2.testnet
export SOLVER_REGISTRY_CONTRACT=solver-registry-stg.testnet
export SOLVER_GOV_ACCOUNT=solver-gov.testnet
export WORKER_PING_TIMEOUT_MS=600000

# create account
# near account create-account sponsor-by-faucet-service solver-beta.testnet autogenerate-new-keypair save-to-legacy-keychain network-config testnet create

# deploy mock intents contract
near deploy $INTENTS_CONTRACT ../../contracts/mock-intents/res/mock_intents.wasm --initFunction new --initArgs '{}'

# deploy solver registry contract
near deploy $SOLVER_REGISTRY_CONTRACT ../../contracts/solver-registry/res/solver_registry.wasm --initFunction new --initArgs '{"owner_id":"'$SOLVER_GOV_ACCOUNT'","intents_contract_id":"'$INTENTS_CONTRACT'","worker_ping_timeout_ms":'$WORKER_PING_TIMEOUT_MS'}'

