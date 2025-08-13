use anyhow::Result;

use verifier_supernova::{
    decode_b64, f_to_hex, read_json,
    FibStep, ProofFile, VkFile,
};

// Nova/SuperNova types
use nova_snark::{
    nova::{PublicParams, RecursiveSNARK},
    provider::{PallasEngine, VestaEngine},
    traits::Engine,
};
use ff::Field; // for F::ONE

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} proof.json vk.json inputs.json", args[0]);
        std::process::exit(2);
    }

    // 1) Baca file JSON
    let pf: ProofFile = read_json(&args[1])?;
    let vkf: VkFile   = read_json(&args[2])?;
    let inputs: Vec<String> = read_json(&args[3])?;

    // 2) Validasi header
    if pf.format != "supernova_v1" || vkf.format != "supernova_v1" {
        anyhow::bail!("unexpected format: proof={}, vk={}", pf.format, vkf.format);
    }
    if pf.curve != "pasta" || vkf.curve != "pasta" {
        anyhow::bail!("unexpected curve: proof={}, vk={}", pf.curve, vkf.curve);
    }

    // 3) Decode b64 → bincode → objek Nova
    type E1 = PallasEngine;
    type E2 = VestaEngine;
    type F1 = <E1 as Engine>::Scalar;

    // PublicParams<E1,E2,FibStep>
    let pp: PublicParams<E1, E2, FibStep> = decode_b64(&vkf.vk_b64)
        .map_err(|e| anyhow::anyhow!("decode vk_b64: {e}"))?;
    // RecursiveSNARK<E1,E2,FibStep>
    let rs: RecursiveSNARK<E1, E2, FibStep> = decode_b64(&pf.proof_b64)
        .map_err(|e| anyhow::anyhow!("decode proof_b64: {e}"))?;

    // 4) Verifikasi
    let z0 = vec![F1::ONE, F1::ONE]; // harus sama dengan generator
    let z_final = rs.verify(&pp, pf.num_steps as usize, &z0)
        .map_err(|e| anyhow::anyhow!("verify failed: {e}"))?;

    // 5) Cek optional: cocokan public input yang dikirim (hex) dengan z_final[1]
    let mut ok = true;
    if let Some(want_hex) = inputs.first() {
        let got_hex = f_to_hex(z_final[1]);
        ok = &got_hex == want_hex;
    }

    println!("Verification result: {ok}");
    Ok(())
}
