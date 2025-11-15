#![cfg(target_os = "linux")]

use std::fs;
use std::process::Command;

use tempfile::tempdir;

fn run_case(input: &str) -> i32 {
    let tmp_dir = tempdir().expect("failed to create temp dir");
    let dir_path = tmp_dir.path();

    let src_path = dir_path.join("tmp.c");
    fs::write(&src_path, input).expect("failed to write tmp.c");

    let chimocc = env!("CARGO_BIN_EXE_chimocc");
    let status = Command::new(chimocc)
        .current_dir(dir_path)
        .arg("tmp.c")
        .status()
        .expect("failed to run chimocc");
    assert!(status.success(), "chimocc failed for input {}", input);

    let asm_path = dir_path.join("tmp.s");
    assert!(
        asm_path.exists(),
        "expected tmp.s to be generated for input {}",
        input
    );

    let status = Command::new("cc")
        .current_dir(dir_path)
        .args(["-o", "tmp", "tmp.s"])
        .status()
        .expect("failed to run cc");
    assert!(status.success(), "cc failed for input {}", input);

    let exec_path = dir_path.join("tmp");
    let status = Command::new(&exec_path)
        .status()
        .unwrap_or_else(|e| panic!("failed to run compiled binary: {e}"));

    status
        .code()
        .unwrap_or_else(|| panic!("program terminated by signal for input {}", input))
}

fn assert_case(expected: i32, input: &str) {
    let actual = run_case(input);
    assert_eq!(actual, expected, "input: {}", input);
}

#[test]
fn test_literals_and_arithmetic() {
    assert_case(1, "1;");
    assert_case(0, "0;");
    assert_case(255, "255;");
    assert_case(1, "1 + 0;");
    assert_case(2, "1 + 1;");
    assert_case(97, "1 + 100 - 4;");
    assert_case(4, "1 * 2 + 8 / 4;");
    assert_case(6, "1 * 2 + 2 *8 / 4;");
    assert_case(97, "1 * 2 - 2 *8 / 4 + 99;");
    assert_case(99, "1 * (2 - 2) *8 / 4 + 99;");
    assert_case(5, "(1 - 2) * (0 - 8) - 3*1;");
    assert_case(10, "-10 + 20;");
    assert_case(10, "- -10;");
    assert_case(10, "- - +10;");
}

#[test]
fn test_comparisons() {
    assert_case(1, "1 == 1;");
    assert_case(0, "1 != 1;");
    assert_case(0, "1 > 2;");
    assert_case(1, "2 > 1;");
    assert_case(1, "1 < 2;");
    assert_case(0, "1 < 1;");
    assert_case(1, "1 <= 2;");
    assert_case(1, "1 <= 1;");
    assert_case(0, "1 >= 2;");
    assert_case(1, "1 >= 1;");
    assert_case(0, "1 > 1;");
}

#[test]
fn test_comparison_chains() {
    assert_case(1, "1 == 1;");
    assert_case(0, "1 != 1;");
    assert_case(0, "(1 == 1) < (1 != 1);");
    assert_case(1, "1 == 1 < 1 != 1;");
    assert_case(1, "2 >= 2;");
    assert_case(
        100,
        "((32 + -980) <= (-  5*4 * (-2) + 9 -997 * (-2) / (-2))) + 99;",
    );
}

#[test]
fn test_assignments() {
    assert_case(1, "a = 1; a;");
    assert_case(2, "a = 1; a = 2; a;");
    assert_case(3, "a = 1; a = a + 2; a;");
    assert_case(
        55,
        "a = 1; b = 2; c = 3; z = 4; y = 5; x = 6; t = 10; u = 9; v = 8; w = 7; a +  b + c + z + y + x + t + u + v + w;",
    );
    assert_case(3, "abc = 22; cde=7; abc / cde;");
}

#[test]
fn test_return() {
    assert_case(1, "return 1;");
    assert_case(3, "return 1 + 2;");
    assert_case(1, "abc = 22; cde=7; return abc > cde;");
    assert_case(4, "return 4; return 5;");
}

#[test]
fn test_if() {
    assert_case(3, "a = 1; if (44 > 32) a = 3; if(44 < 32) a = 5; return a;");
    assert_case(100, "a = 5; if (55 != 43) a = 100; else a = 50; return a;");
}

#[test]
fn test_while() {
    assert_case(10, "a = 0; while (a < 10) a = a + 1; return a;");
    assert_case(0, "a = 10; while (a > 0) a = a - 1; return a;");
}

#[test]
fn test_for() {
    assert_case(
        10,
        "a = 0; for (i = 0; i < 10; i = i + 1) a = a + 1; return a;",
    );
    assert_case(
        0,
        "a = 10; for (i = 0; i < 10; i = i + 1) a = a - 1; return a;",
    );
    assert_case(
        55,
        "sum = 0; for (i = 1; i <= 10; i = i + 1) sum = sum + i; return sum;",
    );
}
