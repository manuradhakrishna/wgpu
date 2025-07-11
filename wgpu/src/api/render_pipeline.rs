use core::num::NonZeroU32;

use crate::*;

/// Handle to a rendering (graphics) pipeline.
///
/// A `RenderPipeline` object represents a graphics pipeline and its stages, bindings, vertex
/// buffers and targets. It can be created with [`Device::create_render_pipeline`].
///
/// Corresponds to [WebGPU `GPURenderPipeline`](https://gpuweb.github.io/gpuweb/#render-pipeline).
#[derive(Debug, Clone)]
pub struct RenderPipeline {
    pub(crate) inner: dispatch::DispatchRenderPipeline,
}
#[cfg(send_sync)]
static_assertions::assert_impl_all!(RenderPipeline: Send, Sync);

crate::cmp::impl_eq_ord_hash_proxy!(RenderPipeline => .inner);

impl RenderPipeline {
    /// Get an object representing the bind group layout at a given index.
    ///
    /// If this pipeline was created with a [default layout][RenderPipelineDescriptor::layout], then
    /// bind groups created with the returned `BindGroupLayout` can only be used with this pipeline.
    ///
    /// This method will raise a validation error if there is no bind group layout at `index`.
    pub fn get_bind_group_layout(&self, index: u32) -> BindGroupLayout {
        let layout = self.inner.get_bind_group_layout(index);
        BindGroupLayout { inner: layout }
    }

    #[cfg(custom)]
    /// Returns custom implementation of RenderPipeline (if custom backend and is internally T)
    pub fn as_custom<T: custom::RenderPipelineInterface>(&self) -> Option<&T> {
        self.inner.as_custom()
    }
}

