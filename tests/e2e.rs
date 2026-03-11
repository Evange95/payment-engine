use std::process::Command;

#[test]
fn processes_transactions_and_outputs_account_balances() {
    let input = "type,client,tx,amount\n\
                 deposit,1,1,1.0\n\
                 deposit,2,2,2.0\n\
                 deposit,1,3,2.0\n\
                 withdrawal,1,4,1.5\n\
                 withdrawal,2,5,3.0\n";

    let tmp = std::env::temp_dir().join("payment_engine_e2e.csv");
    std::fs::write(&tmp, input).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_payment-engine"))
        .arg(&tmp)
        .output()
        .expect("failed to run binary");

    assert!(output.status.success(), "binary failed: {:?}", output.stderr);

    let stdout = String::from_utf8(output.stdout).unwrap();
    let mut lines: Vec<&str> = stdout.trim().lines().collect();

    // Header should be first
    assert_eq!(lines[0], "client,available,held,total,locked");

    // Sort remaining lines by client id for deterministic comparison
    let header = lines.remove(0);
    lines.sort();

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "1,1.5000,0.0000,1.5000,false");
    assert_eq!(lines[1], "2,2.0000,0.0000,2.0000,false");

    let _ = header;
    std::fs::remove_file(&tmp).ok();
}
