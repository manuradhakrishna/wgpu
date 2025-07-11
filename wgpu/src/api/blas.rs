use alloc::{boxed::Box, vec::Vec};

use wgt::{WasmNotSend, WasmNotSendSync};

use crate::dispatch;
use crate::{Buffer, Label};

/// Descriptor for the size defining attributes of a triangle geometry, for a bottom level acceleration structure.
pub type BlasTriangleGeometrySizeDescriptor = wgt::BlasTriangleGeometrySizeDescriptor;
static_assertions::assert_impl_all!(BlasTriangleGeometrySizeDescriptor: Send, Sync);

/// Descriptor for the size defining attributes, for a bottom level acceleration structure.
pub type BlasGeometrySizeDescriptors = wgt::BlasGeometrySizeDescriptors;
static_assertions::assert_impl_all!(BlasGeometrySizeDescriptors: Send, Sync);

/// Flags for an acceleration structure.
pub type AccelerationStructureFlags = wgt::AccelerationStructureFlags;
static_assertions::assert_impl_all!(AccelerationStructureFlags: Send, Sync);

/// Flags for a geometry inside a bottom level acceleration structure.
pub type AccelerationStructureGeometryFlags = wgt::AccelerationStructureGeometryFlags;
static_assertions::assert_impl_all!(AccelerationStructureGeometryFlags: Send, Sync);

/// Update mode for acceleration structure builds.
pub type AccelerationStructureUpdateMode = wgt::AccelerationStructureUpdateMode;
static_assertions::assert_impl_all!(AccelerationStructureUpdateMode: Send, Sync);

