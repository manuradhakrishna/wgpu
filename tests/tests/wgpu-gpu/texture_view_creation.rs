use wgpu::*;
use wgpu_test::{gpu_test, FailureCase, GpuTestConfiguration, GpuTestInitializer, TestParameters};

pub fn all_tests(vec: &mut Vec<GpuTestInitializer>) {
    vec.extend([
        STENCIL_ONLY_VIEW_CREATION,
        DEPTH_ONLY_VIEW_CREATION,
        SHARED_USAGE_VIEW_CREATION,
    ]);
}

#[gpu_test]
static STENCIL_ONLY_VIEW_CREATION: GpuTestConfiguration = GpuTestConfiguration::new()
    .parameters(
        TestParameters::default()
            .skip(FailureCase::webgl2()) // WebGL doesn't have stencil only views
            .limits(wgpu::Limits::downlevel_defaults())
            .enable_noop(),
    )
    .run_async(|ctx| async move {
        for format in [TextureFormat::Stencil8, TextureFormat::Depth24PlusStencil8] {
            let texture = ctx.device.create_texture(&TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: 256,
                    height: 256,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::COPY_SRC
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let _view = texture.create_view(&TextureViewDescriptor {
                aspect: TextureAspect::StencilOnly,
                ..Default::default()
            });
        }
    });

#[gpu_test]
static DEPTH_ONLY_VIEW_CREATION: GpuTestConfiguration = GpuTestConfiguration::new()
    .parameters(TestParameters::default().enable_noop())
    .run_async(|ctx| async move {
        for format in [
            TextureFormat::Depth16Unorm,
            TextureFormat::Depth24Plus,
            TextureFormat::Depth24PlusStencil8,
        ] {
            let texture = ctx.device.create_texture(&TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: 256,
                    height: 256,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::COPY_SRC
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let _view = texture.create_view(&TextureViewDescriptor {
                aspect: TextureAspect::DepthOnly,
                ..Default::default()
            });
        }
    });

#[gpu_test]
static SHARED_USAGE_VIEW_CREATION: GpuTestConfiguration = GpuTestConfiguration::new()
    .parameters(
        TestParameters::default()
            .downlevel_flags(DownlevelFlags::VIEW_FORMATS)
            .enable_noop(),
    )
    .run_async(|ctx| async move {
        {
            let (texture_format, view_format) =
                (TextureFormat::Rgba8Unorm, TextureFormat::Rgba8UnormSrgb);
            let texture = ctx.device.create_texture(&TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: 256,
                    height: 256,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: texture_format,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[TextureFormat::Rgba8UnormSrgb],
            });
            let _view = texture.create_view(&TextureViewDescriptor {
                aspect: TextureAspect::All,
                format: Some(view_format),
                usage: Some(
                    TextureUsages::COPY_DST
                        | TextureUsages::TEXTURE_BINDING
                        | TextureUsages::RENDER_ATTACHMENT,
                ),
                ..Default::default()
            });
        }
    });
