export NEAR_ENV=testnet
export INTENTS_CONTRACT=mock-intents.testnet
export SOLVER_REGISTRY_CONTRACT=solver-registry-dev.testnet
export SOLVER_GOV_ACCOUNT=solver-gov.testnet
export SOLVER_TESTER_ACCOUNT=solver-alpha.testnet
export WNEAR_TOKEN=wrap.testnet
export USDC_TOKEN=usdc.fakes.testnet
export POOL_ID=0
export POOL_CONTRACT='pool-'$POOL_ID'.'$SOLVER_REGISTRY_CONTRACT

# create liquidity pool
near call $SOLVER_REGISTRY_CONTRACT create_liquidity_pool '{"token_ids":["'$WNEAR_TOKEN'","'$USDC_TOKEN'"],"fee":100}' --accountId $SOLVER_TESTER_ACCOUNT --deposit 1.5 --gas 300000000000000

# deposit NEAR for wNEAR
near call $WNEAR_TOKEN near_deposit '{}' --accountId $SOLVER_TESTER_ACCOUNT --deposit 2
near view $WNEAR_TOKEN ft_balance_of '{"account_id":"'$SOLVER_TESTER_ACCOUNT'"}'

# mint fake USDC
near call $USDC_TOKEN storage_deposit '{"account_id":"'$SOLVER_TESTER_ACCOUNT'","registration_only":true}' --accountId $SOLVER_TESTER_ACCOUNT --deposit 0.00125
near call $USDC_TOKEN mint '{"account_id":"'$SOLVER_TESTER_ACCOUNT'","amount":"100000000"}' --accountId $SOLVER_TESTER_ACCOUNT
near view $USDC_TOKEN ft_balance_of '{"account_id":"'$SOLVER_TESTER_ACCOUNT'"}'

# register accounts
near call $WNEAR_TOKEN storage_deposit '{"account_id":"'$SOLVER_REGISTRY_CONTRACT'","registration_only":true}' --accountId $SOLVER_TESTER_ACCOUNT --deposit 0.00125
near call $USDC_TOKEN storage_deposit '{"account_id":"'$SOLVER_REGISTRY_CONTRACT'","registration_only":true}' --accountId $SOLVER_TESTER_ACCOUNT --deposit 0.00125
near call $WNEAR_TOKEN storage_deposit '{"account_id":"'$INTENTS_CONTRACT'","registration_only":true}' --accountId $SOLVER_TESTER_ACCOUNT --deposit 0.00125
near call $USDC_TOKEN storage_deposit '{"account_id":"'$INTENTS_CONTRACT'","registration_only":true}' --accountId $SOLVER_TESTER_ACCOUNT --deposit 0.00125

# deposit into pool
near call $WNEAR_TOKEN ft_transfer_call '{"receiver_id":"'$SOLVER_REGISTRY_CONTRACT'","amount":"100000000000000000000000","msg":"{\"DepositIntoPool\": {\"pool_id\": '$POOL_ID'}}"}' --accountId $SOLVER_TESTER_ACCOUNT --gas 300000000000000 --depositYocto 1
near call $USDC_TOKEN ft_transfer_call '{"receiver_id":"'$SOLVER_REGISTRY_CONTRACT'","amount":"5000000","msg":"{\"DepositIntoPool\": {\"pool_id\": '$POOL_ID'}}"}' --accountId $SOLVER_TESTER_ACCOUNT --gas 300000000000000 --depositYocto 1

# check balance (USDC)
near view $USDC_TOKEN ft_balance_of '{"account_id":"'$INTENTS_CONTRACT'"}'
near view $USDC_TOKEN ft_balance_of '{"account_id":"'$SOLVER_REGISTRY_CONTRACT'"}'
near view $USDC_TOKEN ft_balance_of '{"account_id":"'$SOLVER_TESTER_ACCOUNT'"}'

# check balance (wNAER)
near view $WNEAR_TOKEN ft_balance_of '{"account_id":"'$INTENTS_CONTRACT'"}'
near view $WNEAR_TOKEN ft_balance_of '{"account_id":"'$SOLVER_REGISTRY_CONTRACT'"}'
near view $WNEAR_TOKEN ft_balance_of '{"account_id":"'$SOLVER_TESTER_ACCOUNT'"}'
