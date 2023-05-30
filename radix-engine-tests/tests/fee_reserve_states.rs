use radix_engine::{system::system_modules::costing::transmute_u128_as_decimal, types::*};
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_fee_states() {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish("./tests/blueprints/fee_reserve_states");

    // Run test case
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 100.into())
            .call_function(
                package_address,
                "FeeReserveChecker",
                "check",
                manifest_args!(),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let output: (u32, Decimal, u32, Decimal) = receipt.expect_commit_success().output(1);
    assert_eq!(output.0, DEFAULT_COST_UNIT_LIMIT);
    assert_eq!(output.1, transmute_u128_as_decimal(DEFAULT_COST_UNIT_PRICE));
    assert_eq!(output.2, u32::from(DEFAULT_TIP_PERCENTAGE));
    // At the time checking fee balance, it should be still using system loan. This is because
    // loan is designed to be slightly more than what it takes to `lock_fee` from a component.
    // Therefore, the balance should be between `100` and `100 + loan_in_xrd`.
    assert!(
        output.3 > dec!(100)
            && output.3
                < dec!(100)
                    + Decimal::from(DEFAULT_SYSTEM_LOAN)
                        * (transmute_u128_as_decimal(DEFAULT_COST_UNIT_PRICE)
                            * (dec!(1) + Decimal::from(DEFAULT_TIP_PERCENTAGE) / dec!(100)))
    );
}
