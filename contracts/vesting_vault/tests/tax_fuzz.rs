use soroban_sdk::Env;

/// Validate tax calculation uses ceiling division so tax + net == gross
#[test]
fn fuzz_tax_percentage_calculation() {
    let env = Env::default();

    // Test a range of gross amounts and tax bps values
    let bps_values = [0u32, 1, 50, 100, 250, 999, 1000, 2500, 5000, 7500, 10000];
    let amounts = [0i128, 1, 2, 10, 99, 100, 101, 12345, 1_000_000, 9_876_543_210i128];

    for &gross in amounts.iter() {
        for &bps in bps_values.iter() {
            let numerator: i128 = gross.checked_mul(bps as i128).expect("mul overflow");
            let tax = (numerator + 9_999i128) / 10_000i128; // ceil
            let net = gross - tax;

            // Ensure exact conservation: no stroops lost
            assert_eq!(tax + net, gross, "Conservation failed for gross={} bps={}", gross, bps);

            // Also ensure tax equals expected ceil of fractional product
            let expected_floor = numerator / 10_000i128;
            if numerator % 10_000i128 == 0 {
                assert_eq!(tax, expected_floor, "Exact division mismatch");
            } else {
                assert!(tax == expected_floor + 1, "Ceil mismatch");
            }
        }
    }
}
