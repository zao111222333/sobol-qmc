use crate::{ParamDimension, SobolParams};

#[derive(Debug, Clone, Copy)]
pub struct JoeKuoD6 {
    pub dim_params: &'static [JoeKuoD6Dim],
    pub max_dims: usize,
}

include!(concat!(env!("OUT_DIR"), "/gen_joe_kuo_d6.rs"));
impl JoeKuoD6 {
    /// Load parameter values supporting up to **1000** dimensions
    pub const STANDARD: Self = STANDARD;

    /// Load parameter values supporting up to **100** dimensions
    pub const MINIMAL: Self = MINIMAL;

    /// Load parameter values supporting up to **21,201** dimensions
    pub const EXTENDED: Self = EXTENDED;
}

impl SobolParams<u32> for JoeKuoD6 {
    type Dimension = JoeKuoD6Dim;
    #[inline]
    fn get_dim(&self, dim: usize) -> &JoeKuoD6Dim {
        &self.dim_params[dim - 2]
    }

    #[inline]
    fn max_dims(&self) -> usize {
        self.max_dims
    }
}

/// Parameters for a single dimension
#[derive(Debug, Clone, Copy)]
pub struct JoeKuoD6Dim {
    pub d: u16,
    pub a: u32,
    pub m: &'static [u32],
}

impl ParamDimension<u32> for JoeKuoD6Dim {
    #[inline]
    fn d(&self) -> u16 {
        self.d
    }

    #[inline]
    fn s(&self) -> usize {
        self.m.len()
    }

    #[inline]
    fn coefficient(&self, i: usize) -> u32 {
        (self.a >> i) & 1
    }

    #[inline]
    fn m(&self, i: usize) -> u32 {
        self.m[i]
    }
}
