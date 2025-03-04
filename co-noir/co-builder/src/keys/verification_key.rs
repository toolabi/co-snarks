use crate::{
    builder::{GenericUltraCircuitBuilder, UltraCircuitBuilder},
    crs::{parse::CrsParser, Crs, ProverCrs},
    honk_curve::HonkCurve,
    keys::proving_key::ProvingKey,
    polynomials::polynomial_types::{PrecomputedEntities, PRECOMPUTED_ENTITIES_SIZE},
    serialize::{Serialize, SerializeP},
    types::types::{AggregationObjectPubInputIndices, AGGREGATION_OBJECT_SIZE},
    utils::Utils,
    HonkProofError, HonkProofResult, TranscriptFieldType,
};
use ark_ec::pairing::Pairing;
use co_acvm::mpc::NoirWitnessExtensionProtocol;
use eyre::Result;

pub struct VerifyingKey<P: Pairing> {
    pub crs: P::G2Affine,
    pub circuit_size: u32,
    pub num_public_inputs: u32,
    pub pub_inputs_offset: u32,
    pub commitments: PrecomputedEntities<P::G1Affine>,
}

impl<P: Pairing> VerifyingKey<P> {
    pub fn create(circuit: UltraCircuitBuilder<P>, crs: Crs<P>) -> HonkProofResult<Self> {
        let (_, vk) = circuit.create_keys(crs)?;
        Ok(vk)
    }

    pub fn from_barrettenberg_and_crs(
        barretenberg_vk: VerifyingKeyBarretenberg<P>,
        crs: P::G2Affine,
    ) -> Self {
        Self {
            crs,
            circuit_size: barretenberg_vk.circuit_size as u32,
            num_public_inputs: barretenberg_vk.num_public_inputs as u32,
            pub_inputs_offset: barretenberg_vk.pub_inputs_offset as u32,
            commitments: barretenberg_vk.commitments,
        }
    }

    pub fn get_crs<T: NoirWitnessExtensionProtocol<P::ScalarField>>(
        circuit: &GenericUltraCircuitBuilder<P, T>,
        path_g1: &str,
        path_g2: &str,
    ) -> Result<Crs<P>> {
        tracing::trace!("Getting crs");
        ProvingKey::get_crs(circuit, path_g1, path_g2)
    }

    pub fn get_prover_crs<T: NoirWitnessExtensionProtocol<P::ScalarField>>(
        circuit: &GenericUltraCircuitBuilder<P, T>,
        path_g1: &str,
    ) -> Result<ProverCrs<P>> {
        tracing::trace!("Getting crs");
        ProvingKey::get_prover_crs(circuit, path_g1)
    }

    pub fn get_verifier_crs(path_g2: &str) -> Result<P::G2Affine> {
        tracing::trace!("Getting verifier crs");
        CrsParser::<P>::get_crs_g2(path_g2)
    }
}

pub struct VerifyingKeyBarretenberg<P: Pairing> {
    pub(crate) circuit_size: u64,
    pub(crate) log_circuit_size: u64,
    pub(crate) num_public_inputs: u64,
    pub(crate) pub_inputs_offset: u64,
    pub(crate) contains_recursive_proof: bool,
    pub(crate) recursive_proof_public_input_indices: AggregationObjectPubInputIndices,
    pub(crate) commitments: PrecomputedEntities<P::G1Affine>,
}

impl<P: HonkCurve<TranscriptFieldType>> VerifyingKeyBarretenberg<P> {
    const FIELDSIZE_BYTES: u32 = SerializeP::<P>::FIELDSIZE_BYTES;
    const SER_FULL_SIZE: usize = 4 * 8
        + 1
        + AGGREGATION_OBJECT_SIZE * 4
        + PRECOMPUTED_ENTITIES_SIZE * 2 * Self::FIELDSIZE_BYTES as usize;
    const SER_COMPRESSED_SIZE: usize = Self::SER_FULL_SIZE - 1 - AGGREGATION_OBJECT_SIZE * 4;