/// Descriptor to create bottom level acceleration structures.
pub type CreateBlasDescriptor<'a> = wgt::CreateBlasDescriptor<Label<'a>>;
static_assertions::assert_impl_all!(CreateBlasDescriptor<'_>: Send, Sync);

/// Safe instance for a [Tlas].
///
/// A TlasInstance may be made invalid, if a TlasInstance is invalid, any attempt to build a [Tlas] containing an
/// invalid TlasInstance will generate a validation error
///
/// Each one contains:
/// - A reference to a BLAS, this ***must*** be interacted with using [TlasInstance::new] or [TlasInstance::set_blas], a
///   TlasInstance that references a BLAS keeps that BLAS from being dropped
/// - A user accessible transformation matrix
/// - A user accessible mask
/// - A user accessible custom index
///
/// [Tlas]: crate::Tlas
#[derive(Debug, Clone)]
pub struct TlasInstance {
    pub(crate) blas: dispatch::DispatchBlas,
    /// Affine transform matrix 3x4 (rows x columns, row major order).
    pub transform: [f32; 12],
    /// Custom index for the instance used inside the shader.
    ///
    /// This must only use the lower 24 bits, if any bits are outside that range (byte 4 does not equal 0) the TlasInstance becomes
    /// invalid and generates a validation error when built
    pub custom_data: u32,
    /// Mask for the instance used inside the shader to filter instances.
    /// Reports hit only if `(shader_cull_mask & tlas_instance.mask) != 0u`.
    pub mask: u8,
}

impl TlasInstance {
    /// Construct TlasInstance.
    /// - blas: Reference to the bottom level acceleration structure
    /// - transform: Transform buffer offset in bytes (optional, required if transform buffer is present)
    /// - custom_data: Custom index for the instance used inside the shader (max 24 bits)
    /// - mask: Mask for the instance used inside the shader to filter instances
    ///
    /// Note: while one of these contains a reference to a BLAS that BLAS will not be dropped,
    /// but it can still be destroyed. Destroying a BLAS that is referenced by one or more
    /// TlasInstance(s) will immediately make them invalid. If one or more of those invalid
    /// TlasInstances is inside a TlasPackage that is attempted to be built, the build will
    /// generate a validation error.
    pub fn new(blas: &Blas, transform: [f32; 12], custom_data: u32, mask: u8) -> Self {
        Self {
            blas: blas.inner.clone(),
            transform,
            custom_data,
            mask,
        }
    }

    /// Set the bottom level acceleration structure.
    ///
    /// See the note on [TlasInstance] about the
    /// guarantees of keeping a BLAS alive.
    pub fn set_blas(&mut self, blas: &Blas) {
        self.blas = blas.inner.clone();
    }
}

#[derive(Debug)]
/// Definition for a triangle geometry for a Bottom Level Acceleration Structure (BLAS).
///
/// The size must match the rest of the structures fields, otherwise the build will fail.
/// (e.g. if index count is present in the size, the index buffer must be present as well.)
pub struct BlasTriangleGeometry<'a> {
    /// Sub descriptor for the size defining attributes of a triangle geometry.
    pub size: &'a BlasTriangleGeometrySizeDescriptor,
    /// Vertex buffer.
    pub vertex_buffer: &'a Buffer,
    /// Offset into the vertex buffer as a factor of the vertex stride.
    pub first_vertex: u32,
    /// Vertex stride, must be greater than [`wgpu_types::VertexFormat::min_acceleration_structure_vertex_stride`]
    /// of the format and must be a multiple of [`wgpu_types::VertexFormat::acceleration_structure_stride_alignment`].
    pub vertex_stride: wgt::BufferAddress,
    /// Index buffer (optional).
    pub index_buffer: Option<&'a Buffer>,
    /// Number of indexes to skip in the index buffer (optional, required if index buffer is present).
    pub first_index: Option<u32>,
    /// Transform buffer containing 3x4 (rows x columns, row major) affine transform matrices `[f32; 12]` (optional).
    pub transform_buffer: Option<&'a Buffer>,
    /// Transform buffer offset in bytes (optional, required if transform buffer is present).
    pub transform_buffer_offset: Option<wgt::BufferAddress>,
}
static_assertions::assert_impl_all!(BlasTriangleGeometry<'_>: WasmNotSendSync);

/// Contains the sets of geometry that go into a [Blas].
pub enum BlasGeometries<'a> {
    /// Triangle geometry variant.
    TriangleGeometries(Vec<BlasTriangleGeometry<'a>>),
}
static_assertions::assert_impl_all!(BlasGeometries<'_>: WasmNotSendSync);

/// Builds the given sets of geometry into the given [Blas].
pub struct BlasBuildEntry<'a> {
    /// Reference to the acceleration structure.
    pub blas: &'a Blas,
    /// Geometries.
    pub geometry: BlasGeometries<'a>,
}
static_assertions::assert_impl_all!(BlasBuildEntry<'_>: WasmNotSendSync);

#[derive(Debug, Clone)]
/// Bottom Level Acceleration Structure (BLAS).
///
/// A BLAS is a device-specific raytracing acceleration structure that contains geometry data.
///
/// These BLASes are combined with transform in a [TlasInstance] to create a [Tlas].
///
/// [Tlas]: crate::Tlas
pub struct Blas {
    pub(crate) handle: Option<u64>,
    pub(crate) inner: dispatch::DispatchBlas,
}
static_assertions::assert_impl_all!(Blas: WasmNotSendSync);

crate::cmp::impl_eq_ord_hash_proxy!(Blas => .inner);

impl Blas {
    /// Raw handle to the acceleration structure, used inside raw instance buffers.
    pub fn handle(&self) -> Option<u64> {
        self.handle
    }

    /// Returns the inner hal Acceleration Structure using a callback. The hal acceleration structure
    /// will be `None` if the backend type argument does not match with this wgpu Blas
    ///
    /// This method will start the wgpu_core level command recording.
    ///
    /// # Safety
    ///
    /// - The raw handle obtained from the hal Acceleration Structure must not be manually destroyed
    #[cfg(wgpu_core)]
    pub unsafe fn as_hal<
        A: wgc::hal_api::HalApi,
        F: FnOnce(Option<&A::AccelerationStructure>) -> R,
        R,
    >(
        &mut self,
        hal_blas_callback: F,
    ) -> R {
        if let Some(blas) = self.inner.as_core_opt() {
            unsafe { blas.context.blas_as_hal::<A, F, R>(blas, hal_blas_callback) }
        } else {
            hal_blas_callback(None)
        }
    }

    #[cfg(custom)]
    /// Returns custom implementation of Blas (if custom backend and is internally T)
    pub fn as_custom<T: crate::custom::BlasInterface>(&self) -> Option<&T> {
        self.inner.as_custom()
    }
}

/// Context version of [BlasTriangleGeometry].
pub struct ContextBlasTriangleGeometry<'a> {
    #[expect(dead_code)]
    pub(crate) size: &'a BlasTriangleGeometrySizeDescriptor,
    #[expect(dead_code)]
    pub(crate) vertex_buffer: &'a dispatch::DispatchBuffer,
    #[expect(dead_code)]
    pub(crate) index_buffer: Option<&'a dispatch::DispatchBuffer>,
    #[expect(dead_code)]
    pub(crate) transform_buffer: Option<&'a dispatch::DispatchBuffer>,
    #[expect(dead_code)]
    pub(crate) first_vertex: u32,
    #[expect(dead_code)]
    pub(crate) vertex_stride: wgt::BufferAddress,
    #[expect(dead_code)]
    pub(crate) index_buffer_offset: Option<wgt::BufferAddress>,
    #[expect(dead_code)]
    pub(crate) transform_buffer_offset: Option<wgt::BufferAddress>,
}

/// Context version of [BlasGeometries].
pub enum ContextBlasGeometries<'a> {
    /// Triangle geometries.
    TriangleGeometries(Box<dyn Iterator<Item = ContextBlasTriangleGeometry<'a>> + 'a>),
}

