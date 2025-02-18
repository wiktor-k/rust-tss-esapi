// Copyright 2020 Contributors to the Parsec project.
// SPDX-License-Identifier: Apache-2.0
use std::{convert::TryFrom, env, str::FromStr, sync::Once};

use tss_esapi::{
    abstraction::cipher::Cipher,
    attributes::{ObjectAttributesBuilder, SessionAttributesBuilder},
    constants::SessionType,
    interface_types::{
        algorithm::HashingAlgorithm, resource_handles::Hierarchy, session_handles::PolicySession,
    },
    structures::{Digest, MaxBuffer, PcrSelectionListBuilder, PcrSlot, SymmetricDefinition},
    tcti_ldr::TctiNameConf,
    tss2_esys::{TPM2B_PUBLIC, TPMU_PUBLIC_PARMS},
    utils, Context,
};

#[allow(dead_code)]
pub const HASH: [u8; 64] = [
    0x69, 0x3E, 0xDB, 0x1B, 0x22, 0x79, 0x03, 0xF4, 0xC0, 0xBF, 0xD6, 0x91, 0x76, 0x37, 0x84, 0x69,
    0x3E, 0xDB, 0x1B, 0x22, 0x79, 0x03, 0xF4, 0xC0, 0xBF, 0xD6, 0x91, 0x76, 0x37, 0x84, 0xA2, 0x94,
    0x8E, 0x92, 0x50, 0x35, 0xC2, 0x8C, 0x5C, 0x3C, 0xCA, 0xFE, 0x18, 0xE8, 0x81, 0xA2, 0x94, 0x8E,
    0x92, 0x50, 0x35, 0xC2, 0x8C, 0x5C, 0x3C, 0xCA, 0xFE, 0x18, 0xE8, 0x81, 0x37, 0x78, 0x37, 0x78,
];

#[allow(dead_code)]
pub const KEY: [u8; 512] = [
    231, 97, 201, 180, 0, 1, 185, 150, 85, 90, 174, 188, 105, 133, 188, 3, 206, 5, 222, 71, 185, 1,
    209, 243, 36, 130, 250, 116, 17, 0, 24, 4, 25, 225, 250, 198, 245, 210, 140, 23, 139, 169, 15,
    193, 4, 145, 52, 138, 149, 155, 238, 36, 74, 152, 179, 108, 200, 248, 250, 100, 115, 214, 166,
    165, 1, 27, 51, 11, 11, 244, 218, 157, 3, 174, 171, 142, 45, 8, 9, 36, 202, 171, 165, 43, 208,
    186, 232, 15, 241, 95, 81, 174, 189, 30, 213, 47, 86, 115, 239, 49, 214, 235, 151, 9, 189, 174,
    144, 238, 200, 201, 241, 157, 43, 37, 6, 96, 94, 152, 159, 205, 54, 9, 181, 14, 35, 246, 49,
    150, 163, 118, 242, 59, 54, 42, 221, 215, 248, 23, 18, 223, 179, 229, 0, 204, 65, 69, 166, 180,
    11, 49, 131, 96, 163, 96, 158, 7, 109, 119, 208, 17, 237, 125, 187, 121, 94, 65, 2, 86, 105,
    68, 51, 197, 73, 108, 185, 231, 126, 199, 81, 1, 251, 211, 45, 47, 15, 113, 135, 197, 152, 239,
    180, 111, 18, 192, 136, 222, 11, 99, 41, 248, 205, 253, 209, 56, 214, 32, 225, 3, 49, 161, 58,
    57, 190, 69, 86, 95, 185, 184, 155, 76, 8, 122, 104, 81, 222, 234, 246, 40, 98, 182, 90, 160,
    111, 74, 102, 36, 148, 99, 69, 207, 214, 104, 87, 128, 238, 26, 121, 107, 166, 4, 64, 5, 210,
    164, 162, 189, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0,
];