    pub fn to_buffer(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(Self::SER_FULL_SIZE);

        Serialize::<P::ScalarField>::write_u64(&mut buffer, self.circuit_size);
        Serialize::<P::ScalarField>::write_u64(&mut buffer, self.log_circuit_size);
        Serialize::<P::ScalarField>::write_u64(&mut buffer, self.num_public_inputs);
        Serialize::<P::ScalarField>::write_u64(&mut buffer, self.pub_inputs_offset);
        Serialize::<P::ScalarField>::write_u8(&mut buffer, self.contains_recursive_proof as u8);

        for val in self.recursive_proof_public_input_indices.iter() {
            Serialize::<P::ScalarField>::write_u32(&mut buffer, *val);
        }

        for el in self.commitments.iter() {
            SerializeP::<P>::write_g1_element(&mut buffer, el, true);
        }

        debug_assert_eq!(buffer.len(), Self::SER_FULL_SIZE);
        buffer
    }
    // BB for Keccak doesn't use the recursive stuff in the vk
    pub fn to_buffer_keccak(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(Self::SER_COMPRESSED_SIZE);

        Serialize::<P::ScalarField>::write_u64(&mut buffer, self.circuit_size);
        Serialize::<P::ScalarField>::write_u64(&mut buffer, self.log_circuit_size);
        Serialize::<P::ScalarField>::write_u64(&mut buffer, self.num_public_inputs);
        Serialize::<P::ScalarField>::write_u64(&mut buffer, self.pub_inputs_offset);

        for el in self.commitments.iter() {
            SerializeP::<P>::write_g1_element(&mut buffer, el, true);
        }

        debug_assert_eq!(buffer.len(), Self::SER_COMPRESSED_SIZE);
        buffer
    }

    pub fn from_buffer(buf: &[u8]) -> HonkProofResult<Self> {
        let size = buf.len();
        let mut offset = 0;

        if size != Self::SER_FULL_SIZE && size != Self::SER_COMPRESSED_SIZE {
            return Err(HonkProofError::InvalidKeyLength);
        }

        // Read data
        let circuit_size = Serialize::<P::ScalarField>::read_u64(buf, &mut offset);
        let log_circuit_size = Serialize::<P::ScalarField>::read_u64(buf, &mut offset);
        if log_circuit_size != Utils::get_msb64(circuit_size) as u64 {
            return Err(HonkProofError::CorruptedKey);
        }
        let num_public_inputs = Serialize::<P::ScalarField>::read_u64(buf, &mut offset);
        let pub_inputs_offset = Serialize::<P::ScalarField>::read_u64(buf, &mut offset);

        let (contains_recursive_proof, recursive_proof_public_input_indices) =
            if size == Self::SER_FULL_SIZE {
                let contains_recursive_proof_u8 =
                    Serialize::<P::ScalarField>::read_u8(buf, &mut offset);
                if contains_recursive_proof_u8 > 1 {
                    return Err(HonkProofError::CorruptedKey);
                }

                let mut recursive_proof_public_input_indices =
                    AggregationObjectPubInputIndices::default();
                for val in recursive_proof_public_input_indices.iter_mut() {
                    *val = Serialize::<P::ScalarField>::read_u32(buf, &mut offset);
                }
                (
                    contains_recursive_proof_u8 == 1,
                    recursive_proof_public_input_indices,
                )
            } else {
                (false, Default::default())
            };

        let mut commitments = PrecomputedEntities::default();

        for el in commitments.iter_mut() {
            *el = SerializeP::<P>::read_g1_element(buf, &mut offset, true);
        }

        debug_assert!(offset == Self::SER_FULL_SIZE || offset == Self::SER_COMPRESSED_SIZE);

        Ok(Self {
            circuit_size,
            log_circuit_size,
            num_public_inputs,
            pub_inputs_offset,
            contains_recursive_proof,
            recursive_proof_public_input_indices,
            commitments,
        })
    }
}
