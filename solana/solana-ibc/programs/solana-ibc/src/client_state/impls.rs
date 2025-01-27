//! Implementation of IBC traits for [`AnyClientState`].
//!
//! We cannot use [`::ibc::derive::ClientState`] derive because we need a custom
//! implementation for `verify_client_message` which uses custom signature
//! verifier.

use anchor_lang::solana_program;

use super::AnyClientState;
use crate::ibc;
use crate::storage::IbcStorage;

type Result<T = (), E = ibc::ClientError> = core::result::Result<T, E>;

macro_rules! delegate {
    (fn $name:ident(&self $(, $arg:ident: $ty:ty)* $(,)?) -> $ret:ty) => {
        fn $name(&self, $($arg: $ty),*) -> $ret {
            match self {
                AnyClientState::Tendermint(cs) => cs.$name($($arg),*),
                AnyClientState::Wasm(_) => unimplemented!(),
                AnyClientState::Rollup(cs) => cs.$name($($arg),*),
                AnyClientState::Guest(cs) => cs.$name($($arg),*),
                #[cfg(any(test, feature = "mocks"))]
                AnyClientState::Mock(cs) => cs.$name($($arg),*),
            }
        }
    }
}

impl ibc::ClientStateCommon for AnyClientState {
    delegate!(fn verify_consensus_state(&self, consensus_state: ibc::Any) -> Result);
    delegate!(fn client_type(&self) -> ibc::ClientType);
    delegate!(fn latest_height(&self) -> ibc::Height);
    delegate!(fn validate_proof_height(&self, proof_height: ibc::Height) -> Result);

    fn verify_upgrade_client(
        &self,
        upgraded_client_state: ibc::Any,
        upgraded_consensus_state: ibc::Any,
        proof_upgrade_client: ibc::CommitmentProofBytes,
        proof_upgrade_consensus_state: ibc::CommitmentProofBytes,
        root: &ibc::CommitmentRoot,
    ) -> Result {
        match self {
            AnyClientState::Tendermint(cs) => {
                ibc::tm::client_state::verify_upgrade_client::<
                    SolanaHostFunctions,
                >(
                    cs.inner(),
                    upgraded_client_state,
                    upgraded_consensus_state,
                    proof_upgrade_client,
                    proof_upgrade_consensus_state,
                    root,
                )
            }
            AnyClientState::Wasm(_) => unimplemented!(),
            AnyClientState::Rollup(_) => unimplemented!(),
            AnyClientState::Guest(_) => unimplemented!(),
            #[cfg(any(test, feature = "mocks"))]
            AnyClientState::Mock(cs) => cs.verify_upgrade_client(
                upgraded_client_state,
                upgraded_consensus_state,
                proof_upgrade_client,
                proof_upgrade_consensus_state,
                root,
            ),
        }
    }

    fn verify_membership(
        &self,
        prefix: &ibc::CommitmentPrefix,
        proof: &ibc::CommitmentProofBytes,
        root: &ibc::CommitmentRoot,
        path: ibc::path::Path,
        value: Vec<u8>,
    ) -> Result {
        match self {
            AnyClientState::Tendermint(cs) => {
                ibc::tm::client_state::verify_membership::<SolanaHostFunctions>(
                    &cs.inner().proof_specs,
                    prefix,
                    proof,
                    root,
                    path,
                    value,
                )
            }
            AnyClientState::Wasm(_) => unimplemented!(),
            AnyClientState::Rollup(cs) => {
                cs.verify_membership(prefix, proof, root, path, value)
            }
            AnyClientState::Guest(cs) => {
                cs.verify_membership(prefix, proof, root, path, value)
            }
            #[cfg(any(test, feature = "mocks"))]
            AnyClientState::Mock(cs) => {
                cs.verify_membership(prefix, proof, root, path, value)
            }
        }
    }

    fn verify_non_membership(
        &self,
        prefix: &ibc::CommitmentPrefix,
        proof: &ibc::CommitmentProofBytes,
        root: &ibc::CommitmentRoot,
        path: ibc::path::Path,
    ) -> Result {
        match self {
            AnyClientState::Tendermint(cs) => {
                ibc::tm::client_state::verify_non_membership::<
                    SolanaHostFunctions,
                >(
                    &cs.inner().proof_specs, prefix, proof, root, path
                )
            }
            AnyClientState::Wasm(_) => unimplemented!(),
            AnyClientState::Rollup(cs) => {
                cs.verify_non_membership(prefix, proof, root, path)
            }
            AnyClientState::Guest(cs) => {
                cs.verify_non_membership(prefix, proof, root, path)
            }
            #[cfg(any(test, feature = "mocks"))]
            AnyClientState::Mock(cs) => {
                cs.verify_non_membership(prefix, proof, root, path)
            }
        }
    }
}

