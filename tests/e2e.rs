use std::process::Command;

fn run(fixture: &str) -> std::process::Output {
    let path = format!("tests/fixtures/{fixture}");
    Command::new(env!("CARGO_BIN_EXE_payment-engine"))
        .arg(&path)
        .output()
        .expect("failed to run binary")
}

fn sorted_body(stdout: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines[0], "client,available,held,total,locked");
    lines.remove(0);
    lines.sort();
    lines
}

#[test]
fn basic_deposits_and_withdrawals() {
    let output = run("basic.csv");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines = sorted_body(&stdout);

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "1,1.50,0.00,1.50,false");
    assert_eq!(lines[1], "2,2.00,0.00,2.00,false");
}

#[test]
fn frozen_account_rejects_transactions() {
    let output = run("frozen_account.csv");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines = sorted_body(&stdout);

    assert_eq!(lines.len(), 1);
    // Chargeback removes held funds and freezes; subsequent deposit and withdrawal are rejected
    assert_eq!(lines[0], "1,0.00,0.00,0.00,true");

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("frozen"),
        "expected frozen errors on stderr, got: {stderr}"
    );
}
