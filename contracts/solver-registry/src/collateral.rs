use crate::*;
use serde_json::Value;
use sha2::{Digest as _, Sha256, Sha384};

pub fn get_collateral(raw_quote_collateral: String) -> QuoteCollateralV3 {
    let quote_collateral: serde_json::Value =
        serde_json::from_str(&raw_quote_collateral).expect("TCB Info should be valid JSON");

    let tcb_info_issuer_chain = quote_collateral["tcb_info_issuer_chain"]
        .as_str()
        .unwrap()
        .to_owned();
    let tcb_info = quote_collateral["tcb_info"].as_str().unwrap().to_owned();
    let tcb_info_signature =
        hex::decode(quote_collateral["tcb_info_signature"].as_str().unwrap()).unwrap();
    let qe_identity_issuer_chain = quote_collateral["qe_identity_issuer_chain"]
        .as_str()
        .unwrap()
        .to_owned();
    let qe_identity = quote_collateral["qe_identity"].as_str().unwrap().to_owned();
    let qe_identity_signature =
        hex::decode(quote_collateral["qe_identity_signature"].as_str().unwrap()).unwrap();
    let pck_crl_issuer_chain = quote_collateral["pck_crl_issuer_chain"]
        .as_str()
        .unwrap()
        .to_owned();
    let root_ca_crl = hex::decode(quote_collateral["root_ca_crl"].as_str().unwrap()).unwrap();
    let pck_crl = hex::decode(quote_collateral["pck_crl"].as_str().unwrap()).unwrap();

    QuoteCollateralV3 {
        tcb_info_issuer_chain,
        tcb_info,
        tcb_info_signature,
        qe_identity_issuer_chain,
        qe_identity,
        qe_identity_signature,
        pck_crl_issuer_chain,
        root_ca_crl,
        pck_crl,
    }
}

pub fn verify_codehash(raw_tcb_info: String, rtmr3: String) -> String {
    let tcb_info: Value =
        serde_json::from_str(&raw_tcb_info).expect("TCB Info should be valid JSON");
    let event_log = tcb_info["event_log"].as_array().unwrap();
    // get compose hash from events
    let expected_compose_hash = event_log
        .iter()
        .find(|e| e["event"].as_str().unwrap() == "compose-hash")
        .unwrap()["digest"]
        .as_str()
        .unwrap();

    // replay the rtmr3 and compose hash
    let replayed_rtmr3 = replay_rtmr(event_log.to_owned(), 3);
    let app_compose = tcb_info["app_compose"].as_str().unwrap();
    let replayed_compose_hash: String = replay_app_compose(app_compose);

    // compose hash match expected
    require!(
        replayed_compose_hash == expected_compose_hash,
        "Invalid compose hash"
    );
    // event with compose hash matches report rtmr3
    require!(replayed_rtmr3 == rtmr3, "Invalid rtmr3");

    let (_, right) = app_compose.split_once("\\n    image:").unwrap();
    let (left, _) = right.split_once("\\n").unwrap();
    let (_, codehash) = left.split_once("@sha256:").unwrap();

    codehash.to_owned()
}

// helpers

fn replay_rtmr(event_log: Vec<Value>, imr: u8) -> String {
    let mut digest = [0u8; 48];

    // filter by imr
    let filtered_events = event_log
        .iter()
        .filter(|e| e["imr"].as_u64().unwrap() as u8 == imr);

    // hash all digests together
    for event in filtered_events {
        let mut hasher = Sha384::new();
        hasher.update(digest);
        hasher.update(
            decode(event["digest"].as_str().unwrap())
                .unwrap()
                .as_slice(),
        );
        digest = hasher.finalize().into();
    }

    // return hex encoded digest (rtmr[imr])
    encode(digest)
}

fn replay_app_compose(app_compose: &str) -> String {
    // sha256 of app_compose from TcbInfo
    let mut sha256 = Sha256::new();
    sha256.update(app_compose);
    let sha256bytes: [u8; 32] = sha256.finalize().into();

    // sha384 of custom encoding: [phala_prefix]:[event_name]:[sha256_payload]
    let mut hasher = Sha384::new();
    hasher.update(vec![0x01, 0x00, 0x00, 0x08]);
    hasher.update(b":");
    hasher.update("compose-hash".as_bytes());
    hasher.update(b":");
    hasher.update(sha256bytes);
    let digest: [u8; 48] = hasher.finalize().into();

    encode(digest)
}