impl<'a, 'b> ibc::ClientStateValidation<IbcStorage<'a, 'b>> for AnyClientState {
    fn verify_client_message(
        &self,
        ctx: &IbcStorage<'a, 'b>,
        client_id: &ibc::ClientId,
        client_message: ibc::Any,
    ) -> Result {
        match self {
            AnyClientState::Tendermint(cs) => {
                ibc::tm::client_state::verify_client_message::<
                    _,
                    SolanaHostFunctions,
                >(
                    cs.inner(), ctx, client_id, client_message, &tm::TmVerifier
                )
            }
            AnyClientState::Wasm(_) => unimplemented!(),
            AnyClientState::Rollup(cs) => {
                cs.verify_client_message(ctx, client_id, client_message)
            }
            AnyClientState::Guest(cs) => {
                cs.verify_client_message(ctx, client_id, client_message)
            }
            #[cfg(any(test, feature = "mocks"))]
            AnyClientState::Mock(cs) => {
                cs.verify_client_message(ctx, client_id, client_message)
            }
        }
    }

    fn verify_tm_client_message(
        &self,
        ctx: &IbcStorage<'a, 'b>,
        client_id: &ibc::ClientId,
        client_message: Option<ibc_client_tendermint_types::Header>,
    ) -> Result {
        match self {
            AnyClientState::Tendermint(cs) => {
                ibc::tm::client_state::verify_tm_client_message::<
                    _,
                    SolanaHostFunctions,
                >(
                    cs.inner(), ctx, client_id, client_message, &tm::TmVerifier
                )
            }
            AnyClientState::Wasm(_) => unimplemented!(),
            AnyClientState::Rollup(_) => unimplemented!(),
            AnyClientState::Guest(_) => unimplemented!(),
            #[cfg(any(test, feature = "mocks"))]
            AnyClientState::Mock(_) => unimplemented!(),
        }
    }

    delegate!(fn check_for_misbehaviour(
        &self,
        ctx: &IbcStorage<'a, 'b>,
        client_id: &ibc::ClientId,
        client_message: ibc::Any,
    ) -> Result<bool>);
    delegate!(fn check_for_tm_misbehaviour(
        &self,
        ctx: &IbcStorage<'a, 'b>,
        client_id: &ibc::ClientId,
        client_message: Option<ibc_client_tendermint_types::Header>,
    ) -> Result<bool>);
    delegate!(fn status(
        &self,
        ctx: &IbcStorage<'a, 'b>,
        client_id: &ibc::ClientId,
    ) -> Result<ibc::Status>);
}

impl<'a, 'b> ibc::ClientStateExecution<IbcStorage<'a, 'b>> for AnyClientState {
    delegate!(fn initialise(
        &self,
        ctx: &mut IbcStorage<'a, 'b>,
        client_id: &ibc::ClientId,
        consensus_state: ibc::Any,
    ) -> Result);
    delegate!(fn update_state(
        &self,
        ctx: &mut IbcStorage<'a, 'b>,
        client_id: &ibc::ClientId,
        header: ibc::Any,
    ) -> Result<Vec<ibc::Height>>);
    delegate!(fn update_tm_state(
        &self,
        ctx: &mut IbcStorage<'a, 'b>,
        client_id: &ibc::ClientId,
        header: Option<ibc_client_tendermint_types::Header>,
    ) -> Result<Vec<ibc::Height>>);
    delegate!(fn update_state_on_misbehaviour(
        &self,
        ctx: &mut IbcStorage<'a, 'b>,
        client_id: &ibc::ClientId,
        client_message: ibc::Any,
    ) -> Result);
    delegate!(fn update_state_on_upgrade(
        &self,
        ctx: &mut IbcStorage<'a, 'b>,
        client_id: &ibc::ClientId,
        upgraded_client_state: ibc::Any,
        upgraded_consensus_state: ibc::Any,
    ) -> Result<ibc::Height>);
}

mod tm {
    use tendermint::crypto::signature::Error;
    use tendermint_light_client_verifier::operations::commit_validator::ProdCommitValidator;
    use tendermint_light_client_verifier::operations::voting_power::ProvidedVotingPowerCalculator;
    use tendermint_light_client_verifier::predicates::VerificationPredicates;
    use tendermint_light_client_verifier::PredicateVerifier;

    pub(super) struct TmVerifier;
    pub(super) struct SigVerifier;

    #[derive(Default)]
    pub(super) struct InnerProdPredicates;

    impl crate::ibc::tm::TmVerifier for TmVerifier {
        type Verifier = PredicateVerifier<
            InnerProdPredicates,
            ProvidedVotingPowerCalculator<SigVerifier>,
            ProdCommitValidator,
        >;
        fn verifier(&self) -> Self::Verifier { Default::default() }
    }

    impl VerificationPredicates for InnerProdPredicates {
        type Sha256 = super::SolanaHostFunctions;
    }

