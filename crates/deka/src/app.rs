use crate::renderer::{gui::utils::TVertex, shaders};

use super::{DAL, renderer::gui::GuiRenderer};
use std::sync::Arc;
use vulkano::{
    Validated, VulkanError, VulkanLibrary,
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassBeginInfo,
        SubpassContents, allocator::StandardCommandBufferAllocator,
    },
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
        physical::PhysicalDeviceType,
    },
    format::Format,
    image::{Image, ImageCreateInfo, ImageType, ImageUsage, view::ImageView},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, StandardMemoryAllocator},
    pipeline::{
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::{CullMode, RasterizationState},
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Scissor, Viewport, ViewportState},
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{
        Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo, acquire_next_image,
    },
    sync::{self, GpuFuture, future::FenceSignalFuture},
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::Window,
};

use log::{debug, warn};

pub struct Application {
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    rcx: Option<RenderContext>,
    gui_renderer: GuiRenderer,
    dal: DAL,
}

struct RenderContext {
    window: Arc<Window>,
    swapchain: Arc<Swapchain>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline: Arc<GraphicsPipeline>,
    viewport: Viewport,
    recreate_swapchain: bool,
    fences: Vec<Option<Arc<FenceSignalFuture<Box<dyn GpuFuture>>>>>,
}

fn window_size_dependent_setup(
    images: &[Arc<Image>],
    render_pass: &Arc<RenderPass>,
    memory_allocator: &Arc<StandardMemoryAllocator>,
) -> Vec<Arc<Framebuffer>> {
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();

            let depth_buffer = Image::new(
                memory_allocator.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: Format::D16_UNORM, // Must match RenderPass
                    extent: image.extent(),
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .unwrap();

            let depth_view = ImageView::new_default(depth_buffer).unwrap();

            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view, depth_view],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}

