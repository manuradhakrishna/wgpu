use alloc::boxed::Box;
use core::ops::{Deref, DerefMut};

use crate::*;

/// Handle to a command queue on a device.
///
/// A `Queue` executes recorded [`CommandBuffer`] objects and provides convenience methods
/// for writing to [buffers](Queue::write_buffer) and [textures](Queue::write_texture).
/// It can be created along with a [`Device`] by calling [`Adapter::request_device`].
///
/// Corresponds to [WebGPU `GPUQueue`](https://gpuweb.github.io/gpuweb/#gpu-queue).
#[derive(Debug, Clone)]
pub struct Queue {
    pub(crate) inner: dispatch::DispatchQueue,
}
#[cfg(send_sync)]
static_assertions::assert_impl_all!(Queue: Send, Sync);

crate::cmp::impl_eq_ord_hash_proxy!(Queue => .inner);

impl Queue {
    #[cfg(custom)]
    /// Returns custom implementation of Queue (if custom backend and is internally T)
    pub fn as_custom<T: custom::QueueInterface>(&self) -> Option<&T> {
        self.inner.as_custom()
    }

    #[cfg(custom)]
    /// Creates Queue from custom implementation
    pub fn from_custom<T: custom::QueueInterface>(queue: T) -> Self {
        Self {
            inner: dispatch::DispatchQueue::custom(queue),
        }
    }
}

/// Identifier for a particular call to [`Queue::submit`]. Can be used
/// as part of an argument to [`Device::poll`] to block for a particular
/// submission to finish.
///
/// This type is unique to the Rust API of `wgpu`.
/// There is no analogue in the WebGPU specification.
#[derive(Debug, Clone)]
pub struct SubmissionIndex {
    pub(crate) index: u64,
}
#[cfg(send_sync)]
static_assertions::assert_impl_all!(SubmissionIndex: Send, Sync);

/// Passed to [`Device::poll`] to control how and if it should block.
pub type PollType = wgt::PollType<SubmissionIndex>;
#[cfg(send_sync)]
static_assertions::assert_impl_all!(PollType: Send, Sync);

/// A write-only view into a staging buffer.
///
/// Reading into this buffer won't yield the contents of the buffer from the
/// GPU and is likely to be slow. Because of this, although [`AsMut`] is
/// implemented for this type, [`AsRef`] is not.
pub struct QueueWriteBufferView<'a> {
    queue: &'a Queue,
    buffer: &'a Buffer,
    offset: BufferAddress,
    inner: dispatch::DispatchQueueWriteBuffer,
}
#[cfg(send_sync)]
static_assertions::assert_impl_all!(QueueWriteBufferView<'_>: Send, Sync);

impl QueueWriteBufferView<'_> {
    #[cfg(custom)]
    /// Returns custom implementation of QueueWriteBufferView (if custom backend and is internally T)
    pub fn as_custom<T: custom::QueueWriteBufferInterface>(&self) -> Option<&T> {
        self.inner.as_custom()
    }
}

impl Deref for QueueWriteBufferView<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        log::warn!("Reading from a QueueWriteBufferView won't yield the contents of the buffer and may be slow.");
        self.inner.slice()
    }
}

impl DerefMut for QueueWriteBufferView<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.slice_mut()
    }
}

impl AsMut<[u8]> for QueueWriteBufferView<'_> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.inner.slice_mut()
    }
}

impl Drop for QueueWriteBufferView<'_> {
    fn drop(&mut self) {
        self.queue
            .inner
            .write_staging_buffer(&self.buffer.inner, self.offset, &self.inner);
    }
}

