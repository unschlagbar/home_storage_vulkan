#![allow(clippy::modulo_one)]
use ash::vk::{
    self, AccessFlags, Buffer, BufferUsageFlags, Extent3D, Format, ImageUsageFlags,
    MemoryPropertyFlags, PipelineStageFlags,
};
use iron_oxide::{
    graphics::{self, Material, SinlgeTimeCommands, Swapchain, TextureAtlas, VkBase},
    primitives::Matrix4,
    ui::{
        Ressources, Ui,
        materials::{AtlasInstance, FontInstance, ShadowInstance, UiInstance},
    },
};
use std::{
    cell::RefCell,
    io::Cursor,
    path::Path,
    ptr,
    rc::Rc,
    thread::sleep,
    time::{Duration, Instant},
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::app::{DEBUG_PERF, VSYNC};

// Max frames in flight
pub const MFIF: usize = 1;

pub struct VulkanRender {
    pub base: VkBase,

    pub window_size: PhysicalSize<u32>,
    pub swapchain: Swapchain,
    pub render_pass: vk::RenderPass,

    pub cmd_pool: vk::CommandPool,
    pub single_time_cmd_pool: vk::CommandPool,

    pub uniform_buffers: [Buffer; MFIF],
    uniform_buffers_mapped: [*mut Matrix4; MFIF],

    pub command_buffers: [vk::CommandBuffer; MFIF],

    image_available_semaphores: [vk::Semaphore; MFIF],
    render_finsih_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: [vk::Fence; MFIF],
    pub current_frame: usize,

    pub font_atlas: graphics::Image,

    pub depth_image: graphics::Image,

    pub ui: Rc<RefCell<Ui>>,
    pub ressources: Ressources,
}

impl VulkanRender {
    pub fn create(window: &Window, ui_state: Rc<RefCell<Ui>>) -> Self {
        let start_time = Instant::now();

        let (base, surface_loader, surface) =
            VkBase::create(0, vk::API_VERSION_1_2, c"Home Storage", window);

        let cmd_pool = Self::create_cmd_pool(&base);
        let single_time_cmd_pool = Self::create_single_time_cmd_pool(&base);

        let window_size = window.inner_size();

        let mut swapchain = Swapchain::create(
            &base,
            if VSYNC {
                vk::PresentModeKHR::FIFO
            } else {
                vk::PresentModeKHR::IMMEDIATE
            },
            surface_loader,
            surface,
        );
        let render_pass =
            Self::create_render_pass(&base, swapchain.format, true, true, false, true);

        let depth_image = Self::create_depth_resources(
            &base,
            cmd_pool,
            Extent3D {
                width: window_size.width,
                height: window_size.height,
                depth: 1,
            },
        );
        let cmd_buf = SinlgeTimeCommands::begin(&base, single_time_cmd_pool);
        let (mut font_atlas, mut staging_buf) = Self::create_font_atlas(&base, cmd_buf);
        SinlgeTimeCommands::end(&base, single_time_cmd_pool, cmd_buf);

        staging_buf.destroy(&base.device);

        swapchain.update_caps(&base, window_size);
        swapchain.recreate(&base, render_pass, depth_image.view);

        font_atlas.create_view(&base, vk::ImageAspectFlags::COLOR);

        let command_buffers = Self::create_command_buffers(&base.device, cmd_pool);
        let (image_available_semaphores, render_finsih_semaphores, in_flight_fences) =
            Self::create_sync_object(&base.device, swapchain.image_views.len());

        let mut texture_atlas = TextureAtlas::new((1024, 1024));
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/textures");
        texture_atlas.load_directory(path, &base, cmd_pool);

        let mut ressources = Ressources::new(&base, texture_atlas);

        let shadow_shaders = (
            include_bytes!("../spv/shadow.vert.spv").as_ref(),
            include_bytes!("../spv/shadow.frag.spv").as_ref(),
        );

        let base_shaders = (
            include_bytes!("../spv/basic.vert.spv").as_ref(),
            include_bytes!("../spv/basic.frag.spv").as_ref(),
        );

        let font_shaders = (
            include_bytes!("../spv/atlas_texture.vert.spv").as_ref(),
            include_bytes!("../spv/bitmap.frag.spv").as_ref(),
        );

        let atlas_shaders = (
            include_bytes!("../spv/atlas_texture.vert.spv").as_ref(),
            include_bytes!("../spv/atlas_texture.frag.spv").as_ref(),
        );
        let mut uniform_buffers = [vk::Buffer::null(); MFIF];
        let mut uniform_buffers_mapped = [ptr::null_mut(); MFIF];

        for (i, (buffer, buffer_mapped)) in uniform_buffers
            .iter_mut()
            .zip(&mut uniform_buffers_mapped)
            .enumerate()
        {
            let buffer_size;
            (*buffer, buffer_size) = ressources.buffer_manager.create_buffer(
                &base,
                size_of::<Matrix4>() as u64,
                BufferUsageFlags::UNIFORM_BUFFER,
            );

            let mem = &ressources.buffer_manager.memory_pool[0];
            *buffer_mapped = mem.get_ptr(buffer_size as usize * i) as _
        }

        {
            let ubo_layout = Ui::create_ubo_desc_layout(&base.device);
            let img_layout = Ui::create_img_desc_layout(&base.device);

            ressources.add_mat(Material::new::<UiInstance>(
                &base,
                window_size,
                render_pass,
                &[ubo_layout],
                false,
                base_shaders,
            ));

            ressources.add_mat(Material::new::<FontInstance>(
                &base,
                window_size,
                render_pass,
                &[ubo_layout, img_layout],
                false,
                font_shaders,
            ));

            ressources.add_mat(Material::new::<ShadowInstance>(
                &base,
                window_size,
                render_pass,
                &[ubo_layout],
                true,
                shadow_shaders,
            ));

            ressources.add_mat(Material::new::<AtlasInstance>(
                &base,
                window_size,
                render_pass,
                &[ubo_layout, img_layout],
                false,
                atlas_shaders,
            ));

            ressources.create_desc_sets(
                &base.device,
                &[ubo_layout, img_layout, img_layout],
                &[1, 3],
                uniform_buffers[0],
                font_atlas.view,
                ressources.texture_atlas.atlas.as_ref().unwrap().view,
            );
            unsafe {
                base.device.destroy_descriptor_set_layout(ubo_layout, None);
                base.device.destroy_descriptor_set_layout(img_layout, None);
            }
        }

        println!("Vulkan time: {:?}", start_time.elapsed());

        let mut renderer = Self {
            window_size,
            base,
            swapchain,
            render_pass,

            cmd_pool,
            single_time_cmd_pool,

            uniform_buffers,
            uniform_buffers_mapped,

            command_buffers,
            image_available_semaphores,
            render_finsih_semaphores,
            in_flight_fences,

            current_frame: 0,

            font_atlas,

            depth_image,

            ui: ui_state,
            ressources,
        };

        renderer.update_uniform_buffer();

        renderer
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

        self.depth_image = Self::create_depth_resources(
            &self.base,
            self.single_time_cmd_pool,
            Extent3D {
                width: extend.width,
                height: extend.height,
                depth: 1,
            },
        );

        self.swapchain
            .recreate(&self.base, self.render_pass, self.depth_image.view);
        self.update_uniform_buffer();

        self.ui.borrow_mut().resize(new_size.into());
    }

    fn create_render_pass(
        base: &VkBase,
        format: vk::SurfaceFormatKHR,
        clear: bool,
        depth: bool,
        has_previus: bool,
        is_final: bool,
    ) -> vk::RenderPass {
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

        let attachments = if depth {
            vec![color_attachment, depth_attachment]
        } else {
            vec![color_attachment]
        };

        let subpasses = [vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_ref as _,
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

        let render_pass_info = vk::RenderPassCreateInfo {
            attachment_count: attachments.len() as _,
            p_attachments: attachments.as_ptr(),
            subpass_count: subpasses.len() as _,
            p_subpasses: subpasses.as_ptr(),
            dependency_count: dependencies.len() as _,
            p_dependencies: dependencies.as_ptr(),
            ..Default::default()
        };

        unsafe {
            base.device
                .create_render_pass(&render_pass_info, None)
                .unwrap()
        }
    }

    fn create_cmd_pool(base: &VkBase) -> vk::CommandPool {
        let pool_info = vk::CommandPoolCreateInfo {
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER
                | vk::CommandPoolCreateFlags::TRANSIENT,
            queue_family_index: base.queue_family_index,
            ..Default::default()
        };

        unsafe { base.device.create_command_pool(&pool_info, None).unwrap() }
    }

    fn create_single_time_cmd_pool(base: &VkBase) -> vk::CommandPool {
        let pool_info = vk::CommandPoolCreateInfo {
            flags: vk::CommandPoolCreateFlags::TRANSIENT,
            queue_family_index: base.queue_family_index,
            ..Default::default()
        };

        unsafe { base.device.create_command_pool(&pool_info, None).unwrap() }
    }

    fn create_command_buffers(
        device: &ash::Device,
        cmd_pool: vk::CommandPool,
    ) -> [vk::CommandBuffer; MFIF] {
        let aloc_info = vk::CommandBufferAllocateInfo {
            command_pool: cmd_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: MFIF as u32,
            ..Default::default()
        };

        let vec = unsafe { device.allocate_command_buffers(&aloc_info).unwrap() };
        let mut buffers = [vk::CommandBuffer::null(); MFIF];

        buffers.copy_from_slice(&vec);

        buffers
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
                vk::Fence::null(),
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

    fn record_command_buffer(&mut self, index: u32, command_buffer: vk::CommandBuffer) {
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

    fn create_sync_object(
        device: &ash::Device,
        swap_chain_images: usize,
    ) -> ([vk::Semaphore; MFIF], Vec<vk::Semaphore>, [vk::Fence; MFIF]) {
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo {
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        let mut image_available_semaphores = [vk::Semaphore::null(); MFIF];
        let mut render_finsih_semaphores = vec![vk::Semaphore::null(); swap_chain_images];
        let mut in_flight_fences = [vk::Fence::null(); MFIF];

        unsafe {
            for i in 0..MFIF {
                image_available_semaphores[i] = device
                    .create_semaphore(&semaphore_info, None)
                    .unwrap_unchecked();
                in_flight_fences[i] = device.create_fence(&fence_info, None).unwrap_unchecked();
            }

            for semaphore in &mut render_finsih_semaphores {
                *semaphore = device
                    .create_semaphore(&semaphore_info, None)
                    .unwrap_unchecked();
            }
        }

        (
            image_available_semaphores,
            render_finsih_semaphores,
            in_flight_fences,
        )
    }

    fn update_uniform_buffer(&mut self) {
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

    fn create_font_atlas(
        base: &VkBase,
        cmd_buf: vk::CommandBuffer,
    ) -> (graphics::Image, graphics::Buffer) {
        let decoder = png::Decoder::new(Cursor::new(include_bytes!("../font/default8.png")));

        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut buf).unwrap();
        let width = info.width;
        let height = info.height;
        let image_size = buf.len() as u64;
        let extent = Extent3D {
            width,
            height,
            depth: 1,
        };

        let staging_buffer = graphics::Buffer::create(
            base,
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
        );

        let mapped_memory: *mut u8 = staging_buffer.map_memory(&base.device, image_size, 0);
        unsafe {
            mapped_memory.copy_from_nonoverlapping(buf.as_ptr(), image_size as usize);
        };
        staging_buffer.unmap_memory(&base.device);

        let mut texture_image = graphics::Image::create(
            base,
            extent,
            Format::R8_UNORM,
            vk::ImageTiling::OPTIMAL,
            ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::SAMPLED,
            MemoryPropertyFlags::DEVICE_LOCAL,
        );

        texture_image.trasition_layout(base, cmd_buf, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        texture_image.copy_from_buffer(
            base,
            cmd_buf,
            &staging_buffer,
            extent,
            vk::ImageAspectFlags::COLOR,
        );
        texture_image.trasition_layout(base, cmd_buf, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        (texture_image, staging_buffer)
    }

    fn create_depth_resources(
        base: &VkBase,
        cmd_pool: vk::CommandPool,
        extent: Extent3D,
    ) -> graphics::Image {
        let mut depth_image = graphics::Image::create(
            base,
            extent,
            Format::D16_UNORM,
            vk::ImageTiling::OPTIMAL,
            ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | ImageUsageFlags::TRANSIENT_ATTACHMENT,
            MemoryPropertyFlags::DEVICE_LOCAL
                | if cfg!(target_os = "android") {
                    MemoryPropertyFlags::LAZILY_ALLOCATED
                } else {
                    MemoryPropertyFlags::empty()
                },
        );
        let cmd_buf = SinlgeTimeCommands::begin(base, cmd_pool);
        depth_image.trasition_layout(
            base,
            cmd_buf,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        );
        SinlgeTimeCommands::end(base, cmd_pool, cmd_buf);
        depth_image.create_view(base, vk::ImageAspectFlags::DEPTH);
        depth_image
    }

    pub fn destroy(&mut self) {
        unsafe {
            let device = &self.base.device;
            device.device_wait_idle().unwrap();
            #[cfg(debug_assertions)]
            self.base
                .debug_utils
                .destroy_debug_utils_messenger(self.base.utils_messenger, None);

            for i in 0..self.swapchain.image_views.len() {
                device.destroy_semaphore(self.render_finsih_semaphores[i], None);
            }

            for i in 0..MFIF {
                device.destroy_semaphore(self.image_available_semaphores[i], None);
                device.destroy_fence(self.in_flight_fences[i], None);
            }

            self.ressources.destroy(&self.base);
            device.destroy_command_pool(self.cmd_pool, None);
            device.destroy_command_pool(self.single_time_cmd_pool, None);
            device.destroy_render_pass(self.render_pass, None);
            self.swapchain.destroy(device);
            self.depth_image.destroy(device);
            self.font_atlas.destroy(device);
            device.destroy_device(None);
            self.base.instance.destroy_instance(None);
        };
    }
}