impl Application {
    pub fn new(event_loop: &EventLoop<()>, dal: DAL) -> Self {
        let library = VulkanLibrary::new().unwrap();

        let required_extensions = Surface::required_extensions(event_loop).unwrap();
        let layers = vec![String::from("VK_LAYER_KHRONOS_validation")];
        let available_layers = library.layer_properties().unwrap();
        if available_layers
            .into_iter()
            .all(|l| l.name() != "VK_LAYER_KHRONOS_validation")
        {
            warn!(
                "VK_LAYER_KHRONOS_validation is not available. Install the Vulkan SDK to get validation layers."
            )
        }

        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                enabled_layers: layers,
                ..Default::default()
            },
        )
        .unwrap();

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.presentation_support(i as u32, event_loop).unwrap()
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::IntegratedGpu => 0,
                PhysicalDeviceType::DiscreteGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .expect("[error::vulkan]: No suitable physical device found");

        debug!(
            "using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type
        );

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .unwrap();

        let queue = queues.next().unwrap();
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let gui_renderer = GuiRenderer::new(memory_allocator.clone());

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let rcx = None;

        Application {
            instance,
            device,
            queue,
            command_buffer_allocator,
            gui_renderer,
            rcx,
            dal,
        }
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_resizable(self.dal.attr.resizable)
                        .with_title(&self.dal.attr.title)
                        .with_inner_size(PhysicalSize::new(
                            self.dal.attr.size.0,
                            self.dal.attr.size.1,
                        ))
                        .with_decorations(false),
                )
                .unwrap(),
        );

        let surface = Surface::from_window(self.instance.clone(), window.clone()).unwrap();
        let window_size = window.inner_size();

        let (swapchain, images) = {
            let surface_capabilities = self
                .device
                .physical_device()
                .surface_capabilities(&surface, Default::default())
                .unwrap();
            let (image_format, _) = self
                .device
                .physical_device()
                .surface_formats(&surface, Default::default())
                .unwrap()[0];

            Swapchain::new(
                self.device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count.max(2),
                    image_format,
                    image_extent: window_size.into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT,
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .into_iter()
                        .next()
                        .unwrap(),
                    ..Default::default()
                },
            )
            .unwrap()
        };

        self.gui_renderer.resize(images.len());

        let render_pass = vulkano::single_pass_renderpass!(
            self.device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth_stencil: {
                    format: Format::D16_UNORM, // Standard depth format
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                }
            },

            pass: {
                color: [color],
                depth_stencil: {depth_stencil},
            }
        )
        .unwrap();

        let framebuffers =
            window_size_dependent_setup(&images, &render_pass, &self.gui_renderer.memory_allocator);

        let pipeline = {
            let vs = shaders::rectvs::load(self.device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();

            let fs = shaders::rectfs::load(self.device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();

            let vertex_input_state = TVertex::per_vertex().definition(&vs).unwrap();

            let stages = [
                PipelineShaderStageCreateInfo::new(vs),
                PipelineShaderStageCreateInfo::new(fs),
            ];

            let layout = PipelineLayout::new(
                self.device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                    .into_pipeline_layout_create_info(self.device.clone())
                    .unwrap(),
            )
            .unwrap();

            let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

            GraphicsPipeline::new(
                self.device.clone(),
                None,
                GraphicsPipelineCreateInfo {
                    stages: stages.into_iter().collect(),
                    vertex_input_state: Some(vertex_input_state),
                    input_assembly_state: Some(InputAssemblyState::default()),
                    viewport_state: Some(ViewportState::default()),
                    rasterization_state: Some(RasterizationState {
                        cull_mode: CullMode::None,
                        ..Default::default()
                    }),
                    multisample_state: Some(MultisampleState::default()),
                    color_blend_state: Some(ColorBlendState::with_attachment_states(
                        subpass.num_color_attachments(),
                        ColorBlendAttachmentState {
                            blend: Some(
                                vulkano::pipeline::graphics::color_blend::AttachmentBlend::alpha(),
                            ),
                            ..Default::default()
                        },
                    )),
                    dynamic_state: [DynamicState::Viewport, DynamicState::Scissor]
                        .into_iter()
                        .collect(),
                    subpass: Some(subpass.into()),
                    depth_stencil_state: Some(DepthStencilState {
                        depth: Some(DepthState {
                            compare_op: CompareOp::LessOrEqual, // Closer things overwrite further things
                            write_enable: true,
                        }),
                        ..Default::default()
                    }),
                    ..GraphicsPipelineCreateInfo::layout(layout)
                },
            )
            .unwrap()
        };

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: window_size.into(),
            depth_range: 0.0..=1.0,
        };

        let recreate_swapchain = false;
        let fences = vec![None; images.len()];

        self.rcx = Some(RenderContext {
            window,
            swapchain,
            render_pass,
            framebuffers,
            pipeline,
            viewport,
            recreate_swapchain,
            fences,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let rcx = self.rcx.as_mut().unwrap();

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.dal.mouse_pos = position;
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                self.dal.click(button, state.is_pressed());
            }

            WindowEvent::Resized(PhysicalSize { width, height }) => {
                rcx.recreate_swapchain = true;
                self.dal.resize(width, height);
            }
            WindowEvent::RedrawRequested => {
                let window_size = rcx.window.inner_size();

                if window_size.width == 0 || window_size.height == 0 {
                    return;
                }

                if rcx.recreate_swapchain {
                    let (new_swapchain, new_images) = rcx
                        .swapchain
                        .recreate(SwapchainCreateInfo {
                            image_extent: window_size.into(),
                            ..rcx.swapchain.create_info()
                        })
                        .expect("failed to recreate swapchain");

                    rcx.swapchain = new_swapchain;
                    rcx.framebuffers = window_size_dependent_setup(
                        &new_images,
                        &rcx.render_pass,
                        &self.gui_renderer.memory_allocator,
                    );
                    rcx.viewport.extent = window_size.into();
                    rcx.recreate_swapchain = false;
                    self.gui_renderer.resize(new_images.len());
                    rcx.fences.resize(new_images.len(), None);
                }

                let (image_index, suboptimal, acquire_future) = match acquire_next_image(
                    rcx.swapchain.clone(),
                    None,
                )
                .map_err(Validated::unwrap)
                {
                    Ok(r) => r,
                    Err(VulkanError::OutOfDate) => {
                        rcx.recreate_swapchain = true;
                        return;
                    }
                    Err(e) => panic!("[error::vulkan]: failed to acquire next image: {e}"),
                };

                if suboptimal {
                    rcx.recreate_swapchain = true;
                }

                if let Some(image_fence) = &mut rcx.fences[image_index as usize] {
                    image_fence.wait(None).unwrap();
                    image_fence.cleanup_finished();
                }

                let mut builder = AutoCommandBufferBuilder::primary(
                    self.command_buffer_allocator.clone(),
                    self.queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .unwrap();

                let scissor = Scissor {
                    offset: [rcx.viewport.offset[0] as u32, rcx.viewport.offset[1] as u32],
                    extent: [rcx.viewport.extent[0] as u32, rcx.viewport.extent[1] as u32],
                };

                builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![
                                Some([0., 0., 0., 0.0].into()), // Color
                                Some(1.0f32.into()),            // Depth
                            ],
                            ..RenderPassBeginInfo::framebuffer(
                                rcx.framebuffers[image_index as usize].clone(),
                            )
                        },
                        SubpassBeginInfo {
                            contents: SubpassContents::Inline,
                            ..Default::default()
                        },
                    )
                    .unwrap()
                    .set_viewport(0, [rcx.viewport.clone()].into_iter().collect())
                    .unwrap()
                    .set_scissor(0, [scissor].into_iter().collect())
                    .unwrap()
                    .bind_pipeline_graphics(rcx.pipeline.clone())
                    .unwrap();

                self.dal.compute_layout();
                let commands = self.dal.render();
                let size = [window_size.width as f32, window_size.height as f32];

                if commands.is_empty() {
                    debug!("Frame {}: No draw commands generated!", image_index);
                }

                self.gui_renderer.upload_draw_commands(
                    image_index as usize,
                    &commands,
                    size,
                    &mut self.dal,
                );
                self.gui_renderer.render(image_index as usize, &mut builder);

                builder.end_render_pass(Default::default()).unwrap();

                let command_buffer = builder.build().unwrap();

                let logic_future = sync::now(self.device.clone())
                    .join(acquire_future)
                    .then_execute(self.queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(
                        self.queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(
                            rcx.swapchain.clone(),
                            image_index,
                        ),
                    )
                    .boxed();

                let fence_future = logic_future.then_signal_fence_and_flush();

                match fence_future.map_err(Validated::unwrap) {
                    Ok(future) => {
                        rcx.fences[image_index as usize] = Some(Arc::new(future));
                    }
                    Err(VulkanError::OutOfDate) => {
                        rcx.recreate_swapchain = true;
                        // For safe recovery, we can just clear the fence or keep the old one
                        // rcx.fences[image_index as usize] = None;
                    }
                    Err(e) => {
                        panic!("[error::vulkan]: failed to flush future: {e}");
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.dal.is_dirty() {
            let rcx = self.rcx.as_mut().unwrap();
            rcx.window.request_redraw();
            event_loop.set_control_flow(ControlFlow::Poll);
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}