impl Queue {
    /// Copies the bytes of `data` into `buffer` starting at `offset`.
    ///
    /// The data must be written fully in-bounds, that is, `offset + data.len() <= buffer.len()`.
    ///
    /// # Performance considerations
    ///
    /// * Calls to `write_buffer()` do *not* submit the transfer to the GPU
    ///   immediately. They begin GPU execution only on the next call to
    ///   [`Queue::submit()`], just before the explicitly submitted commands.
    ///   To get a set of scheduled transfers started immediately,
    ///   it's fine to call `submit` with no command buffers at all:
    ///
    ///   ```no_run
    ///   # let queue: wgpu::Queue = todo!();
    ///   # let buffer: wgpu::Buffer = todo!();
    ///   # let data = [0u8];
    ///   queue.write_buffer(&buffer, 0, &data);
    ///   queue.submit([]);
    ///   ```
    ///
    ///   However, `data` will be immediately copied into staging memory, so the
    ///   caller may discard it any time after this call completes.
    ///
    /// * Consider using [`Queue::write_buffer_with()`] instead.
    ///   That method allows you to prepare your data directly within the staging
    ///   memory, rather than first placing it in a separate `[u8]` to be copied.
    ///   That is, `queue.write_buffer(b, offset, data)` is approximately equivalent
    ///   to `queue.write_buffer_with(b, offset, data.len()).copy_from_slice(data)`,
    ///   so use `write_buffer_with()` if you can do something smarter than that
    ///   [`copy_from_slice()`](slice::copy_from_slice). However, for small values
    ///   (e.g. a typical uniform buffer whose contents come from a `struct`),
    ///   there will likely be no difference, since the compiler will be able to
    ///   optimize out unnecessary copies regardless.
    ///
    /// * Currently on native platforms, for both of these methods, the staging
    ///   memory will be a new allocation. This will then be released after the
    ///   next submission finishes. To entirely avoid short-lived allocations, you might
    ///   be able to use [`StagingBelt`](crate::util::StagingBelt),
    ///   or buffers you explicitly create, map, and unmap yourself.
    pub fn write_buffer(&self, buffer: &Buffer, offset: BufferAddress, data: &[u8]) {
        self.inner.write_buffer(&buffer.inner, offset, data);
    }

