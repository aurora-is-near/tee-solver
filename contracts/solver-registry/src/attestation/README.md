## TEE Attestation

The current TEE attestation module reuses the implementation from [NEAR MPC](https://github.com/near/mpc) TEE attestation:
- Referenced source code on 2025-09-03: https://github.com/near/mpc/tree/4241e195016c6667ec4d796399759bea1060ef1a/crates/attestation/src

We have made slight changes to make it compatible with TEE Solver's configuration.

1. Skipped verification: LINE 310~326 in `attestation.rs`
    1. Skip local key provider since KMS will be enabled
    2. Skip MPC hash since we don't emit the docker image hash event in worker

2. Modified App Compose Validation: LINE 147~158 in `attestation.rs`
    1. Requires KMS enabled (with dstack v0.5.x)
    2. Local key provider can be disabled
    3. Gateway can be enabled
    4. Allow a few environment variables for solver
    5. Instance ID is available
    6. `secure_time` is true by default in dstack. It's OK as long as `secure_time` is not set to false.
    7. Pre launch script can be set

3. Updated TCB Info Template in `assets/tcb_info.json` with the one from TEE Solver's CVM
    1. The `mrtd`, `rtmr0`, `rtmr1` and `rtmr2` fields of workers' CVMs must be the same as the TCB info template file