/// Specifies an interpretation of the bytes of a vertex buffer as vertex attributes.
///
/// Use this in a [`RenderPipelineDescriptor`] to describe the format of the vertex buffers that
/// are passed to [`RenderPass::set_vertex_buffer()`].
///
/// Corresponds to [WebGPU `GPUVertexBufferLayout`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpuvertexbufferlayout).
///
/// # Example
///
/// The following example defines a `struct` with three fields,
/// and a [`VertexBufferLayout`] that contains [`VertexAttribute`]s for each field,
/// using the [`vertex_attr_array!`] macro to compute attribute offsets:
///
/// ```
/// #[repr(C, packed)]
/// struct Vertex {
///     foo: [f32; 2],
///     bar: f32,
///     baz: [u16; 4],
/// }
///
/// impl Vertex {
///     /// Layout to use with a buffer whose contents are a `[Vertex]`.
///     pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
///         array_stride: size_of::<Self>() as wgpu::BufferAddress,
///         step_mode: wgpu::VertexStepMode::Vertex,
///         attributes: &wgpu::vertex_attr_array![
///             0 => Float32x2,
///             1 => Float32,
///             2 => Uint16x4,
///         ],
///     };
/// }
///
/// # assert_eq!(Vertex::LAYOUT.attributes[2].offset, Vertex::LAYOUT.array_stride - 2 * 4);
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct VertexBufferLayout<'a> {
    /// The stride, in bytes, between elements of this buffer (between vertices).
    ///
    /// This must be a multiple of [`VERTEX_ALIGNMENT`].
    pub array_stride: BufferAddress,
    /// How often this vertex buffer is "stepped" forward.
    pub step_mode: VertexStepMode,
    /// The list of attributes which comprise a single vertex.
    pub attributes: &'a [VertexAttribute],
}
static_assertions::assert_impl_all!(VertexBufferLayout<'_>: Send, Sync);

/// Describes the vertex processing in a render pipeline.
///
/// For use in [`RenderPipelineDescriptor`].
///
/// Corresponds to [WebGPU `GPUVertexState`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpuvertexstate).
#[derive(Clone, Debug)]
pub struct VertexState<'a> {
    /// The compiled shader module for this stage.
    pub module: &'a ShaderModule,
    /// The name of the entry point in the compiled shader to use.
    ///
    /// If [`Some`], there must be a vertex-stage shader entry point with this name in `module`.
    /// Otherwise, expect exactly one vertex-stage entry point in `module`, which will be
    /// selected.
    // NOTE: keep phrasing in sync. with `ComputePipelineDescriptor::entry_point`
    // NOTE: keep phrasing in sync. with `FragmentState::entry_point`
    pub entry_point: Option<&'a str>,
    /// Advanced options for when this pipeline is compiled
    ///
    /// This implements `Default`, and for most users can be set to `Default::default()`
    pub compilation_options: PipelineCompilationOptions<'a>,
    /// The format of any vertex buffers used with this pipeline via
    /// [`RenderPass::set_vertex_buffer()`].
    ///
    /// The attribute locations and types specified in this layout must match the
    /// locations and types of the inputs to the `entry_point` function.
    pub buffers: &'a [VertexBufferLayout<'a>],
}
#[cfg(send_sync)]
static_assertions::assert_impl_all!(VertexState<'_>: Send, Sync);

/// Describes the fragment processing in a render pipeline.
///
/// For use in [`RenderPipelineDescriptor`].
///
/// Corresponds to [WebGPU `GPUFragmentState`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpufragmentstate).
#[derive(Clone, Debug)]
pub struct FragmentState<'a> {
    /// The compiled shader module for this stage.
    pub module: &'a ShaderModule,
    /// The name of the entry point in the compiled shader to use.
    ///
    /// If [`Some`], there must be a `@fragment` shader entry point with this name in `module`.
    /// Otherwise, expect exactly one fragment-stage entry point in `module`, which will be
    /// selected.
    // NOTE: keep phrasing in sync. with `ComputePipelineDescriptor::entry_point`
    // NOTE: keep phrasing in sync. with `VertexState::entry_point`
    pub entry_point: Option<&'a str>,
    /// Advanced options for when this pipeline is compiled
    ///
    /// This implements `Default`, and for most users can be set to `Default::default()`
    pub compilation_options: PipelineCompilationOptions<'a>,
    /// The color state of the render targets.
    pub targets: &'a [Option<ColorTargetState>],
}
#[cfg(send_sync)]
static_assertions::assert_impl_all!(FragmentState<'_>: Send, Sync);

/// Describes a render (graphics) pipeline.
///
/// For use with [`Device::create_render_pipeline`].
///
/// Corresponds to [WebGPU `GPURenderPipelineDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpurenderpipelinedescriptor).
#[derive(Clone, Debug)]
pub struct RenderPipelineDescriptor<'a> {
    /// Debug label of the pipeline. This will show up in graphics debuggers for easy identification.
    pub label: Label<'a>,
    /// The layout of bind groups for this pipeline.
    ///
    /// If this is set, then [`Device::create_render_pipeline`] will raise a validation error if
    /// the layout doesn't match what the shader module(s) expect.
    ///
    /// Using the same [`PipelineLayout`] for many [`RenderPipeline`] or [`ComputePipeline`]
    /// pipelines guarantees that you don't have to rebind any resources when switching between
    /// those pipelines.
    ///
    /// ## Default pipeline layout
    ///
    /// If `layout` is `None`, then the pipeline has a [default layout] created and used instead.
    /// The default layout is deduced from the shader modules.
    ///
    /// You can use [`RenderPipeline::get_bind_group_layout`] to create bind groups for use with the
    /// default layout. However, these bind groups cannot be used with any other pipelines. This is
    /// convenient for simple pipelines, but using an explicit layout is recommended in most cases.
    ///
    /// [default layout]: https://www.w3.org/TR/webgpu/#default-pipeline-layout
    pub layout: Option<&'a PipelineLayout>,
    /// The compiled vertex stage, its entry point, and the input buffers layout.
    pub vertex: VertexState<'a>,
    /// The properties of the pipeline at the primitive assembly and rasterization level.
    pub primitive: PrimitiveState,
    /// The effect of draw calls on the depth and stencil aspects of the output target, if any.
    pub depth_stencil: Option<DepthStencilState>,
    /// The multi-sampling properties of the pipeline.
    pub multisample: MultisampleState,
    /// The compiled fragment stage, its entry point, and the color targets.
    pub fragment: Option<FragmentState<'a>>,
    /// If the pipeline will be used with a multiview render pass, this indicates how many array
    /// layers the attachments will have.
    pub multiview: Option<NonZeroU32>,
    /// The pipeline cache to use when creating this pipeline.
    pub cache: Option<&'a PipelineCache>,
}
#[cfg(send_sync)]
static_assertions::assert_impl_all!(RenderPipelineDescriptor<'_>: Send, Sync);
