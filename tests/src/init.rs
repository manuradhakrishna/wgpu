use wgpu::{Adapter, Backends, Device, Features, Instance, Limits, Queue};

use crate::{report::AdapterReport, TestParameters};

/// Initialize the logger for the test runner.
pub fn init_logger() {
    // We don't actually care if it fails
    #[cfg(not(target_arch = "wasm32"))]
    let _ = env_logger::try_init();
    #[cfg(target_arch = "wasm32")]
    let _ = console_log::init_with_level(log::Level::Info);
}

/// Initialize a wgpu instance with the options from the environment.
pub fn initialize_instance(backends: wgpu::Backends, params: &TestParameters) -> Instance {
    // We ignore `WGPU_BACKEND` for now, merely using test filtering to only run a single backend's tests.
    //
    // We can potentially work support back into the test runner in the future, but as the adapters are matched up
    // based on adapter index, removing some backends messes up the indexes in annoying ways.
    //
    // WORKAROUND for https://github.com/rust-lang/cargo/issues/7160:
    // `--no-default-features` is not passed through correctly to the test runner.
    // We use it whenever we want to explicitly run with webgl instead of webgpu.
    // To "disable" webgpu regardless, we do this by removing the webgpu backend whenever we see
    // the webgl feature.
    let backends = if cfg!(feature = "webgl") {
        backends - wgpu::Backends::BROWSER_WEBGPU
    } else {
        backends
    };
    // Some tests need to be able to force demote to FXC, to specifically test workarounds for FXC
    // behavior.
    let dx12_shader_compiler = if params.force_fxc {
        wgpu::Dx12Compiler::Fxc
    } else {
        wgpu::Dx12Compiler::from_env().unwrap_or(wgpu::Dx12Compiler::StaticDxc)
    };
    // The defaults for debugging, overridden by the environment, overridden by the test parameters.
    let flags = wgpu::InstanceFlags::debugging()
        .with_env()
        .union(params.required_instance_flags);

    Instance::new(&wgpu::InstanceDescriptor {
        backends,
        flags,
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds {
            for_resource_creation: Some(99),
            for_device_loss: None,
        },
        backend_options: wgpu::BackendOptions {
            dx12: wgpu::Dx12BackendOptions {
                shader_compiler: dx12_shader_compiler,
            },
            gl: wgpu::GlBackendOptions {
                fence_behavior: if cfg!(target_family = "wasm") {
                    // On WebGL, you cannot call Poll(Wait) with any timeout. This is because the
                    // browser does not things to block. However all of our tests are written to
                    // expect this behavior. This is the workaround to allow this to work.
                    //
                    // However on native you can wait, so we want to ensure that behavior as well.
                    wgpu::GlFenceBehavior::AutoFinish
                } else {
                    wgpu::GlFenceBehavior::Normal
                },
                ..Default::default()
            }
            .with_env(),
            // Allow the noop backend to be used in tests. This will not be used unless
            // WGPU_GPU_TESTS_USE_NOOP_BACKEND env var is set, because wgpu-info will not
            // enumerate the noop backend.
            //
            // However, we use wasm_bindgen_test to run tests on wasm, and wgpu
            // will chose the noop on wasm32 for some reason.
            noop: wgpu::NoopBackendOptions {
                enable: !cfg!(target_arch = "wasm32"),
            },
        },
    })
}

/// Initialize a wgpu adapter, using the given adapter report to match the adapter.
pub async fn initialize_adapter(
    adapter_report: Option<&AdapterReport>,
    params: &TestParameters,
) -> (Instance, Adapter, Option<SurfaceGuard>) {
    let backends = adapter_report
        .map(|report| Backends::from(report.info.backend))
        .unwrap_or_default();

    let instance = initialize_instance(backends, params);
    #[allow(unused_variables)]
    let surface: Option<wgpu::Surface>;
    let surface_guard: Option<SurfaceGuard>;

    #[allow(unused_assignments)]
    // Create a canvas if we need a WebGL2RenderingContext to have a working device.
    #[cfg(not(all(
        target_arch = "wasm32",
        any(target_os = "emscripten", feature = "webgl")
    )))]
    {
        surface = None;
        surface_guard = None;
    }
    #[cfg(all(
        target_arch = "wasm32",
        any(target_os = "emscripten", feature = "webgl")
    ))]
    {
        // On wasm, append a canvas to the document body for initializing the adapter
        let canvas = initialize_html_canvas();

        surface = Some(
            instance
                .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
                .expect("could not create surface from canvas"),
        );

        surface_guard = Some(SurfaceGuard { canvas });
    }

    cfg_if::cfg_if! {
        if #[cfg(not(target_arch = "wasm32"))] {
            let adapter_iter = instance.enumerate_adapters(backends);
            let adapter = adapter_iter.into_iter()
                // If we have a report, we only want to match the adapter with the same info.
                //
                // If we don't have a report, we just take the first adapter.
                .find(|adapter| if let Some(adapter_report) = adapter_report {
                    adapter.get_info() == adapter_report.info
                } else {
                    true
                });
            let Some(adapter) = adapter else {
                panic!(
                    "Could not find adapter with info {:#?} in {:#?}",
                    adapter_report.map(|r| &r.info),
                    instance.enumerate_adapters(backends).into_iter().map(|a| a.get_info()).collect::<Vec<_>>(),
                );
            };
        } else {
            let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: surface.as_ref(),
                ..Default::default()
            }).await.unwrap();
        }
    }

    log::info!("Testing using adapter: {:#?}", adapter.get_info());

    (instance, adapter, surface_guard)
}

/// Initialize a wgpu device from a given adapter.
pub async fn initialize_device(
    adapter: &Adapter,
    features: Features,
    limits: Limits,
) -> (Device, Queue) {
    let bundle = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: features,
            required_limits: limits,
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            trace: wgpu::Trace::Off,
        })
        .await;

    match bundle {
        Ok(b) => b,
        Err(e) => panic!("Failed to initialize device: {e}"),
    }
}

/// Create a canvas for testing.
#[cfg(target_arch = "wasm32")]
pub fn initialize_html_canvas() -> web_sys::HtmlCanvasElement {
    use wasm_bindgen::JsCast;

    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let canvas = doc.create_element("Canvas").unwrap();
            canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok()
        })
        .expect("couldn't create canvas")
}

pub struct SurfaceGuard {
    #[cfg(target_arch = "wasm32")]
    #[allow(unused)]
    canvas: web_sys::HtmlCanvasElement,
}

impl SurfaceGuard {
    #[cfg(all(
        target_arch = "wasm32",
        any(target_os = "emscripten", feature = "webgl")
    ))]
    pub(crate) fn check_for_unreported_errors(&self) -> bool {
        use wasm_bindgen::JsCast;

        self.canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .unwrap()
            .get_error()
            != web_sys::WebGl2RenderingContext::NO_ERROR
    }
}
