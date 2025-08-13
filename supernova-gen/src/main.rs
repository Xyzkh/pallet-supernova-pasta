use anyhow::Result;
use serde::Serialize;
use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

use bincode;
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;

// === nova-snark frontend ===
use nova_snark::frontend::{
    num::AllocatedNum,
    ConstraintSystem,
    SynthesisError,
};
use nova_snark::{
    nova::{PublicParams, RecursiveSNARK},
    provider::{PallasEngine, VestaEngine},
    traits::{circuit::StepCircuit, Engine},
    traits::snark::default_ck_hint,
};

use ff::{PrimeField, Field};
use hex;

// ---------------- JSON schemas ----------------
#[derive(Serialize)]
struct VkJson {
    format: String,     // "supernova_v1"
    curve: String,      // "pasta"
    vk_b64: String,     // base64(bincode(PublicParams))
}

#[derive(Serialize)]
struct ProofJson {
    format: String,     // "supernova_v1"
    curve: String,      // "pasta"
    num_steps: u64,     // jumlah langkah folding
    proof_b64: String,  // base64(bincode(RecursiveSNARK))
}

// inputs.json = Vec<String> (top-level array)

// Field → hex lower (tanpa "0x")
fn f_to_hex<F: PrimeField>(x: F) -> String {
    let bytes = x.to_repr();
    hex::encode(bytes.as_ref())
}

// ---------- Circuit langkah: Fibonacci [a,b] -> [b, a+b] ----------
#[derive(Clone, Default)]
struct FibStep;

impl<F: PrimeField> StepCircuit<F> for FibStep {
    fn arity(&self) -> usize { 2 }

    fn synthesize<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        z: &[AllocatedNum<F>],
    ) -> Result<Vec<AllocatedNum<F>>, SynthesisError> {
        assert_eq!(z.len(), 2, "FibStep expects arity=2");
        let z0 = &z[0];
        let z1 = &z[1];

        // sum = z0 + z1 (witness)
        let sum = AllocatedNum::alloc(cs.namespace(|| "sum"), || {
            let a = z0.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let b = z1.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            Ok(a + b)
        })?;

        // ENFORCE PENJUMLAHAN: (z0 + z1) * 1 = sum
        cs.enforce(
            || "z0 + z1 = sum",
            |lc| lc + z0.get_variable() + z1.get_variable(), // A = z0 + z1
            |lc| lc + CS::one(),                              // B = 1
            |lc| lc + sum.get_variable(),                     // C = sum
        );

        // next state: [z1, sum]
        Ok(vec![z1.clone(), sum])
    }
}

// ---------- setup → prove(N) → verify → kemas JSON ----------
fn build_and_prove_supernova(num_steps: usize) -> Result<(VkJson, ProofJson, Vec<String>)> {
    type E1 = PallasEngine;
    type E2 = VestaEngine;
    type F1 = <E1 as Engine>::Scalar;

    let c = FibStep::default();

    // hint untuk masing-masing engine: E1 (Pallas) dulu, lalu E2 (Vesta)
    let ck1 = default_ck_hint::<E1>();
    let ck2 = default_ck_hint::<E2>();

    // urutan benar: ck1 (E1/Pallas), lalu ck2 (E2/Vesta)
    let pp = PublicParams::<E1, E2, FibStep>::setup(&c, ck1.as_ref(), ck2.as_ref())?;

    // z0 = [1, 1]
    let one = F1::ONE;
    let z0 = vec![one, one];

    // prove N langkah
    let mut rs: RecursiveSNARK<E1, E2, FibStep> = RecursiveSNARK::new(&pp, &c, &z0)?;
    for _ in 0..num_steps {
        rs.prove_step(&pp, &c)?;
    }

    // verify (butuh num_steps)
    let z_final = rs.verify(&pp, num_steps, &z0)?; // Vec<F1> berukuran 2

    // serialize → base64(bincode(..))
    let vk_b64 = B64.encode(bincode::serialize(&pp)?);
    let proof_b64 = B64.encode(bincode::serialize(&rs)?);

    // public inputs: pakai elemen kedua (F_n)
    let inputs = vec![f_to_hex(z_final[1])];

    let vk = VkJson {
        format: "supernova_v1".into(),
        curve: "pasta".into(),
        vk_b64,
    };
    let proof = ProofJson {
        format: "supernova_v1".into(),
        curve: "pasta".into(),
        num_steps: num_steps as u64,
        proof_b64,
    };

    Ok((vk, proof, inputs))
}

// ---------------- IO util ----------------
fn write_json<P: AsRef<Path>, T: Serialize>(path: P, val: &T) -> Result<()> {
    let f = File::create(path)?;
    let mut w = BufWriter::new(f);
    serde_json::to_writer_pretty(&mut w, val)?;
    Ok(())
}

// ---------------- main ----------------
fn main() -> Result<()> {
    let n_steps = 10; // ubah sesuai kebutuhan

    let (vk_json, proof_json, inputs_array) = build_and_prove_supernova(n_steps)?;

    // selalu tulis ke folder crate ini
    let outdir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    write_json(outdir.join("vk.json"), &vk_json)?;
    write_json(outdir.join("proof.json"), &proof_json)?;
    write_json(outdir.join("inputs.json"), &inputs_array)?;

    eprintln!("Generated vk.json / proof.json / inputs.json (SuperNova Pasta, real proof).");
    Ok(())
}
