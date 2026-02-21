#![allow(clippy::modulo_one)]
use ash::vk::{
    self, AccessFlags, Buffer, CommandBuffer, CommandPool, Extent3D, Fence, Format, Handle,
    ImageCreateInfo, ImageUsageFlags, PipelineStageFlags, RenderPass, Semaphore,
    SemaphoreCreateInfo,
};
use iron_oxide::{
    graphics::{Ressources, Swapchain, VkBase, VulkanImage},
    primitives::Matrix4,
    ui::Ui,
};
use std::{
    cell::RefCell,
    ptr,
    rc::Rc,
    thread::sleep,
    time::{Duration, Instant},
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    app::{DEBUG_PERF, VSYNC},
    render_assets::RenderAssets,
};

// Max frames in flight
pub const MFIF: usize = 1;

#[allow(unused)]
#[derive(Clone, Copy)]
pub enum MemType {
    Host,
    DeviceLocal,
    Lazy,
}

pub struct VulkanRender {
    pub base: VkBase,

    pub cmd_pool: CommandPool,
    pub command_buffers: [CommandBuffer; MFIF],

    pub uniform_buffers: [Buffer; MFIF],
    pub uniform_buffers_mapped: [*mut Matrix4; MFIF],

    image_available_semaphores: [Semaphore; MFIF],
    in_flight_fences: [Fence; MFIF],
    render_finsih_semaphores: Vec<Semaphore>,

    pub render_pass: RenderPass,

    pub window_size: PhysicalSize<u32>,
    pub depth_image: VulkanImage,
    pub swapchain: Swapchain,

    pub ui: Rc<RefCell<Ui>>,
    pub ressources: Ressources,

    pub current_frame: usize,
}

impl VulkanRender {
    pub fn create(window: &Window, ui_state: Rc<RefCell<Ui>>) -> Self {
        let (base, surface_loader, surface) =
            VkBase::create(0, DEBUG_PERF, vk::API_VERSION_1_2, c"Home Storage", window);

        let window_size = window.inner_size();

        let present_mode = if VSYNC {
            vk::PresentModeKHR::FIFO
        } else {
            vk::PresentModeKHR::IMMEDIATE
        };

        let swapchain = Swapchain::new(&base, present_mode, surface_loader, surface, window_size);
        let render_pass = Self::create_render_pass(&base, swapchain.format);

        let ressources = Ressources::new(&base);

        let cmd_pool = Self::create_cmd_pool(&base);
        let depth_image = VulkanImage::default();

        let command_buffers = Self::create_command_buffers(&base.device, cmd_pool);
        let (image_available_semaphores, in_flight_fences) = Self::create_sync_object(&base.device);

        let mut this = Self {
            base,

            cmd_pool,
            command_buffers,

            uniform_buffers: [Buffer::null(); MFIF],
            uniform_buffers_mapped: [ptr::null_mut(); MFIF],

            image_available_semaphores,
            in_flight_fences,
            render_finsih_semaphores: Vec::new(),

            render_pass,

            window_size,
            depth_image,
            swapchain,

            ui: ui_state,
            ressources,
            current_frame: 0,
        };

        this.create_depth_resources(Extent3D {
            width: window_size.width,
            height: window_size.height,
            depth: 1,
        });
        this.swapchain
            .recreate(&this.base, this.render_pass, this.depth_image.view);

        this.render_finsih_semaphores = (0..this.swapchain.image_views.len())
            .map(|_| unsafe {
                this.base
                    .device
                    .create_semaphore(&SemaphoreCreateInfo::default(), None)
                    .unwrap()
            })
            .collect();

        this
    }

    fn create_render_pass(base: &VkBase, format: vk::SurfaceFormatKHR) -> RenderPass {
        let (clear, depth, has_previus, is_final) = (true, true, false, true);

        let color_attachment = vk::AttachmentDescription {
            format: format.format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: if clear {
                vk::AttachmentLoadOp::CLEAR
            } else {
                vk::AttachmentLoadOp::DONT_CARE
            },
            store_op: if is_final {
                vk::AttachmentStoreOp::STORE
            } else {
                vk::AttachmentStoreOp::DONT_CARE
            },
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: if has_previus {
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            } else {
                vk::ImageLayout::UNDEFINED
            },
            final_layout: if is_final {
                vk::ImageLayout::PRESENT_SRC_KHR
            } else {
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            },
            flags: vk::AttachmentDescriptionFlags::empty(),
        };

        let depth_attachment = vk::AttachmentDescription {
            format: Format::D16_UNORM,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            flags: vk::AttachmentDescriptionFlags::empty(),
        };

        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let attachments: &[_] = if depth {
            &[color_attachment, depth_attachment]
        } else {
            &[color_attachment]
        };

        let subpasses = [vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_ref,
            p_depth_stencil_attachment: if depth {
                &depth_attachment_ref
            } else {
                ptr::null()
            },
            ..Default::default()
        }];