/// Context version see [BlasBuildEntry].
pub struct ContextBlasBuildEntry<'a> {
    #[expect(dead_code)]
    pub(crate) blas: &'a dispatch::DispatchBlas,
    #[expect(dead_code)]
    pub(crate) geometries: ContextBlasGeometries<'a>,
}

/// Error occurred when trying to asynchronously prepare a blas for compaction.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BlasAsyncError;
static_assertions::assert_impl_all!(BlasAsyncError: Send, Sync);

impl core::fmt::Display for BlasAsyncError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Error occurred when trying to asynchronously prepare a blas for compaction"
        )
    }
}

impl core::error::Error for BlasAsyncError {}

impl Blas {
    /// Asynchronously prepares this BLAS for compaction. The callback is called once all builds
    /// using this BLAS are finished and the BLAS is compactable. This can be checked using
    /// [`Blas::ready_for_compaction`]. Rebuilding this BLAS will reset its compacted state, and it
    /// will need to be prepared again.
    ///
    /// ### Interaction with other functions
    /// On native, `queue.submit(..)` and polling devices (that is calling `instance.poll_all` or
    /// `device.poll`) with [`PollType::Poll`] may call the callback. On native, polling devices with
    /// [`PollType::Wait`] (or [`PollType::WaitForSubmissionIndex`] with a submission index greater
    /// than the last submit the BLAS was used in) will guarantee callback is called.
    ///
    /// [`PollType::Poll`]: wgpu_types::PollType::Poll
    /// [`PollType::Wait`]: wgpu_types::PollType::Wait
    /// [`PollType::WaitForSubmissionIndex`]: wgpu_types::PollType::WaitForSubmissionIndex
    pub fn prepare_compaction_async(
        &self,
        callback: impl FnOnce(Result<(), BlasAsyncError>) + WasmNotSend + 'static,
    ) {
        self.inner.prepare_compact_async(Box::new(callback));
    }

    /// Checks whether this BLAS is ready for compaction. The returned value is `true` if
    /// [`Blas::prepare_compaction_async`]'s callback was called with a non-error value, otherwise
    /// this is `false`.
    pub fn ready_for_compaction(&self) -> bool {
        self.inner.ready_for_compaction()
    }
}