    impl tendermint::crypto::signature::Verifier for SigVerifier {
        fn verify(
            pubkey: tendermint::PublicKey,
            msg: &[u8],
            signature: &tendermint::Signature,
        ) -> Result<(), Error> {
            let pubkey = match pubkey {
                tendermint::PublicKey::Ed25519(pubkey) => pubkey,
                _ => return Err(Error::UnsupportedKeyType),
            };
            let pubkey = <&[u8; 32]>::try_from(pubkey.as_bytes())
                .map_err(|_| Error::MalformedPublicKey)?;
            let sig = <&[u8; 64]>::try_from(signature.as_bytes())
                .map_err(|_| Error::MalformedSignature)?;
            if let Some(verifier) = crate::global().verifier() {
                if verifier.verify(msg, pubkey, sig).unwrap_or(false) {
                    return Ok(());
                }
            }
            Err(Error::VerificationFailed)
        }
    }
}

#[derive(Default)]
struct SolanaHostFunctions;

impl ibc::HostFunctionsProvider for SolanaHostFunctions {
    fn sha2_256(message: &[u8]) -> [u8; 32] {
        solana_program::hash::hash(message).to_bytes()
    }

    fn keccak_256(_message: &[u8]) -> [u8; 32] {
        // Solana deploy break if code includes call to a syscall which is not
        // implemented on the blockchain (even if the call is never executed).
        // Even though Solana supports Kaccek, don’t implement it so that we
        // don’t risk breaking deployment.
        //solana_program::keccak::hash(message).0
        unimplemented!()
    }

    fn blake3(_message: &[u8]) -> [u8; 32] {
        // Solana deploy break if code includes call to a syscall which is not
        // implemented on the blockchain (even if the call is never executed).
        // Even though Solana supports Blake3, don’t implement it since as of
        // May 2024, it’s not enabled in mainnet.
        //solana_program::blake3::hash(message).0
        unimplemented!()
    }

    fn sha2_512(_message: &[u8]) -> [u8; 64] { unimplemented!() }
    fn sha2_512_truncated(_message: &[u8]) -> [u8; 32] { unimplemented!() }
    fn ripemd160(_message: &[u8]) -> [u8; 20] { unimplemented!() }
    fn blake2b_512(_message: &[u8]) -> [u8; 64] { unimplemented!() }
    fn blake2s_256(_message: &[u8]) -> [u8; 32] { unimplemented!() }
}

#[cfg(all(test, not(miri)))]
mod test_host_functions {
    use ibc::HostFunctionsProvider;

    use crate::ibc;

    macro_rules! test {
        ($func:ident) => {
            #[test]
            fn $func() {
                for input in ["", "foo", "bar"] {
                    assert_eq!(
                        ibc::HostFunctionsManager::$func(input.as_bytes()),
                        super::SolanaHostFunctions::$func(input.as_bytes()),
                    );
                }
            }
        };
    }

    test!(sha2_256);
    // test!(keccak_256);
    // test!(blake3);
}

impl tendermint::crypto::Sha256 for SolanaHostFunctions {
    fn digest(data: impl AsRef<[u8]>) -> [u8; 32] {
        <Self as ibc::HostFunctionsProvider>::sha2_256(data.as_ref())
    }
}

#[test]
fn test_sha256() {
    use tendermint::crypto::default::Sha256;
    use tendermint::crypto::Sha256 as _;

    for input in ["".as_bytes(), "foo".as_bytes(), "bar".as_bytes()] {
        assert_eq!(Sha256::digest(input), SolanaHostFunctions::digest(input));
    }
}

impl tendermint::merkle::MerkleHash for SolanaHostFunctions {
    fn empty_hash(&mut self) -> [u8; 32] {
        // This is sha256("").  test_merkle_hash below verifies that this is
        // correct.
        hex_literal::hex!("e3b0c44298fc1c14 9afbf4c8996fb924"
                          "27ae41e4649b934c a495991b7852b855")
    }

    fn leaf_hash(&mut self, bytes: &[u8]) -> [u8; 32] {
        solana_program::hash::hashv(&[&[0], bytes]).to_bytes()
    }

    fn inner_hash(&mut self, left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
        solana_program::hash::hashv(&[&[1], &left, &right]).to_bytes()
    }
}

#[test]
fn test_merkle_hash() {
    use tendermint::crypto::default::Sha256;
    use tendermint::crypto::Sha256 as _;
    use tendermint::merkle::MerkleHash as _;

    let mut theirs = tendermint::merkle::NonIncremental::<Sha256>::default();
    let mut ours = SolanaHostFunctions;

    assert_eq!(Sha256::digest(b""), ours.empty_hash());
    assert_eq!(theirs.empty_hash(), ours.empty_hash());

    for input in ["".as_bytes(), "foo".as_bytes(), "bar".as_bytes()] {
        assert_eq!(theirs.leaf_hash(input), ours.leaf_hash(input));
    }

    let one = Sha256::digest(b"foo");
    let two = Sha256::digest(b"bar");
    assert_eq!(theirs.inner_hash(one, two), ours.inner_hash(one, two));
}
