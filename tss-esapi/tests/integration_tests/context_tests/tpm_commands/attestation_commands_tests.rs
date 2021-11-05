// Copyright 2021 Contributors to the Parsec project.
// SPDX-License-Identifier: Apache-2.0
mod test_quote {
    use crate::common::{create_ctx_with_session, signing_key_pub};
    use std::convert::TryFrom;
    use tss_esapi::{
        interface_types::{algorithm::HashingAlgorithm, resource_handles::Hierarchy},
        structures::{Data, PcrSelectionListBuilder, PcrSlot, SignatureScheme},
    };

    #[test]
    fn pcr_quote() {
        let mut context = create_ctx_with_session();
        // Quote PCR 0
        let pcr_selection_list = PcrSelectionListBuilder::new()
            .with_selection(HashingAlgorithm::Sha256, &[PcrSlot::Slot0])
            .build();
        // No qualifying data
        let qualifying_data = vec![0xff; 16];

        let key_handle = context
            .create_primary(Hierarchy::Owner, &signing_key_pub(), None, None, None, None)
            .unwrap()
            .key_handle;

        let res = context
            .quote(
                key_handle,
                &Data::try_from(qualifying_data).unwrap(),
                SignatureScheme::Null,
                pcr_selection_list,
            )
            .expect("Failed to get a quote");
        assert!(res.0.size != 0);
    }
}
