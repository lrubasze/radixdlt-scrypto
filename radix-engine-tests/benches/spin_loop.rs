use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::types::*;
use scrypto_unit::TestRunner;
use transaction::builder::ManifestBuilder;

fn bench_spin_loop(c: &mut Criterion) {
    // Set up environment.
    let mut test_runner = TestRunner::builder().without_trace().build();

    let package_address = test_runner.compile_and_publish("./tests/blueprints/fee");
    let component_address = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(FAUCET_COMPONENT, 10u32.into())
                .call_method(FAUCET_COMPONENT, "free", args!())
                .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                    builder.call_function(package_address, "Fee", "new", args!(bucket_id));
                    builder
                })
                .build(),
            vec![],
        )
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Create a transfer manifest
    let manifest = ManifestBuilder::new()
        // First, lock the fee so that the loan will be repaid
        .call_method(FAUCET_COMPONENT, "lock_fee", args!(Decimal::from(10)))
        // Now spin-loop to wait for the fee loan to burn through
        .call_method(component_address, "spin_loop", args!())
        .build();

    // Loop
    c.bench_function("Spin Loop", |b| {
        b.iter(|| {
            let receipt = test_runner.execute_manifest(manifest.clone(), vec![]);
            receipt.expect_commit_failure();
        })
    });
}

criterion_group!(spin_loop, bench_spin_loop);
criterion_main!(spin_loop);