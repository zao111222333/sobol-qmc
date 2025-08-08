pub mod params;
mod type_support;

use core::{
    fmt,
    ops::{AddAssign, BitXorAssign},
};
use num_traits::{Bounded, One, PrimInt, Unsigned, Zero};
use statrs::distribution::Normal;

/// A low-discrepancy Sobol sequence generator
#[derive(Clone)]
pub struct Sobol<T: SobolType, R: Render<T> = LinearRender> {
    pub dims: usize,
    pub resolution: usize,
    dir_vals: Vec<Vec<T::IT>>,
    previous: Option<Vec<T::IT>>,
    render: R,
    pub count: T::IT,
    pub max_len: T::IT,
}

#[derive(Debug, Clone, Copy)]
pub enum SobolError {
    MaxDim { dims: usize, max_dims: usize },
    RenderDim { dims: usize, render_dims: usize },
}
impl fmt::Display for SobolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MaxDim { dims, max_dims } => write!(
                f,
                "Sobol sequence supports a maximum of {max_dims} dimensions, but was configured for {dims}."
            ),
            Self::RenderDim { dims, render_dims } => write!(
                f,
                "Render supports a {render_dims} dimensions, but Sobol was configured for {dims}."
            ),
        }
    }
}

impl<T: SobolType> Sobol<T, LinearRender>
where
    LinearRender: Render<T>,
{
    /// Constructs a new sequence
    pub fn new<P, Param: SobolParams<P>>(dims: usize, params: &Param) -> Result<Self, SobolError>
    where
        T::IT: LossyFrom<P>,
    {
        Self::new_with_resolution::<P, Param>(dims, params, None, LinearRender)
    }
}
impl<T: SobolType, R: Render<T>> Sobol<T, R> {
    /// Constructs a new sequence of given resolution. Resolution is the number of bits used in the
    /// computation of the sequence and by default is the size of the underlying type. This
    /// constructor is useful for reducing the number of cycles necessary to generate each point when the
    /// length of the sequence is not expected to approach it's theoretically maximum (2^res).
    pub fn new_with_resolution<P, Param: SobolParams<P>>(
        dims: usize,
        params: &Param,
        resolution: Option<usize>,
        render: R,
    ) -> Result<Self, SobolError>
    where
        T::IT: LossyFrom<P>,
    {
        let res = resolution
            .filter(|res| *res <= T::MAX_RESOLUTION)
            .unwrap_or(T::MAX_RESOLUTION);
        let max_dims = params.max_dims();
        if dims > params.max_dims() {
            return Err(SobolError::MaxDim { dims, max_dims });
        }
        if let Some(render_dims) = render.support_dims() {
            if dims != render_dims {
                return Err(SobolError::RenderDim { dims, render_dims });
            }
        }

        let dir_values = Self::init_direction_vals::<P, Param>(dims, res, params);
        // Transpose dir values for better cache locality
        let dir_values = (0..dir_values[0].len())
            .map(|i| dir_values.iter().map(|inner| inner[i]).collect::<Vec<_>>())
            .collect();
        Ok(Sobol {
            dims,
            resolution: res,
            dir_vals: dir_values,
            count: T::IT::zero(),
            max_len: T::IT::max_value() >> (T::IT::BITS - res),
            previous: None,
            render,
        })
    }

    /// Initializes per-dimension direction values given sequence parameters
    pub fn init_direction_vals<P, Param: SobolParams<P>>(
        dims: usize,
        resolution: usize,
        params: &Param,
    ) -> Vec<Vec<T::IT>>
    where
        T::IT: LossyFrom<P>,
    {
        let bits = T::IT::BITS;

        (1..=dims)
            .map(|dim| match dim {
                1 => (1..=resolution)
                    .map(|i| T::IT::one() << (bits - i))
                    .collect(),
                _ => {
                    // Import the parameters needed to prepare this dimension's direction vector
                    let p = params.get_dim(dim);
                    let s = if resolution >= p.s() {
                        p.s()
                    } else {
                        resolution
                    };

                    // Shift initial directions
                    let mut dirs: Vec<T::IT> = vec![T::IT::zero(); resolution];
                    for i in 1..=s {
                        let m = T::IT::lossy_from(p.m(i - 1));
                        dirs[i - 1] = m << (bits - i);
                    }

                    // Compute remaining directions
                    for i in s + 1..=resolution {
                        dirs[i - 1] = dirs[i - s - 1] ^ (dirs[i - s - 1] >> s);

                        for k in 1..s {
                            let a = T::IT::lossy_from(p.coefficient(s - k - 1));
                            let dir = dirs[i - k - 1];
                            dirs[i - 1] ^= a * dir;
                        }
                    }

                    dirs
                }
            })
            .collect()
    }