    /// Prepares to write data to a buffer via a mapped staging buffer.
    ///
    /// This operation allocates a temporary buffer and then returns a
    /// [`QueueWriteBufferView`], which
    ///
    /// * dereferences to a `[u8]` of length `size`, and
    /// * when dropped, schedules a copy of its contents into `buffer` at `offset`.
    ///
    /// Therefore, this obtains the same result as [`Queue::write_buffer()`], but may
    /// allow you to skip one allocation and one copy of your data, if you are able to
    /// assemble your data directly into the returned [`QueueWriteBufferView`] instead of
    /// into a separate allocation like a [`Vec`](alloc::vec::Vec) first.
    ///
    /// The data must be written fully in-bounds, that is, `offset + size <= buffer.len()`.
    ///
    /// # Performance considerations
    ///
    /// * For small data not separately heap-allocated, there is no advantage of this
    ///   over [`Queue::write_buffer()`].
    ///
    /// * Reading from the returned view may be slow, and will not yield the current
    ///   contents of `buffer`. You should treat it as “write-only”.
    ///
    /// * Dropping the [`QueueWriteBufferView`] does *not* submit the
    ///   transfer to the GPU immediately. The transfer begins only on the next
    ///   call to [`Queue::submit()`] after the view is dropped, just before the
    ///   explicitly submitted commands. To get a set of scheduled transfers started
    ///   immediately, it's fine to call `queue.submit([])` with no command buffers at all.
    ///
    /// * Currently on native platforms, the staging memory will be a new allocation, which will
    ///   then be released after the next submission finishes. To entirely avoid short-lived
    ///   allocations, you might be able to use [`StagingBelt`](crate::util::StagingBelt),
    ///   or buffers you explicitly create, map, and unmap yourself.
    #[must_use]
    pub fn write_buffer_with<'a>(
        &'a self,
        buffer: &'a Buffer,
        offset: BufferAddress,
        size: BufferSize,
    ) -> Option<QueueWriteBufferView<'a>> {
        profiling::scope!("Queue::write_buffer_with");
        self.inner
            .validate_write_buffer(&buffer.inner, offset, size)?;
        let staging_buffer = self.inner.create_staging_buffer(size)?;
        Some(QueueWriteBufferView {
            queue: self,
            buffer,
            offset,
            inner: staging_buffer,
        })
    }

    /// Copies the bytes of `data` into into a texture.
    ///
    /// * `data` contains the texels to be written, which must be in
    ///   [the same format as the texture](TextureFormat).
    /// * `data_layout` describes the memory layout of `data`, which does not necessarily
    ///   have to have tightly packed rows.
    /// * `texture` specifies the texture to write into, and the location within the
    ///   texture (coordinate offset, mip level) that will be overwritten.
    /// * `size` is the size, in texels, of the region to be written.
    ///
    /// This method fails if `size` overruns the size of `texture`, or if `data` is too short.
    ///
    /// # Performance considerations
    ///
    /// This operation has the same performance considerations as [`Queue::write_buffer()`];
    /// see its documentation for details.
    ///
    /// However, since there is no “mapped texture” like a mapped buffer,
    /// alternate techniques for writing to textures will generally consist of first copying
    /// the data to a buffer, then using [`CommandEncoder::copy_buffer_to_texture()`], or in
    /// some cases a compute shader, to copy texels from that buffer to the texture.
    pub fn write_texture(
        &self,
        texture: TexelCopyTextureInfo<'_>,
        data: &[u8],
        data_layout: TexelCopyBufferLayout,
        size: Extent3d,
    ) {
        self.inner.write_texture(texture, data, data_layout, size);
    }

    /// Schedule a copy of data from `image` into `texture`.
    #[cfg(web)]
    pub fn copy_external_image_to_texture(
        &self,
        source: &wgt::CopyExternalImageSourceInfo,
        dest: wgt::CopyExternalImageDestInfo<&api::Texture>,
        size: Extent3d,
    ) {
        self.inner
            .copy_external_image_to_texture(source, dest, size);
    }

    /// Submits a series of finished command buffers for execution.
    pub fn submit<I: IntoIterator<Item = CommandBuffer>>(
        &self,
        command_buffers: I,
    ) -> SubmissionIndex {
        let mut command_buffers = command_buffers.into_iter().map(|comb| comb.buffer);

        let index = self.inner.submit(&mut command_buffers);

        SubmissionIndex { index }
    }

    /// Gets the amount of nanoseconds each tick of a timestamp query represents.
    ///
    /// Returns zero if timestamp queries are unsupported.
    ///
    /// Timestamp values are represented in nanosecond values on WebGPU, see `<https://gpuweb.github.io/gpuweb/#timestamp>`
    /// Therefore, this is always 1.0 on the web, but on wgpu-core a manual conversion is required.
    pub fn get_timestamp_period(&self) -> f32 {
        self.inner.get_timestamp_period()
    }

    /// Registers a callback when the previous call to submit finishes running on the gpu. This callback
    /// being called implies that all mapped buffer callbacks which were registered before this call will
    /// have been called.
    ///
    /// For the callback to complete, either `queue.submit(..)`, `instance.poll_all(..)`, or `device.poll(..)`
    /// must be called elsewhere in the runtime, possibly integrated into an event loop or run on a separate thread.
    ///
    /// The callback will be called on the thread that first calls the above functions after the gpu work
    /// has completed. There are no restrictions on the code you can run in the callback, however on native the
    /// call to the function will not complete until the callback returns, so prefer keeping callbacks short
    /// and used to set flags, send messages, etc.
    pub fn on_submitted_work_done(&self, callback: impl FnOnce() + Send + 'static) {
        self.inner.on_submitted_work_done(Box::new(callback));
    }

    /// Get the [`wgpu_hal`] device from this `Queue`.
    ///
    /// Find the Api struct corresponding to the active backend in [`wgpu_hal::api`],
    /// and pass that struct to the to the `A` type parameter.
    ///
    /// Returns a guard that dereferences to the type of the hal backend
    /// which implements [`A::Queue`].
    ///
    /// # Types
    ///
    /// The returned type depends on the backend:
    ///
    #[doc = crate::hal_type_vulkan!("Queue")]
    #[doc = crate::hal_type_metal!("Queue")]
    #[doc = crate::hal_type_dx12!("Queue")]
    #[doc = crate::hal_type_gles!("Queue")]
    ///
    /// # Errors
    ///
    /// This method will return None if:
    /// - The queue is not from the backend specified by `A`.
    /// - The queue is from the `webgpu` or `custom` backend.
    ///
    /// # Safety
    ///
    /// - The returned resource must not be destroyed unless the guard
    ///   is the last reference to it and it is not in use by the GPU.
    ///   The guard and handle may be dropped at any time however.
    /// - All the safety requirements of wgpu-hal must be upheld.
    ///
    /// [`A::Queue`]: hal::Api::Queue
    #[cfg(wgpu_core)]
    pub unsafe fn as_hal<A: wgc::hal_api::HalApi>(
        &self,
    ) -> Option<impl Deref<Target = A::Queue> + WasmNotSendSync> {
        let queue = self.inner.as_core_opt()?;
        unsafe { queue.context.queue_as_hal::<A>(queue) }
    }

    /// Compact a BLAS, it must have had [`Blas::prepare_compaction_async`] called on it and had the
    /// callback provided called.
    ///
    /// The returned BLAS is more restricted than a normal BLAS because it may not be rebuilt or
    /// compacted.
    pub fn compact_blas(&self, blas: &Blas) -> Blas {
        let (handle, dispatch) = self.inner.compact_blas(&blas.inner);
        Blas {
            handle,
            inner: dispatch,
        }
    }
}
