use std::mem;
use wgpu::util::BufferInitDescriptor;
use wgpu::{
    util::DeviceExt, Blas, BlasBuildEntry, BlasGeometries, BlasGeometrySizeDescriptors,
    BlasTriangleGeometry, BlasTriangleGeometrySizeDescriptor, Buffer, CreateBlasDescriptor,
    CreateTlasDescriptor, Tlas, TlasInstance,
};
use wgpu::{
    AccelerationStructureFlags, AccelerationStructureGeometryFlags,
    AccelerationStructureUpdateMode, BufferAddress, BufferUsages, VertexFormat,
};
use wgpu_test::TestingContext;

mod as_build;
mod as_create;
mod as_use_after_free;
mod limits;
mod scene;
mod shader;

pub fn all_tests(tests: &mut Vec<wgpu_test::GpuTestInitializer>) {
    as_build::all_tests(tests);
    as_create::all_tests(tests);
    as_use_after_free::all_tests(tests);
    limits::all_tests(tests);
    scene::all_tests(tests);
    shader::all_tests(tests);
}

fn acceleration_structure_limits() -> wgpu::Limits {
    wgpu::Limits::default().using_minimum_supported_acceleration_structure_values()
}

pub struct AsBuildContext {
    vertices: Buffer,
    blas_size: BlasTriangleGeometrySizeDescriptor,
    blas: Blas,
    // Putting this last, forces the BLAS to die before the TLAS.
    tlas: Tlas,
}

impl AsBuildContext {
    pub fn new(
        ctx: &TestingContext,
        additional_blas_flags: AccelerationStructureFlags,
        additional_tlas_flags: AccelerationStructureFlags,
    ) -> Self {
        let vertices = ctx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &[0; mem::size_of::<[[f32; 3]; 3]>()],
            usage: BufferUsages::BLAS_INPUT,
        });

        let blas_size = BlasTriangleGeometrySizeDescriptor {
            vertex_format: VertexFormat::Float32x3,
            vertex_count: 3,
            index_format: None,
            index_count: None,
            flags: AccelerationStructureGeometryFlags::empty(),
        };

        let blas = ctx.device.create_blas(
            &CreateBlasDescriptor {
                label: Some("BLAS"),
                flags: AccelerationStructureFlags::PREFER_FAST_TRACE | additional_blas_flags,
                update_mode: AccelerationStructureUpdateMode::Build,
            },
            BlasGeometrySizeDescriptors::Triangles {
                descriptors: vec![blas_size.clone()],
            },
        );

        let mut tlas = ctx.device.create_tlas(&CreateTlasDescriptor {
            label: Some("TLAS"),
            max_instances: 1,
            flags: AccelerationStructureFlags::PREFER_FAST_TRACE | additional_tlas_flags,
            update_mode: AccelerationStructureUpdateMode::Build,
        });

        tlas[0] = Some(TlasInstance::new(
            &blas,
            [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
            0,
            0xFF,
        ));

        Self {
            vertices,
            blas_size,
            blas,
            tlas,
        }
    }

    pub fn blas_build_entry(&self) -> BlasBuildEntry {
        BlasBuildEntry {
            blas: &self.blas,
            geometry: BlasGeometries::TriangleGeometries(vec![BlasTriangleGeometry {
                size: &self.blas_size,
                vertex_buffer: &self.vertices,
                first_vertex: 0,
                vertex_stride: mem::size_of::<[f32; 3]>() as BufferAddress,
                index_buffer: None,
                first_index: None,
                transform_buffer: None,
                transform_buffer_offset: None,
            }]),
        }
    }
}