static LOG_INIT: Once = Once::new();
#[allow(dead_code)]
pub fn setup_logging() {
    LOG_INIT.call_once(|| {
        env_logger::init();
    });
}

#[allow(dead_code)]
pub fn create_tcti() -> TctiNameConf {
    setup_logging();

    match env::var("TEST_TCTI") {
        Err(_) => TctiNameConf::Mssim(Default::default()),
        Ok(tctistr) => TctiNameConf::from_str(&tctistr).expect("Error parsing TEST_TCTI"),
    }
}

#[allow(dead_code)]
pub fn create_ctx_without_session() -> Context {
    let tcti = create_tcti();
    Context::new(tcti).unwrap()
}

#[allow(dead_code)]
pub fn create_ctx_with_session() -> Context {
    let mut ctx = create_ctx_without_session();
    let session = ctx
        .start_auth_session(
            None,
            None,
            None,
            SessionType::Hmac,
            SymmetricDefinition::AES_256_CFB,
            HashingAlgorithm::Sha256,
        )
        .unwrap();
    let (session_attributes, session_attributes_mask) = SessionAttributesBuilder::new()
        .with_decrypt(true)
        .with_encrypt(true)
        .build();
    ctx.tr_sess_set_attributes(
        session.unwrap(),
        session_attributes,
        session_attributes_mask,
    )
    .unwrap();
    ctx.set_sessions((session, None, None));

    ctx
}

#[allow(dead_code)]
pub fn decryption_key_pub() -> TPM2B_PUBLIC {
    utils::create_restricted_decryption_rsa_public(Cipher::aes_256_cfb(), 2048, 0).unwrap()
}

#[allow(dead_code)]
pub fn encryption_decryption_key_pub() -> TPM2B_PUBLIC {
    utils::create_unrestricted_encryption_decryption_rsa_public(2048, 0).unwrap()
}

#[allow(dead_code)]
pub fn signing_key_pub() -> TPM2B_PUBLIC {
    utils::create_unrestricted_signing_rsa_public(
        utils::AsymSchemeUnion::RSASSA(HashingAlgorithm::Sha256),
        2048,
        0,
    )
    .unwrap()
}

