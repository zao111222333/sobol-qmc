use libflate::gzip::Decoder;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor, Write},
};

fn gen_joe_kuo_d6() {
    pub struct JoeKuoD6 {
        pub dim_params: Vec<JoeKuoD6Dim>,
        pub max_dims: usize,
    }
    /// Parameters for a single dimension
    pub struct JoeKuoD6Dim {
        pub d: u16,
        pub a: u32,
        pub m: Vec<u32>,
    }

    impl JoeKuoD6Dim {
        /// Parses the dimensional parameters from string according to the format provided by Joe/Kuo
        pub fn parse(s: &str) -> Self {
            let mut tokens = s.split_whitespace();
            let d = tokens.next().unwrap().parse::<u16>().unwrap();
            tokens.next();
            let a = tokens.next().unwrap().parse::<u32>().unwrap();
            let m = tokens.map(|t| t.parse::<u32>().unwrap()).collect();
            JoeKuoD6Dim { d, a, m }
        }
    }

    impl ToTokens for JoeKuoD6 {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let dim_params: Vec<_> = self
                .dim_params
                .iter()
                .map(|JoeKuoD6Dim { d, a, m }| {
                    quote! {
                        JoeKuoD6Dim {
                            d: #d,
                            a: #a,
                            m: &[#(#m),*],
                        }
                    }
                })
                .collect();
            let max_dims = self.max_dims;
            tokens.extend(quote! {
                JoeKuoD6 {
                    dim_params: &[#(#dim_params),*],
                    max_dims: #max_dims,
                }
            });
        }
    }

    impl JoeKuoD6 {
        /// Instantiates parameter struct from gz sequence of bytes
        fn load_gz_bytes(bytes: &[u8]) -> Self {
            let mut byte_cursor = Cursor::new(bytes);
            let gz_decoder = Decoder::new(&mut byte_cursor).unwrap();
            let dim_params: Vec<JoeKuoD6Dim> = BufReader::new(gz_decoder)
                .lines()
                .skip(1)
                .map(|l| JoeKuoD6Dim::parse(&l.unwrap()))
                .collect();
            let max_dims = dim_params.len() + 1;
            JoeKuoD6 {
                dim_params,
                max_dims,
            }
        }
    }
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("gen_joe_kuo_d6.rs");
    let mut file = File::create(&dest_path).expect("Could not create file");
    let p100 = JoeKuoD6::load_gz_bytes(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/params/data/new-joe-kuo-6.100.gz"
    )));
    let p1000 = JoeKuoD6::load_gz_bytes(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/params/data/new-joe-kuo-6.1000.gz"
    )));
    let p21201 = JoeKuoD6::load_gz_bytes(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/params/data/new-joe-kuo-6.21201.gz"
    )));
    file.write_all(
        quote! {
            const MINIMAL: JoeKuoD6 = #p100;
            const STANDARD: JoeKuoD6 = #p1000;
            const EXTENDED: JoeKuoD6 = #p21201;
        }
        .to_string()
        .as_bytes(),
    )
    .expect("Could not write file");
}

fn gen_ref_seq() {
    fn load_ref_seq(filename: &str) -> Vec<Vec<f32>> {
        if let Ok(mut file) = File::open(filename) {
            let mut decoder = Decoder::new(&mut file).unwrap();
            BufReader::new(&mut decoder)
                .lines()
                .map(|res| {
                    res.unwrap()
                        .split_whitespace()
                        .map(|v| v.parse::<f32>().unwrap())
                        .collect()
                })
                .collect()
        } else {
            // for release crate package, no ref_seq file
            vec![vec![]]
        }
    }
    fn ref_seq2token(ref_seq: Vec<Vec<f32>>) -> TokenStream {
        let tokens: Vec<_> = ref_seq
            .into_iter()
            .map(|v| {
                let vs: Vec<_> = v.into_iter().map(|f| quote! {#f}).collect();
                quote! {&[#(#vs),*]}
            })
            .collect();
        quote! {&[#(#tokens),*]}
    }
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("gen_ref_seq.rs");
    let mut file = File::create(&dest_path).expect("Could not create file");
    let lo = ref_seq2token(load_ref_seq(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/ref_seq_lo.tsv.gz"
    )));
    let hi = ref_seq2token(load_ref_seq(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/ref_seq_hi.tsv.gz"
    )));
    file.write_all(
        quote! {
            const REF_SEQ_LO: &[&[f32]] = #lo;
            const REF_SEQ_HI: &[&[f32]] = #hi;
        }
        .to_string()
        .as_bytes(),
    )
    .expect("Could not write file");
}

fn main() {
    gen_joe_kuo_d6();
    gen_ref_seq();
}