    /// Returns zero-based index of the rightmost binary zero. Used for the Gray code optimization
    #[inline]
    pub fn rightmost_zero(n: T::IT) -> usize {
        (n ^ T::IT::max_value()).trailing_zeros() as usize
    }
}

impl<T: SobolType, R: Render<T>> Iterator for Sobol<T, R> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.max_len {
            let next = match &self.previous {
                None => vec![T::IT::zero(); self.dims],
                Some(previous) => {
                    let a = self.count - T::IT::one();
                    let c = Self::rightmost_zero(a);
                    self.dir_vals[c]
                        .iter()
                        .zip(previous)
                        .map(|(p, dir)| *p ^ *dir)
                        .collect::<Vec<T::IT>>()
                }
            };

            let next_render: Vec<T> = next
                .iter()
                .enumerate()
                .map(|(dim, val)| self.render.render(dim, *val))
                .collect();

            self.count += T::IT::one();
            self.previous = Some(next);

            Some(next_render)
        } else {
            None
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        for _ in 0..n {
            if self.count < self.max_len {
                let next = match &self.previous {
                    None => vec![T::IT::zero(); self.dims],
                    Some(previous) => {
                        let a = self.count - T::IT::one();
                        let c = Self::rightmost_zero(a);
                        self.dir_vals[c]
                            .iter()
                            .zip(previous)
                            .map(|(p, dir)| *p ^ *dir)
                            .collect::<Vec<T::IT>>()
                    }
                };

                self.count += T::IT::one();
                self.previous = Some(next);
            } else {
                break;
            }
        }
        self.next()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LinearRender;
#[derive(Debug, Clone, Copy)]
pub struct GaussianRender(pub Normal);
#[derive(Debug, Clone)]
pub struct MultiDimGaussianRender(pub Vec<Normal>);
pub trait Render<T: SobolType>: Clone {
    /// Converts internal values to those expected by the user. This usually
    /// involves casting and, for float values, scaling to the range [0,1).
    fn render(&self, dim: usize, val: <T as SobolType>::IT) -> T;

    fn support_dims(&self) -> Option<usize> {
        None
    }
}

/// The main type parameter for the `Sobol` iterator. This defines the concrete `InternalType`
/// to be used internally, as well as other properties necessary for sequence generation.
pub trait SobolType: Sized + fmt::Display {
    /// The unsigned integer type used internally to compute sequence values.
    type IT: InternalType;

    /// The maximum number of bits this type can support. By default, this is
    /// number of bits in the underlying `InternalType`, but it may be less
    /// in some cases (e.g. floats are limited by the size of their significand).
    const MAX_RESOLUTION: usize = Self::IT::BITS;
}

/// Sequences are computed internally using unsigned types with the following capabilities
pub trait InternalType: PrimInt + Unsigned + One + Zero + AddAssign + BitXorAssign + Copy {
    const BITS: usize;
}

/// Primitive polynomial parameters and initial direction values for all sequence dimensions
pub trait SobolParams<P> {
    type Dimension: ParamDimension<P>;
    /// Parameters for a given dimension
    fn get_dim(&self, dim: usize) -> &Self::Dimension;

    /// Maximum number of dimensions supported by this instance
    fn max_dims(&self) -> usize;
}

/// Primitive polynomial parameters and initial direction values for a single dimension
pub trait ParamDimension<P> {
    /// The one-based index of this dimension
    fn d(&self) -> u16;

    /// The degree of the primitive polynomial
    fn s(&self) -> usize;

    /// The binary coefficient for bit `i`, the zero-based index from the right
    fn coefficient(&self, i: usize) -> P;

    /// The initial direction value for bit `i`, the zero-based index from the right
    fn m(&self, i: usize) -> P;
}

/// A more permissive `From` trait - suitable for cases where lossy casting is
/// acceptable (i.e. truncation). This is used for casting parameter values to
/// internal values.
pub trait LossyFrom<T>: Sized {
    fn lossy_from(_: T) -> Self;
}
