fn risky_business() {
    let x: Option<i32> = None;
    x.unwrap();                 // Violation
    x.expect("Crash");          // Violation
}
