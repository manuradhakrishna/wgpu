//! Convenience macros

#[cfg(doc)]
use crate::{VertexAttribute, VertexBufferLayout, VertexFormat};

/// Macro to produce an array of [`VertexAttribute`].
///
/// The input is a sequence of pairs of shader locations (expression of type [`u32`]) and
/// variant names of [`VertexFormat`].
///
/// The return value has type `[VertexAttribute; N]`, where `N` is the number of inputs.
///
/// Offsets are calculated automatically,
/// using the assumption that there is no padding or other data between attributes.
///
/// # Example
///
/// ```
/// // Suppose that this is our vertex format:
/// #[repr(C, packed)]
/// struct Vertex {
///     foo: [f32; 2],
///     bar: f32,
///     baz: [u16; 4],
/// }
///
/// // Then these attributes match it:
/// let attrs = wgpu::vertex_attr_array![
///     0 => Float32x2,
///     1 => Float32,
///     2 => Uint16x4,
/// ];
///
/// // Here's the full data structure the macro produced:
/// use wgpu::{VertexAttribute as A, VertexFormat as F};
/// assert_eq!(attrs, [
///     A { format: F::Float32x2, offset:  0, shader_location: 0, },
///     A { format: F::Float32,   offset:  8, shader_location: 1, },
///     A { format: F::Uint16x4,  offset: 12, shader_location: 2, },
/// ]);
/// ```
///
/// See [`VertexBufferLayout`] for an example building on this one.
#[macro_export]
macro_rules! vertex_attr_array {
    ($($location:expr => $format:ident),* $(,)?) => {
        $crate::_vertex_attr_array_helper!([] ; 0; $($location => $format ,)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _vertex_attr_array_helper {
    ([$($t:expr,)*] ; $off:expr ;) => { [$($t,)*] };
    ([$($t:expr,)*] ; $off:expr ; $location:expr => $format:ident, $($ll:expr => $ii:ident ,)*) => {
        $crate::_vertex_attr_array_helper!(
            [$($t,)*
            $crate::VertexAttribute {
                format: $crate::VertexFormat :: $format,
                offset: $off,
                shader_location: $location,
            },];
            $off + $crate::VertexFormat :: $format.size();
            $($ll => $ii ,)*
        )
    };
}

#[test]
fn test_vertex_attr_array() {
    let attrs = vertex_attr_array![0 => Float32x2, 3 => Uint16x4];
    // VertexAttribute does not support PartialEq, so we cannot test directly
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0].offset, 0);
    assert_eq!(attrs[0].shader_location, 0);
    assert_eq!(attrs[1].offset, size_of::<(f32, f32)>() as u64);
    assert_eq!(attrs[1].shader_location, 3);
}

/// Macro to load a SPIR-V module statically.
///
/// It ensures the word alignment as well as the magic number.
///
/// Return type: [`crate::ShaderModuleDescriptor`]
#[macro_export]
#[cfg(feature = "spirv")]
macro_rules! include_spirv {
    ($($token:tt)*) => {
        {
            //log::info!("including '{}'", $($token)*);
            $crate::ShaderModuleDescriptor {
                label: Some($($token)*),
                source: $crate::util::make_spirv(include_bytes!($($token)*)),
            }
        }
    };
}

/// Macro to load raw SPIR-V data statically, for use with [`Features::SPIRV_SHADER_PASSTHROUGH`].
///
/// It ensures the word alignment as well as the magic number.
///
/// [`Features::SPIRV_SHADER_PASSTHROUGH`]: crate::Features::SPIRV_SHADER_PASSTHROUGH
#[macro_export]
macro_rules! include_spirv_raw {
    ($($token:tt)*) => {
        {
            //log::info!("including '{}'", $($token)*);
            $crate::ShaderModuleDescriptorPassthrough::SpirV(
                $crate::ShaderModuleDescriptorSpirV {
                    label: $crate::__macro_helpers::Some($($token)*),
                    source: $crate::util::make_spirv_raw($crate::__macro_helpers::include_bytes!($($token)*)),
                }
            )
        }
    };
}

/// Load WGSL source code from a file at compile time.
///
/// The loaded path is relative to the path of the file containing the macro call, in the same way
/// as [`include_str!`] operates.
///
/// ```ignore
/// fn main() {
///     let module: ShaderModuleDescriptor = include_wgsl!("shader.wgsl");
/// }
/// ```
#[macro_export]
macro_rules! include_wgsl {
    ($($token:tt)*) => {
        {
            //log::info!("including '{}'", $($token)*);
            $crate::ShaderModuleDescriptor {
                label: $crate::__macro_helpers::Some($($token)*),
                source: $crate::ShaderSource::Wgsl($crate::__macro_helpers::Cow::Borrowed($crate::__macro_helpers::include_str!($($token)*))),
            }
        }
    };
}

#[doc(hidden)]
pub mod helpers {
    pub use alloc::borrow::Cow;
    pub use core::{include_bytes, include_str};
    pub use Some;
}
