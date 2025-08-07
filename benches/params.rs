#![feature(test)]
extern crate test;

use test::{Bencher, black_box};

use sobol_qmc::params::JoeKuoD6;

/// Benchmark loading of parameter data
#[bench]
fn bench_params(b: &mut Bencher) {
    b.iter(|| black_box(JoeKuoD6::STANDARD));
}
