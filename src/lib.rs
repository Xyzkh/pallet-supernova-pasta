use anyhow::Result;
use serde::Deserialize;
use std::fs::File;

// base64 0.22 (Engine API)
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;

// ==== Schema file JSON ====
#[derive(Deserialize)]
pub struct VkFile {
    pub format: String, // "supernova_v1"
    pub curve: String,  // "pasta"
    pub vk_b64: String, // base64(bincode(PublicParams))
}

#[derive(Deserialize)]
pub struct ProofFile {
    pub format: String, // "supernova_v1"
    pub curve: String,  // "pasta"
    pub num_steps: u64,
    pub proof_b64: String, // base64(bincode(RecursiveSNARK))
}

// === IO util ===
pub fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T> {
    let f = File::open(path)?;
    let mut de = serde_json::Deserializer::from_reader(f);
    let v = serde_path_to_error::deserialize(&mut de)
        .map_err(|e| anyhow::anyhow!("{} parse error at {}: {}", path, e.path(), e))?;
    Ok(v)
}

pub fn decode_b64<T: serde::de::DeserializeOwned>(b64s: &str) -> Result<T> {
    let bytes = B64.decode(b64s)?;
    let obj: T = bincode::deserialize(&bytes)?;
    Ok(obj)
}

// ===== Helper: encode field -> hex (tanpa 0x) untuk pembandingan simple =====
use ff::PrimeField;
pub fn f_to_hex<F: PrimeField>(x: F) -> String {
    let bytes = x.to_repr();
    hex::encode(bytes.as_ref())
}

// ====== Circuit yang sama dengan generator (Fib [a,b] -> [b,a+b]) ======
use nova_snark::frontend::{num::AllocatedNum, ConstraintSystem, SynthesisError};
use nova_snark::traits::circuit::StepCircuit;

#[derive(Clone, Default)]
pub struct FibStep;

impl<F: PrimeField> StepCircuit<F> for FibStep {
    fn arity(&self) -> usize {
        2
    }

    fn synthesize<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        z: &[AllocatedNum<F>],
    ) -> Result<Vec<AllocatedNum<F>>, SynthesisError> {
        assert_eq!(z.len(), 2, "FibStep expects arity=2");
        let z0 = &z[0];
        let z1 = &z[1];

        let sum = AllocatedNum::alloc(cs.namespace(|| "sum"), || {
            let a = z0.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let b = z1.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            Ok(a + b)
        })?;

        // (z0 + z1) * 1 = sum
        cs.enforce(
            || "z0 + z1 = sum",
            |lc| lc + z0.get_variable() + z1.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + sum.get_variable(),
        );

        Ok(vec![z1.clone(), sum])
    }
}
