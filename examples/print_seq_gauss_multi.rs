use sobol_qmc::{MultiDimGaussianRender, Sobol, params::JoeKuoD6};
use statrs::distribution::Normal;

/// The dimensionality of the sequence to generate
const DIMS: usize = 10;

/// Number of points to generate
const N: usize = 128;

/// The type of sequence values
type ValType = f64;

/// Prints first N points from an example sequence
fn main() {
    println!(" [ Dimensions ] = {}", DIMS);
    println!(" [ Count      ] = {}", N);

    let params = JoeKuoD6::EXTENDED;
    let sobol = Sobol::<ValType, MultiDimGaussianRender>::new_with_resolution(
        DIMS,
        &params,
        None,
        MultiDimGaussianRender(
            (0..DIMS)
                .into_iter()
                .map(|mean| Normal::new(mean as f64, 1.0).unwrap())
                .collect(),
        ),
    )
    .unwrap();

    sobol
        .skip(1)
        .take(N)
        .map(|p| {
            p.iter()
                .map(|v| format!("{:<12}", v))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .for_each(|p| println!("{}", p));

    println!("> DONE.");
}