#[allow(dead_code)]
pub fn get_pcr_policy_digest(
    context: &mut Context,
    mangle: bool,
    do_trial: bool,
) -> (Digest, PolicySession) {
    let old_ses = context.sessions();
    context.clear_sessions();

    // Read the pcr values using pcr_read
    let pcr_selection_list = PcrSelectionListBuilder::new()
        .with_selection(HashingAlgorithm::Sha256, &[PcrSlot::Slot0, PcrSlot::Slot1])
        .build();

    let (_update_counter, pcr_selection_list_out, pcr_data) =
        context.pcr_read(&pcr_selection_list).unwrap();

    assert_eq!(pcr_selection_list, pcr_selection_list_out);
    // Run pcr_policy command.
    //
    // "If this command is used for a trial policySession,
    // policySession→policyDigest will be updated using the
    // values from the command rather than the values from a digest of the TPM PCR."
    //
    // "TPM2_Quote() and TPM2_PolicyPCR() digest the concatenation of PCR."
    let mut concatenated_pcr_values = [
        pcr_data
            .pcr_bank(HashingAlgorithm::Sha256)
            .unwrap()
            .pcr_value(PcrSlot::Slot0)
            .unwrap()
            .value(),
        pcr_data
            .pcr_bank(HashingAlgorithm::Sha256)
            .unwrap()
            .pcr_value(PcrSlot::Slot1)
            .unwrap()
            .value(),
    ]
    .concat();

    if mangle {
        concatenated_pcr_values[0] = 0x00;
    }

    let (hashed_data, _ticket) = context
        .hash(
            &MaxBuffer::try_from(concatenated_pcr_values.to_vec()).unwrap(),
            HashingAlgorithm::Sha256,
            Hierarchy::Owner,
        )
        .unwrap();

    if do_trial {
        // Create a trial policy session to use in calls to the policy
        // context methods.
        let trial_policy_auth_session = context
            .start_auth_session(
                None,
                None,
                None,
                SessionType::Trial,
                SymmetricDefinition::AES_256_CFB,
                HashingAlgorithm::Sha256,
            )
            .expect("Start auth session failed")
            .expect("Start auth session returned a NONE handle");

        let (trial_policy_auth_session_attributes, trial_policy_auth_session_attributes_mask) =
            SessionAttributesBuilder::new()
                .with_decrypt(true)
                .with_encrypt(true)
                .build();
        context
            .tr_sess_set_attributes(
                trial_policy_auth_session,
                trial_policy_auth_session_attributes,
                trial_policy_auth_session_attributes_mask,
            )
            .expect("tr_sess_set_attributes call failed");

        let trial_policy_session = PolicySession::try_from(trial_policy_auth_session)
            .expect("Failed to convert auth session into policy session");
        // There should be no errors setting pcr policy for trial session.
        context
            .policy_pcr(trial_policy_session, &hashed_data, pcr_selection_list)
            .expect("Failed to call policy pcr");

        // There is now a policy digest that can be retrived and used.
        let digest = context
            .policy_get_digest(trial_policy_session)
            .expect("Failed to call policy_get_digest");

        // Restore old sessions
        context.set_sessions(old_ses);

        (digest, trial_policy_session)
    } else {
        // Create a policy session to use in calls to the policy
        // context methods.
        let policy_auth_session = context
            .start_auth_session(
                None,
                None,
                None,
                SessionType::Policy,
                SymmetricDefinition::AES_256_CFB,
                HashingAlgorithm::Sha256,
            )
            .expect("Start auth session failed")
            .expect("Start auth session returned a NONE handle");

        let (policy_auth_session_attributes, policy_auth_session_attributes_mask) =
            SessionAttributesBuilder::new()
                .with_decrypt(true)
                .with_encrypt(true)
                .build();
        context
            .tr_sess_set_attributes(
                policy_auth_session,
                policy_auth_session_attributes,
                policy_auth_session_attributes_mask,
            )
            .expect("tr_sess_set_attributes call failed");

        let policy_session = PolicySession::try_from(policy_auth_session)
            .expect("Failed to convert auth session into policy session");
        // There should be no errors setting pcr policy for trial session.
        context
            .policy_pcr(policy_session, &hashed_data, pcr_selection_list)
            .expect("Failed to call policy_pcr");

        // There is now a policy digest that can be retrived and used.
        let digest = context
            .policy_get_digest(policy_session)
            .expect("Failed to call policy_get_digest");

        // Restore old sessions
        context.set_sessions(old_ses);

        (digest, policy_session)
    }
}

#[allow(dead_code)]
pub fn create_public_sealed_object() -> tss_esapi::tss2_esys::TPM2B_PUBLIC {
    let object_attributes = ObjectAttributesBuilder::new()
        .with_fixed_tpm(true)
        .with_fixed_parent(true)
        .with_no_da(true)
        .with_admin_with_policy(true)
        .with_user_with_auth(true)
        .build()
        .expect("Failed to create object attributes");

    let mut params: TPMU_PUBLIC_PARMS = Default::default();
    params.keyedHashDetail.scheme.scheme = tss_esapi::constants::tss::TPM2_ALG_NULL;

    tss_esapi::tss2_esys::TPM2B_PUBLIC {
        size: std::mem::size_of::<tss_esapi::tss2_esys::TPMT_PUBLIC>() as u16,
        publicArea: tss_esapi::tss2_esys::TPMT_PUBLIC {
            type_: tss_esapi::constants::tss::TPM2_ALG_KEYEDHASH,
            nameAlg: tss_esapi::constants::tss::TPM2_ALG_SHA256,
            objectAttributes: object_attributes.0,
            authPolicy: Default::default(),
            parameters: params,
            unique: Default::default(),
        },
    }
}