        let dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            src_access_mask: vk::AccessFlags::empty(),
            dst_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | PipelineStageFlags::LATE_FRAGMENT_TESTS,
            dst_access_mask: if depth {
                AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE | AccessFlags::COLOR_ATTACHMENT_WRITE
            } else {
                AccessFlags::COLOR_ATTACHMENT_WRITE
            },
            dependency_flags: vk::DependencyFlags::empty(),
        }];

        let create_info = vk::RenderPassCreateInfo {
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            subpass_count: subpasses.len() as u32,
            p_subpasses: subpasses.as_ptr(),
            dependency_count: dependencies.len() as u32,
            p_dependencies: dependencies.as_ptr(),
            ..Default::default()
        };

        unsafe { base.device.create_render_pass(&create_info, None).unwrap() }
    }

    fn create_cmd_pool(base: &VkBase) -> CommandPool {
        let create_info = vk::CommandPoolCreateInfo {
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER
                | vk::CommandPoolCreateFlags::TRANSIENT,
            queue_family_index: base.queue_family_index,
            ..Default::default()
        };

        unsafe { base.device.create_command_pool(&create_info, None).unwrap() }
    }

    fn create_command_buffers(
        device: &ash::Device,
        cmd_pool: CommandPool,
    ) -> [CommandBuffer; MFIF] {
        let allocate_info = vk::CommandBufferAllocateInfo {
            command_pool: cmd_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: MFIF as u32,
            ..Default::default()
        };

        let vec = unsafe { device.allocate_command_buffers(&allocate_info).unwrap() };
        let mut buffers = [CommandBuffer::null(); MFIF];

        buffers.copy_from_slice(&vec);

        buffers
    }

    fn create_sync_object(device: &ash::Device) -> ([Semaphore; MFIF], [Fence; MFIF]) {
        let mut image_available_semaphores = [Semaphore::null(); MFIF];
        let mut in_flight_fences = [Fence::null(); MFIF];

        unsafe {
            let create_info = vk::SemaphoreCreateInfo::default();
            for semaphore in &mut image_available_semaphores {
                *semaphore = device.create_semaphore(&create_info, None).unwrap();
            }

            let create_info = vk::FenceCreateInfo {
                flags: vk::FenceCreateFlags::SIGNALED,
                ..Default::default()
            };
            for fence in &mut in_flight_fences {
                *fence = device.create_fence(&create_info, None).unwrap();
            }
        }

        (image_available_semaphores, in_flight_fences)
    }

    fn create_depth_resources(&mut self, extent: Extent3D) {
        let create_info = ImageCreateInfo {
            image_type: vk::ImageType::TYPE_2D,
            format: Format::D16_UNORM,
            extent,
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
                | ImageUsageFlags::TRANSIENT_ATTACHMENT,
            ..Default::default()
        };

        self.depth_image = VulkanImage::create(&self.base, &create_info);
        let requirements = unsafe {
            self.base
                .device
                .get_image_memory_requirements(self.depth_image.image)
        };

        let mem = &self.ressources.mem_manager.memory_pool[MemType::Lazy as usize];
        let allocation_size = requirements.size;
        if mem.memory.is_null() {
            self.ressources.mem_manager.allocate_memory(
                &self.base,
                self.ressources.mem_manager.lazy,
                allocation_size,
                MemType::Lazy as usize,
            );
        } else {
            self.ressources.mem_manager.reallocate_memory(
                &self.base,
                allocation_size,
                MemType::Lazy as usize,
            );
        }

        let layout = vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
        self.ressources.mem_manager.create_image(
            &self.base,
            MemType::Lazy as usize,
            self.cmd_pool,
            &mut self.depth_image,
            layout,
        );

        self.depth_image.create_view(&self.base);
    }

    pub fn draw_frame(&mut self) {
        if self.window_size.width == 0 || self.window_size.height == 0 {
            sleep(Duration::from_millis(200));
            return;
        }

        let in_flight_fence = self.in_flight_fences[self.current_frame];
        let available_semaphore = self.image_available_semaphores[self.current_frame];

        unsafe {
            self.base
                .device
                .wait_for_fences(&[in_flight_fence], true, u64::MAX)
                .unwrap();
            self.base.device.reset_fences(&[in_flight_fence]).unwrap();
            self.base
                .device
                .reset_command_buffer(
                    self.command_buffers[self.current_frame],
                    vk::CommandBufferResetFlags::empty(),
                )
                .unwrap();
        };

        let image_index = unsafe {
            match self.swapchain.loader.acquire_next_image(
                self.swapchain.inner,
                u64::MAX,
                available_semaphore,
                Fence::null(),
            ) {
                Ok(result) => {
                    if result.1 {
                        return;
                    }
                    result.0
                }
                Err(_) => return,
            }
        };

        let render_finsih_semaphore = self.render_finsih_semaphores[image_index as usize];
        let command_buffer = self.command_buffers[self.current_frame];

        self.record_command_buffer(image_index, command_buffer);

        let submit_info = vk::SubmitInfo {
            p_wait_semaphores: &available_semaphore,
            wait_semaphore_count: 1,
            p_wait_dst_stage_mask: &PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            command_buffer_count: 1,
            p_command_buffers: &command_buffer,
            signal_semaphore_count: 1,
            p_signal_semaphores: &render_finsih_semaphore,
            ..Default::default()
        };

        if self
            .base
            .queue_submit(&[submit_info], in_flight_fence)
            .is_err()
        {
            return;
        }

        let present_info = vk::PresentInfoKHR {
            wait_semaphore_count: 1,
            p_wait_semaphores: &render_finsih_semaphore,
            swapchain_count: 1,
            p_swapchains: &self.swapchain.inner,
            p_image_indices: &image_index,
            ..Default::default()
        };

        if unsafe {
            self.swapchain
                .loader
                .queue_present(self.base.queue, &present_info)
                .is_err()
        } {
            return;
        }

        self.current_frame = (self.current_frame + 1) % MFIF
    }

    fn record_command_buffer(&mut self, index: u32, command_buffer: CommandBuffer) {
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 0.0,
                    stencil: 0,
                },
            },
        ];

        let render_pass_info = vk::RenderPassBeginInfo {
            render_pass: self.render_pass,
            framebuffer: self.swapchain.framebuffers[index as usize],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: self.window_size.width,
                    height: self.window_size.height,
                },
            },
            clear_value_count: clear_values.len() as _,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };

        let view_port = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.window_size.width as f32,
            height: self.window_size.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: self.window_size.width,
                height: self.window_size.height,
            },
        };

        let device = &self.base.device;

        let begin_info = vk::CommandBufferBeginInfo {
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            ..Default::default()
        };

        let mut ui = self.ui.borrow_mut();

        if DEBUG_PERF {
            let start = Instant::now();
            ui.update(&self.base, &mut self.ressources, MFIF);
            println!("CPU to GPU time: {:?}", start.elapsed());
        } else {
            ui.update(&self.base, &mut self.ressources, MFIF);
        }

        unsafe {
            device
                .begin_command_buffer(command_buffer, &begin_info)
                .unwrap();

            device.cmd_set_scissor(command_buffer, 0, &[scissor]);
            device.cmd_set_viewport(command_buffer, 0, &[view_port]);

            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );

            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.ressources.materials[0].pipeline.layout,
                0,
                &[self.ressources.ubo_set],
                &[],
            );

            self.ressources.draw(device, command_buffer, scissor);

            device.cmd_end_render_pass(command_buffer);

            device.end_command_buffer(command_buffer).unwrap();
        };
    }

    pub fn recreate_swapchain(&mut self, new_size: PhysicalSize<u32>) {
        self.window_size = new_size;

        #[cfg(not(target_os = "android"))]
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.base.device_wait_idle();
        self.depth_image.destroy(&self.base.device);

        self.swapchain.update_caps(&self.base, new_size);
        let extend = self.swapchain.capabilities.current_extent;

        self.create_depth_resources(Extent3D {
            width: extend.width,
            height: extend.height,
            depth: 1,
        });

        self.swapchain
            .recreate(&self.base, self.render_pass, self.depth_image.view);
        self.update_uniform_buffer();

        self.ui.borrow_mut().resize(new_size.into());
    }

    pub fn update_uniform_buffer(&mut self) {
        let ubo = Matrix4::ortho(
            0.0,
            self.window_size.width as f32,
            0.0,
            self.window_size.height as f32,
            1000.0,
            -1000.0,
        );

        unsafe {
            for uniform_buffer in self.uniform_buffers_mapped {
                uniform_buffer.copy_from_nonoverlapping(&ubo, 1);
            }
        }
    }

    pub fn destroy(&mut self, render_assets: &mut RenderAssets) {
        let device = &self.base.device;
        unsafe {
            self.base.device_wait_idle();

            self.ressources.destroy(&self.base);
            render_assets.destroy(device);
            self.depth_image.destroy(device);
            self.swapchain.destroy(device);

            for &semaphore in &self.render_finsih_semaphores {
                device.destroy_semaphore(semaphore, None);
            }

            for i in 0..MFIF {
                device.destroy_semaphore(self.image_available_semaphores[i], None);
                device.destroy_fence(self.in_flight_fences[i], None);
            }

            device.destroy_command_pool(self.cmd_pool, None);
            device.destroy_render_pass(self.render_pass, None);
            self.base.destroy();
        };
    }

    pub fn destroy_ressources(&mut self, render_assets: &mut RenderAssets) {
        let device = &self.base.device;
        self.base.device_wait_idle();

        self.ressources.destroy(&self.base);
        render_assets.destroy(device);
    }
}
