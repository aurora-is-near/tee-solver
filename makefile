RUSTFLAGS = "-C link-arg=-s"

all: lint solver-registry intents-vault

lint:
	@cargo fmt --all
	@cargo clippy --fix --allow-dirty --allow-staged

solver-registry:
	$(call compile-release,solver-registry)
	@mkdir -p contracts/solver-registry/res
	@cp target/near/solver_registry/solver_registry.wasm ./contracts/solver-registry/res/solver_registry.wasm

intents-vault:
	$(call compile-release,intents-vault)
	@mkdir -p contracts/intents-vault/res
	@cp target/near/intents_vault/intents_vault.wasm ./contracts/intents-vault/res/intents_vault.wasm

mock-intents:
	$(call compile-release,mock-intents)
	@mkdir -p contracts/mock-intents/res
	@cp target/near/mock_intents/mock_intents.wasm ./contracts/mock-intents/res/mock_intents.wasm

test: solver-registry intents-vault mock-intents
	@cargo test -- --nocapture

define compile-release
	@rustup target add wasm32-unknown-unknown
	@cd contracts/$(1) && cargo near build non-reproducible-wasm $(if $(2),--features $(2))
endef
