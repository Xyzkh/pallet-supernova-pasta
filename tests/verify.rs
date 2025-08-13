use predicates::prelude::*;
use std::{fs, path::Path};

#[test]
fn verify_frozen_fixtures_pass() {
    let dir = "fixtures/pasta-fib-n10";
    if !Path::new(dir).exists() {
        eprintln!("fixtures missing; skip");
        return;
    }

    let mut cmd = assert_cmd::Command::cargo_bin("verifier-supernova").unwrap();
    cmd.args([
        &format!("{dir}/proof.json"),
        &format!("{dir}/vk.json"),
        &format!("{dir}/inputs.json"),
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Verification result: true"));
}

#[test]
fn verify_corrupted_proof_fails() {
    let dir = "fixtures/pasta-fib-n10";
    if !Path::new(dir).exists() { return; }

    // korupkan 1 byte di file proof
    let src = format!("{dir}/proof.json");
    let tmp = format!("{dir}/_proof_corrupt.json");
    let mut data = fs::read(&src).expect("read proof");
    if let Some(b) = data.get_mut(16) { *b ^= 0xFF; }
    fs::write(&tmp, &data).expect("write tmp");

    let mut cmd = assert_cmd::Command::cargo_bin("verifier-supernova").unwrap();
    cmd.args([&tmp, &format!("{dir}/vk.json"), &format!("{dir}/inputs.json")]);
    cmd.assert().failure(); // harus exit code non-zero

    let _ = fs::remove_file(&tmp);
}