#[test]
fn test() {
    use dcap_qvl::verify;
    use hex::decode;
    use serde_json::json;

    let tcb_info = json!(
        {
            "rootfs_hash": "355eabbaf84843b85bdfba348baf46dc0f3c4e02326f0b23ff124e36bb053327c2f78d745391b5e9da692472be851d74",
            "mrtd": "c68518a0ebb42136c12b2275164f8c72f25fa9a34392228687ed6e9caeb9c0f1dbd895e9cf475121c029dc47e70e91fd",
            "rtmr0": "85e0855a6384fa1c8a6ab36d0dcbfaa11a5753e5a070c08218ae5fe872fcb86967fd2449c29e22e59dc9fec998cb6547",
            "rtmr1": "154e08f5c1f7b1fce4cbfe1c14f3ba67b70044ede2751487279cd1f2e4239dee99a6d45e24ebde6b6a6f5ae49878e0e6",
            "rtmr2": "9edcd363660e85b71c318324996dda756c372d9f6960edbfa863b1e684822eb48dd95e218ae2b78e51ef97f3b8f5c9dc",
            "rtmr3": "4ac3c9279570e72ed82275275b3daa645868d24251fd357e6bf6a26a59d864797f42e77bdf034b8ce80e5c5c06c81071",
            "event_log": [
                {
                "imr": 0,
                "event_type": 2147483659_u64,
                "digest": "0e35f1b315ba6c912cf791e5c79dd9d3a2b8704516aa27d4e5aa78fb09ede04aef2bbd02ac7a8734c48562b9c26ba35d",
                "event": "",
                "event_payload": "095464785461626c65000100000000000000af96bb93f2b9b84e9462e0ba745642360090800000000000"
                },
                {
                "imr": 0,
                "event_type": 2147483658_u64,
                "digest": "344bc51c980ba621aaa00da3ed7436f7d6e549197dfe699515dfa2c6583d95e6412af21c097d473155875ffd561d6790",
                "event": "",
                "event_payload": "2946762858585858585858582d585858582d585858582d585858582d58585858585858585858585829000000c0ff000000000040080000000000"
                },
                {
                "imr": 0,
                "event_type": 2147483649_u64,
                "digest": "9dc3a1f80bcec915391dcda5ffbb15e7419f77eab462bbf72b42166fb70d50325e37b36f93537a863769bcf9bedae6fb",
                "event": "",
                "event_payload": "61dfe48bca93d211aa0d00e098032b8c0a00000000000000000000000000000053006500630075007200650042006f006f007400"
                },
                {
                "imr": 0,
                "event_type": 2147483649_u64,
                "digest": "6f2e3cbc14f9def86980f5f66fd85e99d63e69a73014ed8a5633ce56eca5b64b692108c56110e22acadcef58c3250f1b",
                "event": "",
                "event_payload": "61dfe48bca93d211aa0d00e098032b8c0200000000000000000000000000000050004b00"
                },
                {
                "imr": 0,
                "event_type": 2147483649_u64,
                "digest": "d607c0efb41c0d757d69bca0615c3a9ac0b1db06c557d992e906c6b7dee40e0e031640c7bfd7bcd35844ef9edeadc6f9",
                "event": "",
                "event_payload": "61dfe48bca93d211aa0d00e098032b8c030000000000000000000000000000004b0045004b00"
                },
                {
                "imr": 0,
                "event_type": 2147483649_u64,
                "digest": "08a74f8963b337acb6c93682f934496373679dd26af1089cb4eaf0c30cf260a12e814856385ab8843e56a9acea19e127",
                "event": "",
                "event_payload": "cbb219d73a3d9645a3bcdad00e67656f0200000000000000000000000000000064006200"
                },
                {
                "imr": 0,
                "event_type": 2147483649_u64,
                "digest": "18cc6e01f0c6ea99aa23f8a280423e94ad81d96d0aeb5180504fc0f7a40cb3619dd39bd6a95ec1680a86ed6ab0f9828d",
                "event": "",
                "event_payload": "cbb219d73a3d9645a3bcdad00e67656f03000000000000000000000000000000640062007800"
                },
                {
                "imr": 0,
                "event_type": 4,
                "digest": "394341b7182cd227c5c6b07ef8000cdfd86136c4292b8e576573ad7ed9ae41019f5818b4b971c9effc60e1ad9f1289f0",
                "event": "",
                "event_payload": "00000000"
                },
                {
                "imr": 0,
                "event_type": 10,
                "digest": "68cd79315e70aecd4afe7c1b23a5ed7b3b8e51a477e1739f111b3156def86bbc56ebf239dcd4591bc7a9fff90023f481",
                "event": "",
                "event_payload": "414350492044415441"
                },
                {
                "imr": 0,
                "event_type": 10,
                "digest": "6bc203b3843388cc4918459c3f5c6d1300a796fb594781b7ecfaa3ae7456975f095bfcc1156c9f2d25e8b8bc1b520f66",
                "event": "",
                "event_payload": "414350492044415441"
                },
                {
                "imr": 0,
                "event_type": 10,
                "digest": "ec9e8622a100c399d71062a945f95d8e4cdb7294e8b1c6d17a6a8d37b5084444000a78b007ef533f290243421256d25c",
                "event": "",
                "event_payload": "414350492044415441"
                },
                {
                "imr": 1,
                "event_type": 2147483651_u64,
                "digest": "b8420535898e24a6abef877153ac1103a695ed07ab9c6c74182d865a560b3b587a34d4bfdc671a505def683cc7bc7596",
                "event": "",
                "event_payload": "1860437b0000000000f4b3000000000000000000000000002a000000000000000403140072f728144ab61e44b8c39ebdd7f893c7040412006b00650072006e0065006c0000007fff0400"
                },
                {
                "imr": 0,
                "event_type": 2147483650_u64,
                "digest": "1dd6f7b457ad880d840d41c961283bab688e94e4b59359ea45686581e90feccea3c624b1226113f824f315eb60ae0a7c",
                "event": "",
                "event_payload": "61dfe48bca93d211aa0d00e098032b8c0900000000000000020000000000000042006f006f0074004f0072006400650072000000"
                },
                {
                "imr": 0,
                "event_type": 2147483650_u64,
                "digest": "23ada07f5261f12f34a0bd8e46760962d6b4d576a416f1fea1c64bc656b1d28eacf7047ae6e967c58fd2a98bfa74c298",
                "event": "",
                "event_payload": "61dfe48bca93d211aa0d00e098032b8c08000000000000003e0000000000000042006f006f0074003000300030003000090100002c0055006900410070007000000004071400c9bdb87cebf8344faaea3ee4af6516a10406140021aa2c4614760345836e8ab6f46623317fff0400"
                },
                {
                "imr": 1,
                "event_type": 2147483655_u64,
                "digest": "77a0dab2312b4e1e57a84d865a21e5b2ee8d677a21012ada819d0a98988078d3d740f6346bfe0abaa938ca20439a8d71",
                "event": "",
                "event_payload": "43616c6c696e6720454649204170706c69636174696f6e2066726f6d20426f6f74204f7074696f6e"
                },
                {
                "imr": 1,
                "event_type": 4,
                "digest": "394341b7182cd227c5c6b07ef8000cdfd86136c4292b8e576573ad7ed9ae41019f5818b4b971c9effc60e1ad9f1289f0",
                "event": "",
                "event_payload": "00000000"
                },
                {
                "imr": 2,
                "event_type": 6,
                "digest": "a68ac6d65dd62f392826c2ae44f6846363ced3418c96574b3e168de9205c8553b8198c3b9d206bc432d70a923c25b098",
                "event": "",
                "event_payload": "ed223b8f1a0000004c4f414445445f494d4147453a3a4c6f61644f7074696f6e7300"
                },
                {
                "imr": 2,
                "event_type": 6,
                "digest": "41ee4a6d142b51085c720bad4170207359538d9785391ce10b17536153ea5ba49edaa5a8c36f9f081fdf1b7aeb0ce3f0",
                "event": "",
                "event_payload": "ec223b8f0d0000004c696e757820696e6974726400"
                },
                {
                "imr": 1,
                "event_type": 2147483655_u64,
                "digest": "214b0bef1379756011344877743fdc2a5382bac6e70362d624ccf3f654407c1b4badf7d8f9295dd3dabdef65b27677e0",
                "event": "",
                "event_payload": "4578697420426f6f7420536572766963657320496e766f636174696f6e"
                },
                {
                "imr": 1,
                "event_type": 2147483655_u64,
                "digest": "0a2e01c85deae718a530ad8c6d20a84009babe6c8989269e950d8cf440c6e997695e64d455c4174a652cd080f6230b74",
                "event": "",
                "event_payload": "4578697420426f6f742053657276696365732052657475726e656420776974682053756363657373"
                },
                {
                "imr": 3,
                "event_type": 134217729,
                "digest": "355eabbaf84843b85bdfba348baf46dc0f3c4e02326f0b23ff124e36bb053327c2f78d745391b5e9da692472be851d74",
                "event": "rootfs-hash",
                "event_payload": "8b32065c2f0e77328fafc18f784b3f0bb02239e4a0dc2e2ebc1918e6a54b9cce"
                },
                {
                "imr": 3,
                "event_type": 134217729,
                "digest": "9e32d7c700b42f81a06ff6fa619189225000bdd5bb1a7b993572f4ef693ac63495aace277189b3e2524904ae119c4f2e",
                "event": "app-id",
                "event_payload": "428b8d8e30296936572684a09c81ffb7709102e3"
                },
                {
                "imr": 3,
                "event_type": 134217729,
                "digest": "b518590d11b84e8109866756a6cd438d707366a330ebc6e83e6fe293dd14d8c562b57dde56b259871fa65ca2bb97d593",
                "event": "compose-hash",
                "event_payload": "e539ba88bb6f4270414cb5180bfda1962759e923e3a09c07b7bf51b60d1263df"
                },
                {
                "imr": 3,
                "event_type": 134217729,
                "digest": "5b6a576d1da40f04179ad469e00f90a1c0044bc9e8472d0da2776acb108dc98a73560d42cea6b8b763eb4a0e6d4d82d5",
                "event": "ca-cert-hash",
                "event_payload": "d2d9c7c29e3f18e69cba87438cef21eea084c2110858230cd39c5decc629a958"
                },
                {
                "imr": 3,
                "event_type": 134217729,
                "digest": "a4aefe9aa3924953698d210a2550538c191005c767d65a5f0ef78304d88700cf9bce558f62c979e8480cf303629deb26",
                "event": "instance-id",
                "event_payload": "5b7fd2d2df7276baaffe2dff54ee779792209195"
                }
            ],
            "app_compose": "{\n    \"allowed_envs\":[],\n    \"default_gateway_domain\":\"dstack-prod8.phala.network\",\n    \"docker_compose_file\":\"version: '3.8'\\n\\nservices:\\n  intents_tee_amm_solver:\\n    image: robortyan/intents-tee-amm-solver:latest@sha256:69f1a94f8c2725523087083139f925aae588ffaa76efd08d7ba06529451c31ed\\n    platform: linux/amd64\\n    ports:\\n      - \\\"3000:3000\\\"\\n    environment:\\n      NEAR_NETWORK_ID: ${NEAR_NETWORK_ID}\\n      NEAR_NODE_URL: ${NEAR_NODE_URL}\\n      INTENTS_CONTRACT: ${INTENTS_CONTRACT}\\n      SOLVER_REGISTRY_CONTRACT: ${SOLVER_REGISTRY_CONTRACT}\\n      SOLVER_POOL_ID: ${SOLVER_POOL_ID}\\n      AMM_TOKEN1_ID: ${AMM_TOKEN1_ID}\\n      AMM_TOKEN2_ID: ${AMM_TOKEN2_ID}\\n    restart: unless-stopped\\n    volumes:\\n      - /var/run/tappd.sock:/var/run/tappd.sock\\n\",\n    \"features\":[\n        \"kms\",\n        \"tproxy-net\"\n    ],\n    \"gateway_enabled\":true,\n    \"kms_enabled\":true,\n    \"local_key_provider_enabled\":false,\n    \"manifest_version\":2,\n    \"name\":\"intents-solver-1\",\n    \"no_instance_id\":false,\n    \"pre_launch_script\":\"\\n#!/bin/bash\\necho \\\"----------------------------------------------\\\"\\necho \\\"Running Phala Cloud Pre-Launch Script v0.0.7\\\"\\necho \\\"----------------------------------------------\\\"\\nset -e\\n\\n# Function: notify host\\n\\nnotify_host() {\\n    if command -v dstack-util >/dev/null 2>&1; then\\n        dstack-util notify-host -e \\\"$1\\\" -d \\\"$2\\\"\\n    else\\n        tdxctl notify-host -e \\\"$1\\\" -d \\\"$2\\\"\\n    fi\\n}\\n\\nnotify_host_hoot_info() {\\n    notify_host \\\"boot.progress\\\" \\\"$1\\\"\\n}\\n\\nnotify_host_hoot_error() {\\n    notify_host \\\"boot.error\\\" \\\"$1\\\"\\n}\\n\\n# Function: Perform Docker cleanup\\nperform_cleanup() {\\n    echo \\\"Pruning unused images\\\"\\n    docker image prune -af\\n    echo \\\"Pruning unused volumes\\\"\\n    docker volume prune -f\\n    notify_host_hoot_info \\\"docker cleanup completed\\\"\\n}\\n\\n# Function: Check Docker login status without exposing credentials\\ncheck_docker_login() {\\n    # Try to verify login status without exposing credentials\\n    if docker info 2>/dev/null | grep -q \\\"Username\\\"; then\\n        return 0\\n    else\\n        return 1\\n    fi\\n}\\n\\n# Main logic starts here\\necho \\\"Starting login process...\\\"\\n\\n# Check if Docker credentials exist\\nif [[ -n \\\"$DSTACK_DOCKER_USERNAME\\\" && -n \\\"$DSTACK_DOCKER_PASSWORD\\\" ]]; then\\n    echo \\\"Docker credentials found\\\"\\n    \\n    # Check if already logged in\\n    if check_docker_login; then\\n        echo \\\"Already logged in to Docker registry\\\"\\n    else\\n        echo \\\"Logging in to Docker registry...\\\"\\n        # Login without exposing password in process list\\n        if [[ -n \\\"$DSTACK_DOCKER_REGISTRY\\\" ]]; then\\n            echo \\\"$DSTACK_DOCKER_PASSWORD\\\" | docker login -u \\\"$DSTACK_DOCKER_USERNAME\\\" --password-stdin \\\"$DSTACK_DOCKER_REGISTRY\\\"\\n        else\\n            echo \\\"$DSTACK_DOCKER_PASSWORD\\\" | docker login -u \\\"$DSTACK_DOCKER_USERNAME\\\" --password-stdin\\n        fi\\n        \\n        if [ $? -eq 0 ]; then\\n            echo \\\"Docker login successful\\\"\\n        else\\n            echo \\\"Docker login failed\\\"\\n            notify_host_hoot_error \\\"docker login failed\\\"\\n            exit 1\\n        fi\\n    fi\\n# Check if AWS ECR credentials exist\\nelif [[ -n \\\"$DSTACK_AWS_ACCESS_KEY_ID\\\" && -n \\\"$DSTACK_AWS_SECRET_ACCESS_KEY\\\" && -n \\\"$DSTACK_AWS_REGION\\\" && -n \\\"$DSTACK_AWS_ECR_REGISTRY\\\" ]]; then\\n    echo \\\"AWS ECR credentials found\\\"\\n    \\n    # Check if AWS CLI is installed\\n    if ! command -v aws &> /dev/null; then\\n        notify_host_hoot_info \\\"awscli not installed, installing...\\\"\\n        echo \\\"AWS CLI not installed, installing...\\\"\\n        curl \\\"https://awscli.amazonaws.com/awscli-exe-linux-x86_64-2.24.14.zip\\\" -o \\\"awscliv2.zip\\\"\\n        echo \\\"6ff031a26df7daebbfa3ccddc9af1450 awscliv2.zip\\\" | md5sum -c\\n        if [ $? -ne 0 ]; then\\n            echo \\\"MD5 checksum failed\\\"\\n            notify_host_hoot_error \\\"awscli install failed\\\"\\n            exit 1\\n        fi\\n        unzip awscliv2.zip &> /dev/null\\n        ./aws/install\\n        \\n        # Clean up installation files\\n        rm -rf awscliv2.zip aws\\n    else\\n        echo \\\"AWS CLI is already installed: $(which aws)\\\"\\n    fi\\n\\n    # Set AWS credentials as environment variables\\n    export AWS_ACCESS_KEY_ID=\\\"$DSTACK_AWS_ACCESS_KEY_ID\\\"\\n    export AWS_SECRET_ACCESS_KEY=\\\"$DSTACK_AWS_SECRET_ACCESS_KEY\\\"\\n    export AWS_DEFAULT_REGION=\\\"$DSTACK_AWS_REGION\\\"\\n    \\n    # Set session token if provided (for temporary credentials)\\n    if [[ -n \\\"$DSTACK_AWS_SESSION_TOKEN\\\" ]]; then\\n        echo \\\"AWS session token found, using temporary credentials\\\"\\n        export AWS_SESSION_TOKEN=\\\"$DSTACK_AWS_SESSION_TOKEN\\\"\\n    fi\\n    \\n    # Test AWS credentials before attempting ECR login\\n    echo \\\"Testing AWS credentials...\\\"\\n    if ! aws sts get-caller-identity &> /dev/null; then\\n        echo \\\"AWS credentials test failed\\\"\\n        notify_host_hoot_error \\\"Invalid AWS credentials\\\"\\n        exit 1\\n    fi\\n\\n    echo \\\"Logging in to AWS ECR...\\\"\\n    aws ecr get-login-password --region $DSTACK_AWS_REGION | docker login --username AWS --password-stdin \\\"$DSTACK_AWS_ECR_REGISTRY\\\"\\n    if [ $? -eq 0 ]; then\\n        echo \\\"AWS ECR login successful\\\"\\n        notify_host_hoot_info \\\"AWS ECR login successful\\\"\\n    else\\n        echo \\\"AWS ECR login failed\\\"\\n        notify_host_hoot_error \\\"AWS ECR login failed\\\"\\n        exit 1\\n    fi\\nfi\\n\\nperform_cleanup\\n\\n#\\n# Set root password if DSTACK_ROOT_PASSWORD is set.\\n#\\nif [[ -n \\\"$DSTACK_ROOT_PASSWORD\\\" ]]; then\\n    echo \\\"root:$DSTACK_ROOT_PASSWORD\\\" | chpasswd\\n    unset $DSTACK_ROOT_PASSWORD\\n    echo \\\"Root password set\\\"\\nfi\\nif [[ -n \\\"$DSTACK_ROOT_PUBLIC_KEY\\\" ]]; then\\n    mkdir -p /root/.ssh\\n    echo \\\"$DSTACK_ROOT_PUBLIC_KEY\\\" > /root/.ssh/authorized_keys\\n    unset $DSTACK_ROOT_PUBLIC_KEY\\n    echo \\\"Root public key set\\\"\\nfi\\n\\n\\nif [[ -e /var/run/dstack.sock ]]; then\\n    export DSTACK_APP_ID=$(curl -s --unix-socket /var/run/dstack.sock http://dstack/Info | jq -j .app_id)\\nelse\\n    export DSTACK_APP_ID=$(curl -s --unix-socket /var/run/tappd.sock http://dstack/prpc/Tappd.Info | jq -j .app_id)\\nfi\\n# Check if app-compose.json has default_gateway_domain field and DSTACK_GATEWAY_DOMAIN is not set\\n# If true, set DSTACK_GATEWAY_DOMAIN from app-compose.json\\nif [[ $(jq 'has(\\\"default_gateway_domain\\\")' app-compose.json) == \\\"true\\\" && -z \\\"$DSTACK_GATEWAY_DOMAIN\\\" ]]; then\\n    export DSTACK_GATEWAY_DOMAIN=$(jq -j '.default_gateway_domain' app-compose.json)\\nfi\\nif [[ -n \\\"$DSTACK_GATEWAY_DOMAIN\\\" ]]; then\\n    export DSTACK_APP_DOMAIN=$DSTACK_APP_ID\\\".\\\"$DSTACK_GATEWAY_DOMAIN\\nfi\\n\\necho \\\"----------------------------------------------\\\"\\necho \\\"Script execution completed\\\"\\necho \\\"----------------------------------------------\\\"\\n\",\n    \"public_logs\":true,\n    \"public_sysinfo\":true,\n    \"runner\":\"docker-compose\",\n    \"tproxy_enabled\":true\n}"
            }
    );

    let event_log = tcb_info["event_log"].as_array().unwrap();

    let quote_collateral = json!({"pck_crl_issuer_chain":"-----BEGIN CERTIFICATE-----\nMIICljCCAj2gAwIBAgIVAJVvXc29G+HpQEnJ1PQzzgFXC95UMAoGCCqGSM49BAMC\nMGgxGjAYBgNVBAMMEUludGVsIFNHWCBSb290IENBMRowGAYDVQQKDBFJbnRlbCBD\nb3Jwb3JhdGlvbjEUMBIGA1UEBwwLU2FudGEgQ2xhcmExCzAJBgNVBAgMAkNBMQsw\nCQYDVQQGEwJVUzAeFw0xODA1MjExMDUwMTBaFw0zMzA1MjExMDUwMTBaMHAxIjAg\nBgNVBAMMGUludGVsIFNHWCBQQ0sgUGxhdGZvcm0gQ0ExGjAYBgNVBAoMEUludGVs\nIENvcnBvcmF0aW9uMRQwEgYDVQQHDAtTYW50YSBDbGFyYTELMAkGA1UECAwCQ0Ex\nCzAJBgNVBAYTAlVTMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAENSB/7t21lXSO\n2Cuzpxw74eJB72EyDGgW5rXCtx2tVTLq6hKk6z+UiRZCnqR7psOvgqFeSxlmTlJl\neTmi2WYz3qOBuzCBuDAfBgNVHSMEGDAWgBQiZQzWWp00ifODtJVSv1AbOScGrDBS\nBgNVHR8ESzBJMEegRaBDhkFodHRwczovL2NlcnRpZmljYXRlcy50cnVzdGVkc2Vy\ndmljZXMuaW50ZWwuY29tL0ludGVsU0dYUm9vdENBLmRlcjAdBgNVHQ4EFgQUlW9d\nzb0b4elAScnU9DPOAVcL3lQwDgYDVR0PAQH/BAQDAgEGMBIGA1UdEwEB/wQIMAYB\nAf8CAQAwCgYIKoZIzj0EAwIDRwAwRAIgXsVki0w+i6VYGW3UF/22uaXe0YJDj1Ue\nnA+TjD1ai5cCICYb1SAmD5xkfTVpvo4UoyiSYxrDWLmUR4CI9NKyfPN+\n-----END CERTIFICATE-----\n-----BEGIN CERTIFICATE-----\nMIICjzCCAjSgAwIBAgIUImUM1lqdNInzg7SVUr9QGzknBqwwCgYIKoZIzj0EAwIw\naDEaMBgGA1UEAwwRSW50ZWwgU0dYIFJvb3QgQ0ExGjAYBgNVBAoMEUludGVsIENv\ncnBvcmF0aW9uMRQwEgYDVQQHDAtTYW50YSBDbGFyYTELMAkGA1UECAwCQ0ExCzAJ\nBgNVBAYTAlVTMB4XDTE4MDUyMTEwNDUxMFoXDTQ5MTIzMTIzNTk1OVowaDEaMBgG\nA1UEAwwRSW50ZWwgU0dYIFJvb3QgQ0ExGjAYBgNVBAoMEUludGVsIENvcnBvcmF0\naW9uMRQwEgYDVQQHDAtTYW50YSBDbGFyYTELMAkGA1UECAwCQ0ExCzAJBgNVBAYT\nAlVTMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEC6nEwMDIYZOj/iPWsCzaEKi7\n1OiOSLRFhWGjbnBVJfVnkY4u3IjkDYYL0MxO4mqsyYjlBalTVYxFP2sJBK5zlKOB\nuzCBuDAfBgNVHSMEGDAWgBQiZQzWWp00ifODtJVSv1AbOScGrDBSBgNVHR8ESzBJ\nMEegRaBDhkFodHRwczovL2NlcnRpZmljYXRlcy50cnVzdGVkc2VydmljZXMuaW50\nZWwuY29tL0ludGVsU0dYUm9vdENBLmRlcjAdBgNVHQ4EFgQUImUM1lqdNInzg7SV\nUr9QGzknBqwwDgYDVR0PAQH/BAQDAgEGMBIGA1UdEwEB/wQIMAYBAf8CAQEwCgYI\nKoZIzj0EAwIDSQAwRgIhAOW/5QkR+S9CiSDcNoowLuPRLsWGf/Yi7GSX94BgwTwg\nAiEA4J0lrHoMs+Xo5o/sX6O9QWxHRAvZUGOdRQ7cvqRXaqI=\n-----END CERTIFICATE-----\n","root_ca_crl":"308201203081c8020101300a06082a8648ce3d0403023068311a301806035504030c11496e74656c2053475820526f6f74204341311a3018060355040a0c11496e74656c20436f72706f726174696f6e3114301206035504070c0b53616e746120436c617261310b300906035504080c024341310b3009060355040613025553170d3235303332303131323135375a170d3236303430333131323135375aa02f302d300a0603551d140403020101301f0603551d2304183016801422650cd65a9d3489f383b49552bf501b392706ac300a06082a8648ce3d0403020347003044022030c9fce1438da0a94e4fffdd46c9650e393be6e5a7862d4e4e73527932d04af302206539efe3f734c3d7df20d9dfc4630e1c7ff0439a0f8ece101f15b5eaff9b4f33","pck_crl":"30820a6230820a08020101300a06082a8648ce3d04030230703122302006035504030c19496e74656c205347582050434b20506c6174666f726d204341311a3018060355040a0c11496e74656c20436f72706f726174696f6e3114301206035504070c0b53616e746120436c617261310b300906035504080c024341310b3009060355040613025553170d3235303833313032353531385a170d3235303933303032353531385a30820934303302146fc34e5023e728923435d61aa4b83c618166ad35170d3235303833313032353531385a300c300a0603551d1504030a01013034021500efae6e9715fca13b87e333e8261ed6d990a926ad170d3235303833313032353531385a300c300a0603551d1504030a01013034021500fd608648629cba73078b4d492f4b3ea741ad08cd170d3235303833313032353531385a300c300a0603551d1504030a010130340215008af924184e1d5afddd73c3d63a12f5e8b5737e56170d3235303833313032353531385a300c300a0603551d1504030a01013034021500b1257978cfa9ccdd0759abf8c5ca72fae3a78a9b170d3235303833313032353531385a300c300a0603551d1504030a01013033021474fea614a972be0e2843f2059835811ed872f9b3170d3235303833313032353531385a300c300a0603551d1504030a01013034021500f9c4ef56b3ab48d577e108baedf4bf88014214b9170d3235303833313032353531385a300c300a0603551d1504030a010130330214071de0778f9e5fc4f2878f30d6b07c9a30e6b30b170d3235303833313032353531385a300c300a0603551d1504030a01013034021500cde2424f972cea94ff239937f4d80c25029dd60b170d3235303833313032353531385a300c300a0603551d1504030a0101303302146c3319e5109b64507d3cf1132ce00349ef527319170d3235303833313032353531385a300c300a0603551d1504030a01013034021500df08d756b66a7497f43b5bb58ada04d3f4f7a937170d3235303833313032353531385a300c300a0603551d1504030a01013033021428af485b6cf67e409a39d5cb5aee4598f7a8fa7b170d3235303833313032353531385a300c300a0603551d1504030a01013034021500fb8b2daec092cada8aa9bc4ff2f1c20d0346668c170d3235303833313032353531385a300c300a0603551d1504030a01013034021500cd4850ac52bdcc69a6a6f058c8bc57bbd0b5f864170d3235303833313032353531385a300c300a0603551d1504030a01013034021500994dd3666f5275fb805f95dd02bd50cb2679d8ad170d3235303833313032353531385a300c300a0603551d1504030a0101303302140702136900252274d9035eedf5457462fad0ef4c170d3235303833313032353531385a300c300a0603551d1504030a01013033021461f2bf73e39b4e04aa27d801bd73d24319b5bf80170d3235303833313032353531385a300c300a0603551d1504030a0101303302143992be851b96902eff38959e6c2eff1b0651a4b5170d3235303833313032353531385a300c300a0603551d1504030a0101303302140fda43a00b68ea79b7c2deaeac0b498bdfb2af90170d3235303833313032353531385a300c300a0603551d1504030a010130330214639f139a5040fdcff191e8a4fb1bf086ed603971170d3235303833313032353531385a300c300a0603551d1504030a01013034021500959d533f9249dc1e513544cdc830bf19b7f1f301170d3235303833313032353531385a300c300a0603551d1504030a0101303302147ae37748a9f912f4c63ba7ab07c593ce1d1d1181170d3235303833313032353531385a300c300a0603551d1504030a01013033021413884b33269938c195aa170fca75da177538df0b170d3235303833313032353531385a300c300a0603551d1504030a0101303402150085d3c9381b77a7e04d119c9e5ad6749ff3ffab87170d3235303833313032353531385a300c300a0603551d1504030a0101303402150093887ca4411e7a923bd1fed2819b2949f201b5b4170d3235303833313032353531385a300c300a0603551d1504030a0101303302142498dc6283930996fd8bf23a37acbe26a3bed457170d3235303833313032353531385a300c300a0603551d1504030a010130340215008a66f1a749488667689cc3903ac54c662b712e73170d3235303833313032353531385a300c300a0603551d1504030a01013034021500afc13610bdd36cb7985d106481a880d3a01fda07170d3235303833313032353531385a300c300a0603551d1504030a01013034021500efe04b2c33d036aac96ca673bf1e9a47b64d5cbb170d3235303833313032353531385a300c300a0603551d1504030a0101303402150083d9ac8d8bb509d1c6c809ad712e8430559ed7f3170d3235303833313032353531385a300c300a0603551d1504030a0101303302147931fd50b5071c1bbfc5b7b6ded8b45b9d8b8529170d3235303833313032353531385a300c300a0603551d1504030a0101303302141fa20e2970bde5d57f7b8ddf8339484e1f1d0823170d3235303833313032353531385a300c300a0603551d1504030a0101303302141e87b2c3b32d8d23e411cef34197b95af0c8adf5170d3235303833313032353531385a300c300a0603551d1504030a010130340215009afd2ee90a473550a167d996911437c7502d1f09170d3235303833313032353531385a300c300a0603551d1504030a0101303302144481b0f11728a13b696d3ea9c770a0b15ec58dda170d3235303833313032353531385a300c300a0603551d1504030a01013034021500a7859f57982ef0e67d37bc8ef2ef5ac835ff1aa9170d3235303833313032353531385a300c300a0603551d1504030a010130340215009d67753b81e47090aea763fbec4c4549bcdb9933170d3235303833313032353531385a300c300a0603551d1504030a01013033021434bfbb7a1d9c568147e118b614f7b76ed3ef68df170d3235303833313032353531385a300c300a0603551d1504030a0101303302142c3cc6fe9279db1516d5ce39f2a898cda5a175e1170d3235303833313032353531385a300c300a0603551d1504030a010130330214717948687509234be979e4b7dce6f31bef64b68c170d3235303833313032353531385a300c300a0603551d1504030a010130340215009d76ef2c39c136e8658b6e7396b1d7445a27631f170d3235303833313032353531385a300c300a0603551d1504030a01013034021500c3e025fca995f36f59b48467939e3e34e6361a6f170d3235303833313032353531385a300c300a0603551d1504030a010130340215008c5f6b3257da05b17429e2e61ba965d67330606a170d3235303833313032353531385a300c300a0603551d1504030a01013034021500a17c51722ec1e0c3278fe8bdf052059cbec4e648170d3235303833313032353531385a300c300a0603551d1504030a0101a02f302d300a0603551d140403020101301f0603551d23041830168014956f5dcdbd1be1e94049c9d4f433ce01570bde54300a06082a8648ce3d04030203480030450220409095b6458b77ef6d9a4cb5636f91787c8147eaf54c997c8cd7a17a400c8669022100f3ddfec12b43c5195fa4bf354e2c6fbe6d9f61e47b5cf8e4a54ac6c3c348300d","tcb_info_issuer_chain":"-----BEGIN CERTIFICATE-----\nMIICjTCCAjKgAwIBAgIUfjiC1ftVKUpASY5FhAPpFJG99FUwCgYIKoZIzj0EAwIw\naDEaMBgGA1UEAwwRSW50ZWwgU0dYIFJvb3QgQ0ExGjAYBgNVBAoMEUludGVsIENv\ncnBvcmF0aW9uMRQwEgYDVQQHDAtTYW50YSBDbGFyYTELMAkGA1UECAwCQ0ExCzAJ\nBgNVBAYTAlVTMB4XDTI1MDUwNjA5MjUwMFoXDTMyMDUwNjA5MjUwMFowbDEeMBwG\nA1UEAwwVSW50ZWwgU0dYIFRDQiBTaWduaW5nMRowGAYDVQQKDBFJbnRlbCBDb3Jw\nb3JhdGlvbjEUMBIGA1UEBwwLU2FudGEgQ2xhcmExCzAJBgNVBAgMAkNBMQswCQYD\nVQQGEwJVUzBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABENFG8xzydWRfK92bmGv\nP+mAh91PEyV7Jh6FGJd5ndE9aBH7R3E4A7ubrlh/zN3C4xvpoouGlirMba+W2lju\nypajgbUwgbIwHwYDVR0jBBgwFoAUImUM1lqdNInzg7SVUr9QGzknBqwwUgYDVR0f\nBEswSTBHoEWgQ4ZBaHR0cHM6Ly9jZXJ0aWZpY2F0ZXMudHJ1c3RlZHNlcnZpY2Vz\nLmludGVsLmNvbS9JbnRlbFNHWFJvb3RDQS5kZXIwHQYDVR0OBBYEFH44gtX7VSlK\nQEmORYQD6RSRvfRVMA4GA1UdDwEB/wQEAwIGwDAMBgNVHRMBAf8EAjAAMAoGCCqG\nSM49BAMCA0kAMEYCIQDdmmRuAo3qCO8TC1IoJMITAoOEw4dlgEBHzSz1TuMSTAIh\nAKVTqOkt59+co0O3m3hC+v5Fb00FjYWcgeu3EijOULo5\n-----END CERTIFICATE-----\n-----BEGIN CERTIFICATE-----\nMIICjzCCAjSgAwIBAgIUImUM1lqdNInzg7SVUr9QGzknBqwwCgYIKoZIzj0EAwIw\naDEaMBgGA1UEAwwRSW50ZWwgU0dYIFJvb3QgQ0ExGjAYBgNVBAoMEUludGVsIENv\ncnBvcmF0aW9uMRQwEgYDVQQHDAtTYW50YSBDbGFyYTELMAkGA1UECAwCQ0ExCzAJ\nBgNVBAYTAlVTMB4XDTE4MDUyMTEwNDUxMFoXDTQ5MTIzMTIzNTk1OVowaDEaMBgG\nA1UEAwwRSW50ZWwgU0dYIFJvb3QgQ0ExGjAYBgNVBAoMEUludGVsIENvcnBvcmF0\naW9uMRQwEgYDVQQHDAtTYW50YSBDbGFyYTELMAkGA1UECAwCQ0ExCzAJBgNVBAYT\nAlVTMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEC6nEwMDIYZOj/iPWsCzaEKi7\n1OiOSLRFhWGjbnBVJfVnkY4u3IjkDYYL0MxO4mqsyYjlBalTVYxFP2sJBK5zlKOB\nuzCBuDAfBgNVHSMEGDAWgBQiZQzWWp00ifODtJVSv1AbOScGrDBSBgNVHR8ESzBJ\nMEegRaBDhkFodHRwczovL2NlcnRpZmljYXRlcy50cnVzdGVkc2VydmljZXMuaW50\nZWwuY29tL0ludGVsU0dYUm9vdENBLmRlcjAdBgNVHQ4EFgQUImUM1lqdNInzg7SV\nUr9QGzknBqwwDgYDVR0PAQH/BAQDAgEGMBIGA1UdEwEB/wQIMAYBAf8CAQEwCgYI\nKoZIzj0EAwIDSQAwRgIhAOW/5QkR+S9CiSDcNoowLuPRLsWGf/Yi7GSX94BgwTwg\nAiEA4J0lrHoMs+Xo5o/sX6O9QWxHRAvZUGOdRQ7cvqRXaqI=\n-----END CERTIFICATE-----\n","tcb_info":"{\"id\":\"TDX\",\"version\":3,\"issueDate\":\"2025-08-31T03:29:34Z\",\"nextUpdate\":\"2025-09-30T03:29:34Z\",\"fmspc\":\"20a06f000000\",\"pceId\":\"0000\",\"tcbType\":0,\"tcbEvaluationDataNumber\":17,\"tdxModule\":{\"mrsigner\":\"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\",\"attributes\":\"0000000000000000\",\"attributesMask\":\"FFFFFFFFFFFFFFFF\"},\"tdxModuleIdentities\":[{\"id\":\"TDX_03\",\"mrsigner\":\"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\",\"attributes\":\"0000000000000000\",\"attributesMask\":\"FFFFFFFFFFFFFFFF\",\"tcbLevels\":[{\"tcb\":{\"isvsvn\":3},\"tcbDate\":\"2024-03-13T00:00:00Z\",\"tcbStatus\":\"UpToDate\"}]},{\"id\":\"TDX_01\",\"mrsigner\":\"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\",\"attributes\":\"0000000000000000\",\"attributesMask\":\"FFFFFFFFFFFFFFFF\",\"tcbLevels\":[{\"tcb\":{\"isvsvn\":4},\"tcbDate\":\"2024-03-13T00:00:00Z\",\"tcbStatus\":\"UpToDate\"},{\"tcb\":{\"isvsvn\":2},\"tcbDate\":\"2023-08-09T00:00:00Z\",\"tcbStatus\":\"OutOfDate\"}]}],\"tcbLevels\":[{\"tcb\":{\"sgxtcbcomponents\":[{\"svn\":2,\"category\":\"BIOS\",\"type\":\"Early Microcode Update\"},{\"svn\":2,\"category\":\"OS/VMM\",\"type\":\"SGX Late Microcode Update\"},{\"svn\":2,\"category\":\"OS/VMM\",\"type\":\"TXT SINIT\"},{\"svn\":2,\"category\":\"BIOS\"},{\"svn\":2,\"category\":\"BIOS\"},{\"svn\":255,\"category\":\"BIOS\"},{\"svn\":0},{\"svn\":2,\"category\":\"OS/VMM\",\"type\":\"SEAMLDR ACM\"},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0}],\"pcesvn\":13,\"tdxtcbcomponents\":[{\"svn\":5,\"category\":\"OS/VMM\",\"type\":\"TDX Module\"},{\"svn\":0,\"category\":\"OS/VMM\",\"type\":\"TDX Module\"},{\"svn\":2,\"category\":\"OS/VMM\",\"type\":\"TDX Late Microcode Update\"},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0}]},\"tcbDate\":\"2024-03-13T00:00:00Z\",\"tcbStatus\":\"UpToDate\"},{\"tcb\":{\"sgxtcbcomponents\":[{\"svn\":2,\"category\":\"BIOS\",\"type\":\"Early Microcode Update\"},{\"svn\":2,\"category\":\"OS/VMM\",\"type\":\"SGX Late Microcode Update\"},{\"svn\":2,\"category\":\"OS/VMM\",\"type\":\"TXT SINIT\"},{\"svn\":2,\"category\":\"BIOS\"},{\"svn\":2,\"category\":\"BIOS\"},{\"svn\":255,\"category\":\"BIOS\"},{\"svn\":0},{\"svn\":2,\"category\":\"OS/VMM\",\"type\":\"SEAMLDR ACM\"},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0}],\"pcesvn\":5,\"tdxtcbcomponents\":[{\"svn\":5,\"category\":\"OS/VMM\",\"type\":\"TDX Module\"},{\"svn\":0,\"category\":\"OS/VMM\",\"type\":\"TDX Module\"},{\"svn\":2,\"category\":\"OS/VMM\",\"type\":\"TDX Late Microcode Update\"},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0},{\"svn\":0}]},\"tcbDate\":\"2018-01-04T00:00:00Z\",\"tcbStatus\":\"OutOfDate\"}]}","tcb_info_signature":"5c0216a117bdf66e52a506c3b7fab0bce5d44f95aaa6d32be2e97898c87b2ad965f1584894c5587320eb4d69ad126499401d1138fff72687db4359d45cdfc2cb","qe_identity_issuer_chain":"-----BEGIN CERTIFICATE-----\nMIICjTCCAjKgAwIBAgIUfjiC1ftVKUpASY5FhAPpFJG99FUwCgYIKoZIzj0EAwIw\naDEaMBgGA1UEAwwRSW50ZWwgU0dYIFJvb3QgQ0ExGjAYBgNVBAoMEUludGVsIENv\ncnBvcmF0aW9uMRQwEgYDVQQHDAtTYW50YSBDbGFyYTELMAkGA1UECAwCQ0ExCzAJ\nBgNVBAYTAlVTMB4XDTI1MDUwNjA5MjUwMFoXDTMyMDUwNjA5MjUwMFowbDEeMBwG\nA1UEAwwVSW50ZWwgU0dYIFRDQiBTaWduaW5nMRowGAYDVQQKDBFJbnRlbCBDb3Jw\nb3JhdGlvbjEUMBIGA1UEBwwLU2FudGEgQ2xhcmExCzAJBgNVBAgMAkNBMQswCQYD\nVQQGEwJVUzBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABENFG8xzydWRfK92bmGv\nP+mAh91PEyV7Jh6FGJd5ndE9aBH7R3E4A7ubrlh/zN3C4xvpoouGlirMba+W2lju\nypajgbUwgbIwHwYDVR0jBBgwFoAUImUM1lqdNInzg7SVUr9QGzknBqwwUgYDVR0f\nBEswSTBHoEWgQ4ZBaHR0cHM6Ly9jZXJ0aWZpY2F0ZXMudHJ1c3RlZHNlcnZpY2Vz\nLmludGVsLmNvbS9JbnRlbFNHWFJvb3RDQS5kZXIwHQYDVR0OBBYEFH44gtX7VSlK\nQEmORYQD6RSRvfRVMA4GA1UdDwEB/wQEAwIGwDAMBgNVHRMBAf8EAjAAMAoGCCqG\nSM49BAMCA0kAMEYCIQDdmmRuAo3qCO8TC1IoJMITAoOEw4dlgEBHzSz1TuMSTAIh\nAKVTqOkt59+co0O3m3hC+v5Fb00FjYWcgeu3EijOULo5\n-----END CERTIFICATE-----\n-----BEGIN CERTIFICATE-----\nMIICjzCCAjSgAwIBAgIUImUM1lqdNInzg7SVUr9QGzknBqwwCgYIKoZIzj0EAwIw\naDEaMBgGA1UEAwwRSW50ZWwgU0dYIFJvb3QgQ0ExGjAYBgNVBAoMEUludGVsIENv\ncnBvcmF0aW9uMRQwEgYDVQQHDAtTYW50YSBDbGFyYTELMAkGA1UECAwCQ0ExCzAJ\nBgNVBAYTAlVTMB4XDTE4MDUyMTEwNDUxMFoXDTQ5MTIzMTIzNTk1OVowaDEaMBgG\nA1UEAwwRSW50ZWwgU0dYIFJvb3QgQ0ExGjAYBgNVBAoMEUludGVsIENvcnBvcmF0\naW9uMRQwEgYDVQQHDAtTYW50YSBDbGFyYTELMAkGA1UECAwCQ0ExCzAJBgNVBAYT\nAlVTMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEC6nEwMDIYZOj/iPWsCzaEKi7\n1OiOSLRFhWGjbnBVJfVnkY4u3IjkDYYL0MxO4mqsyYjlBalTVYxFP2sJBK5zlKOB\nuzCBuDAfBgNVHSMEGDAWgBQiZQzWWp00ifODtJVSv1AbOScGrDBSBgNVHR8ESzBJ\nMEegRaBDhkFodHRwczovL2NlcnRpZmljYXRlcy50cnVzdGVkc2VydmljZXMuaW50\nZWwuY29tL0ludGVsU0dYUm9vdENBLmRlcjAdBgNVHQ4EFgQUImUM1lqdNInzg7SV\nUr9QGzknBqwwDgYDVR0PAQH/BAQDAgEGMBIGA1UdEwEB/wQIMAYBAf8CAQEwCgYI\nKoZIzj0EAwIDSQAwRgIhAOW/5QkR+S9CiSDcNoowLuPRLsWGf/Yi7GSX94BgwTwg\nAiEA4J0lrHoMs+Xo5o/sX6O9QWxHRAvZUGOdRQ7cvqRXaqI=\n-----END CERTIFICATE-----\n","qe_identity":"{\"id\":\"TD_QE\",\"version\":2,\"issueDate\":\"2025-08-31T02:58:39Z\",\"nextUpdate\":\"2025-09-30T02:58:39Z\",\"tcbEvaluationDataNumber\":17,\"miscselect\":\"00000000\",\"miscselectMask\":\"FFFFFFFF\",\"attributes\":\"11000000000000000000000000000000\",\"attributesMask\":\"FBFFFFFFFFFFFFFF0000000000000000\",\"mrsigner\":\"DC9E2A7C6F948F17474E34A7FC43ED030F7C1563F1BABDDF6340C82E0E54A8C5\",\"isvprodid\":2,\"tcbLevels\":[{\"tcb\":{\"isvsvn\":4},\"tcbDate\":\"2024-03-13T00:00:00Z\",\"tcbStatus\":\"UpToDate\"}]}","qe_identity_signature":"6751c1e14e8e4ef2054c384ea005371735093b9b367bce9d0806c9bbdb4e36ce82704da7426c74df816882b85c8f79e3da3831dcd0713ed980e88913cffba3c2"});
    let raw_quote_collateral = quote_collateral.to_string();
    let collateral = get_collateral(raw_quote_collateral);
    let quote_hex = "040002008100000000000000939a7233f79c4ca9940a0db3957f06075c72f05a3e32d1a9750ebd4216f3e09900000000060104000000000000000000000000005b38e33a6487958b72c3c12a938eaa5e3fd4510c51aeeab58c7d5ecee41d7c436489d6c8e4f92f160b7cad34207b00c1000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000702000000000000c68518a0ebb42136c12b2275164f8c72f25fa9a34392228687ed6e9caeb9c0f1dbd895e9cf475121c029dc47e70e91fd00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000085e0855a6384fa1c8a6ab36d0dcbfaa11a5753e5a070c08218ae5fe872fcb86967fd2449c29e22e59dc9fec998cb6547154e08f5c1f7b1fce4cbfe1c14f3ba67b70044ede2751487279cd1f2e4239dee99a6d45e24ebde6b6a6f5ae49878e0e69edcd363660e85b71c318324996dda756c372d9f6960edbfa863b1e684822eb48dd95e218ae2b78e51ef97f3b8f5c9dc4ac3c9279570e72ed82275275b3daa645868d24251fd357e6bf6a26a59d864797f42e77bdf034b8ce80e5c5c06c8107100000000000000000000000000656432353531393a676433633969477a5079366672326331487a6b4a506752457336644d43485a4261334d5a64575068726253d010000017f166e5f593421b3cc7aedaf429bfa4d9c2076fc5ccde188138654595376b85bc04f38fa207d9988c936f49892e9aaff5c308d2ea195117357a5a7b2121ace333c46122c0d32ccddde7443ab502537476fdc23ad990cae1e6de3ce12923ec762e5e5a632a4fe31d7503dc496812a4522dc748cb4734fe08dd2f941350de02de06004a1000000404090905ff00020000000000000000000000000000000000000000000000000000000000000000000000000000000015000000000000000700000000000000e5a3a7b5d830c2953b98534c6c59a3a34fdc34e933f7f5898f0a85cf08846bca0000000000000000000000000000000000000000000000000000000000000000dc9e2a7c6f948f17474e34a7fc43ed030f7c1563f1babddf6340c82e0e54a8c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000317240e92452a8b5f82d31740f6b9ba1ab6fcb76b75bed44107445ccb93bba350000000000000000000000000000000000000000000000000000000000000000b718a478a668a182c6fd1da473501b7cfd6d75185d3384ef22b75bbe69af9586c469ba66fc9c1da3360040b6ce4394569923119e33c5be147fcbb022e2076c7f2000000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f0500620e00002d2d2d2d2d424547494e2043455254494649434154452d2d2d2d2d0a4d494945387a4343424a69674177494241674956414d494a396b79727a596a6a74346c515941576a41363436427258684d416f4743437147534d343942414d430a4d484178496a416742674e5642414d4d47556c756447567349464e4857434251513073675547786864475a76636d306751304578476a415942674e5642416f4d0a45556c756447567349454e76636e4276636d4630615739754d5251774567594456515148444174545957353059534244624746795954454c4d416b47413155450a4341774351304578437a414a42674e5642415954416c56544d423458445449314d4459784d7a45314e444d7a4d316f5844544d794d4459784d7a45314e444d7a0a4d316f77634445694d434147413155454177775a535735305a5777675530645949464244537942445a584a3061575a70593246305a5445614d426747413155450a43677752535735305a577767513239796347397959585270623234784644415342674e564241634d43314e68626e526849454e7359584a684d517377435159440a5651514944414a445154454c4d416b474131554542684d4356564d775754415442676371686b6a4f5051494242676771686b6a4f50514d4242774e43414154620a664e33314c643255696b65667539487a3458613356454b437243514844436e38715550573442464d434a613471716b534775484b42676d5a67506c735a6766510a72535644584a58572b4465533958597539366e486f3449444454434341776b77487759445652306a42426777466f41556c5739647a62306234656c4153636e550a3944504f4156634c336c5177617759445652306642475177596a42676f46366758495a616148523063484d364c79396863476b7564484a316333526c5a484e6c0a636e5a705932567a4c6d6c75644756734c6d4e766253397a5a3367765932567964476c6d61574e6864476c76626939324e4339775932746a636d772f593245390a6347786864475a76636d306d5a57356a62325270626d63395a4756794d423047413155644467515742425331415177713739576f78466a6c34702b45446e54640a424b50314a54414f42674e56485138424166384542414d434273417744415944565230544151482f4241497741444343416a6f4743537147534962345451454e0a41515343416973776767496e4d42344743697147534962345451454e41514545454b53374c746d3263303867704f6f4e503452335a3367776767466b42676f710a686b69472b453042445145434d4949425644415142677371686b69472b45304244514543415149424244415142677371686b69472b45304244514543416749420a4244415142677371686b69472b4530424451454341774942416a415142677371686b69472b4530424451454342414942416a415142677371686b69472b4530420a44514543425149424254415242677371686b69472b4530424451454342674943415038774541594c4b6f5a496876684e4151304241676343415141774541594c0a4b6f5a496876684e4151304241676743415149774541594c4b6f5a496876684e4151304241676b43415141774541594c4b6f5a496876684e4151304241676f430a415141774541594c4b6f5a496876684e4151304241677343415141774541594c4b6f5a496876684e4151304241677743415141774541594c4b6f5a496876684e0a4151304241673043415141774541594c4b6f5a496876684e4151304241673443415141774541594c4b6f5a496876684e4151304241673843415141774541594c0a4b6f5a496876684e4151304241684143415141774541594c4b6f5a496876684e4151304241684543415130774877594c4b6f5a496876684e41513042416849450a45415145416749462f7741434141414141414141414141774541594b4b6f5a496876684e4151304241775143414141774641594b4b6f5a496876684e415130420a42415147494b4276414141414d41384743697147534962345451454e4151554b415145774867594b4b6f5a496876684e415130424267515147523579417a6b490a4749756b70764e535132516f546a424542676f71686b69472b453042445145484d4459774541594c4b6f5a496876684e4151304242774542416638774541594c0a4b6f5a496876684e4151304242774942416638774541594c4b6f5a496876684e4151304242774d4241663877436759494b6f5a497a6a304541774944535141770a526749684149564d434856714e4d4b765958412f335439612f4c6f316e376a72497976763849456757526f64557177734169454177586b69547535613646306d0a61786c45493776557461727868745865464d462f53383674777a37436f6b6f3d0a2d2d2d2d2d454e442043455254494649434154452d2d2d2d2d0a2d2d2d2d2d424547494e2043455254494649434154452d2d2d2d2d0a4d4949436c6a4343416a32674177494241674956414a567658633239472b487051456e4a3150517a7a674658433935554d416f4743437147534d343942414d430a4d476778476a415942674e5642414d4d45556c756447567349464e48574342536232393049454e424d526f77474159445651514b4442464a626e526c624342440a62334a7762334a6864476c76626a45554d424947413155454277774c553246756447456751327868636d4578437a414a42674e564241674d416b4e424d5173770a435159445651514745774a56557a4165467730784f4441314d6a45784d4455774d5442614677307a4d7a41314d6a45784d4455774d5442614d484178496a41670a42674e5642414d4d47556c756447567349464e4857434251513073675547786864475a76636d306751304578476a415942674e5642416f4d45556c75644756730a49454e76636e4276636d4630615739754d5251774567594456515148444174545957353059534244624746795954454c4d416b474131554543417743513045780a437a414a42674e5642415954416c56544d466b77457759484b6f5a497a6a3043415159494b6f5a497a6a304441516344516741454e53422f377432316c58534f0a3243757a7078773734654a423732457944476757357258437478327456544c7136684b6b367a2b5569525a436e71523770734f766771466553786c6d546c4a6c0a65546d693257597a33714f42757a43427544416642674e5648534d4547444157674251695a517a575770303069664f44744a5653763141624f536347724442530a42674e5648523845537a424a4d45656752614244686b466f64485277637a6f764c324e6c636e52705a6d6c6a5958526c63793530636e567a6447566b633256790a646d6c6a5a584d75615735305a577775593239744c306c756447567355306459556d397664454e424c6d526c636a416442674e5648513445466751556c5739640a7a62306234656c4153636e553944504f4156634c336c517744675944565230504151482f42415144416745474d42494741315564457745422f7751494d4159420a4166384341514177436759494b6f5a497a6a30454177494452774177524149675873566b6930772b6936565947573355462f32327561586530594a446a3155650a6e412b546a44316169356343494359623153416d4435786b66545670766f34556f79695359787244574c6d5552344349394e4b7966504e2b0a2d2d2d2d2d454e442043455254494649434154452d2d2d2d2d0a2d2d2d2d2d424547494e2043455254494649434154452d2d2d2d2d0a4d4949436a7a4343416a53674177494241674955496d554d316c71644e496e7a6737535655723951477a6b6e42717777436759494b6f5a497a6a3045417749770a614445614d4267474131554541777752535735305a5777675530645949464a766233516751304578476a415942674e5642416f4d45556c756447567349454e760a636e4276636d4630615739754d5251774567594456515148444174545957353059534244624746795954454c4d416b47413155454341774351304578437a414a0a42674e5642415954416c56544d423458445445344d4455794d5445774e4455784d466f58445451354d54497a4d54497a4e546b314f566f77614445614d4267470a4131554541777752535735305a5777675530645949464a766233516751304578476a415942674e5642416f4d45556c756447567349454e76636e4276636d46300a615739754d5251774567594456515148444174545957353059534244624746795954454c4d416b47413155454341774351304578437a414a42674e56424159540a416c56544d466b77457759484b6f5a497a6a3043415159494b6f5a497a6a3044415163445167414543366e45774d4449595a4f6a2f69505773437a61454b69370a314f694f534c52466857476a626e42564a66566e6b59347533496a6b4459594c304d784f346d717379596a6c42616c54565978465032734a424b357a6c4b4f420a757a43427544416642674e5648534d4547444157674251695a517a575770303069664f44744a5653763141624f5363477244425342674e5648523845537a424a0a4d45656752614244686b466f64485277637a6f764c324e6c636e52705a6d6c6a5958526c63793530636e567a6447566b63325679646d6c6a5a584d75615735300a5a577775593239744c306c756447567355306459556d397664454e424c6d526c636a416442674e564851344546675155496d554d316c71644e496e7a673753560a55723951477a6b6e4271777744675944565230504151482f42415144416745474d42494741315564457745422f7751494d4159424166384341514577436759490a4b6f5a497a6a3045417749445351417752674968414f572f35516b522b533943695344634e6f6f774c7550524c735747662f59693747535839344267775477670a41694541344a306c72486f4d732b586f356f2f7358364f39515778485241765a55474f6452513763767152586171493d0a2d2d2d2d2d454e442043455254494649434154452d2d2d2d2d0a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let quote = decode(quote_hex).unwrap();

    // use blocktime in production
    let now = 1756648272_u64; // Aug-31-2025

    // get compose hash from events
    let expected_compose_hash = event_log
        .iter()
        .find(|e| e["event"].as_str().unwrap() == "compose-hash")
        .unwrap()["digest"]
        .as_str()
        .unwrap();

    // verified report with rtmrs
    let result = verify::verify(&quote, &collateral, now).unwrap();
    let rtmr3 = encode(result.report.as_td10().unwrap().rt_mr3);

    // replay the rtmr3 and compose hash
    let replayed_rtmr3 = replay_rtmr(event_log.to_owned(), 3);
    let replayed_compose_hash: String =
        replay_app_compose(tcb_info["app_compose"].as_str().unwrap());

    // compose hash match expected
    assert!(replayed_compose_hash == expected_compose_hash);
    // event with compose hash matches report rtmr3
    assert!(replayed_rtmr3 == rtmr3);

    println!("replayed_rtmr3 {:?}", replayed_rtmr3);
    println!("replayed_compose_hash {:?}", replayed_compose_hash);

    let codehash = verify_codehash(tcb_info.to_string(), rtmr3);

    println!("codehash {:?}", codehash);
}
