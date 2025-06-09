use std::{
    cell::RefCell,
    ffi::c_void,
    mem::size_of,
    ptr::{self, null},
    rc::Rc,
    thread::sleep,
    time::{Duration, Instant}
};
use ash::vk::{
    self, AccessFlags,
    CompareOp, Extent3D,
    Format, ImageUsageFlags,
    MemoryPropertyFlags,
    PipelineStageFlags,
    ShaderStageFlags
};
use cgmath::{ortho, Matrix4};
use iron_oxide::{graphics::{self, Buffer, SinlgeTimeCommands, VkBase}, ui::UiState};
use winit::{dpi::PhysicalSize, raw_window_handle::{HasDisplayHandle, HasWindowHandle}, window::Window};

use super::buffer::create_uniform_buffers;
use super::UniformBufferObject;
use super::main_pipeline;
use crate::{game::{app::FPS_LIMIT, Cube, World}, graphics::Vertex};

pub const MAXFRAMESINFLIGHT: usize = 1;

pub struct VulkanRender {
    pub base: iron_oxide::graphics::VkBase,
    
    pub window_size: PhysicalSize<u32>,
    pub swapchain: super::Swapchain,
    pub render_pass: vk::RenderPass,

    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,

    pub command_pool: vk::CommandPool,
    pub single_time_command_pool: vk::CommandPool,

    #[allow(unused)]
    pub vertex_count: u32,
    pub vertex_buffer: Buffer,

    pub index_count: u32,
    pub index_buffer: Buffer,

    pub instance_count: u32,
    pub instance_buffer: Buffer,
    pub staging_buffer: Buffer,

    uniform_buffers: [Buffer; MAXFRAMESINFLIGHT],
    uniform_buffers_mapped: [*mut c_void; MAXFRAMESINFLIGHT],

    ui_uniform_buffers: [Buffer; MAXFRAMESINFLIGHT],
    ui_uniform_buffers_mapped: [*mut c_void; MAXFRAMESINFLIGHT],

    descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    ui_descriptor_pool: vk::DescriptorPool,
    pub ui_descriptor_sets: Vec<vk::DescriptorSet>,
    pub ui_descriptor_set_layout: vk::DescriptorSetLayout,

    pub command_buffers: [vk::CommandBuffer; MAXFRAMESINFLIGHT],

    image_available_semaphores: [vk::Semaphore; MAXFRAMESINFLIGHT],
    render_finsih_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: [vk::Fence; MAXFRAMESINFLIGHT],
    pub current_frame: usize,

    texture_image: graphics::Image,
    pub texture_sampler: vk::Sampler,

    font_atlas: graphics::Image,

    pub depth_image: graphics::Image,

    pub ui_state: Rc<RefCell<UiState>>,
    world: *const World,
}

impl VulkanRender {
    pub fn create(window: &Window, world: &World) -> Self {
        let start_time = Instant::now();

        let display_handle = window.display_handle().unwrap().as_raw();
        let window_handle = window.window_handle().unwrap().as_raw();

        let (base, surface_loader, surface) = VkBase::create(Vec::new(), 0, display_handle, window_handle);

        let command_pool = Self::create_command_pool(&base);
        let single_time_command_pool = Self::create_single_time_command_pool(&base);

        let window_size = window.inner_size();
        let mut swapchain = super::Swapchain::create(&base, window_size, if FPS_LIMIT {vk::PresentModeKHR::FIFO} else {vk::PresentModeKHR::IMMEDIATE}, surface_loader, surface);
        let render_pass = Self::create_render_pass(&base, swapchain.format, true, true, false, true);

        let (vertices, indices) = Cube::generate_vertices();
        let instances = world.get_instances();
        
        let vertex_count = vertices.len() as u32;
        let index_count = indices.len() as u32;
        
        
        let (vertex_buffer, index_buffer, instance_buffer) = (
            Buffer::create(&base, vertices.len() as u64 * size_of::<Vertex>() as u64, vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST, vk::MemoryPropertyFlags::DEVICE_LOCAL),
            Buffer::create(&base, vertices.len() as u64 * size_of::<u32>() as u64, vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST, vk::MemoryPropertyFlags::DEVICE_LOCAL),
            Buffer::create(&base, vertices.len() as u64 * size_of::<Matrix4<f32>>() as u64, vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST, vk::MemoryPropertyFlags::DEVICE_LOCAL),
        );

        let staging_size = vertex_buffer.size + index_buffer.size + instance_buffer.size;
        let staging_buffer = Buffer::create(&base, staging_size, vk::BufferUsageFlags::TRANSFER_SRC, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

        let mapped_memory = staging_buffer.map_memory(&base.device, staging_size, 0);
        unsafe {
            std::ptr::copy_nonoverlapping(vertices.as_ptr(), mapped_memory as _, vertices.len());
            std::ptr::copy_nonoverlapping(indices.as_ptr(), mapped_memory.byte_add(vertex_buffer.size as _) as _, indices.len());
            std::ptr::copy_nonoverlapping(instances.as_ptr(), mapped_memory.byte_add(vertex_buffer.size as usize + index_buffer.size as usize) as _, instances.len());
        };
        staging_buffer.unmap_memory(&base.device);

        let cmd_buf = SinlgeTimeCommands::begin(&base, single_time_command_pool);
        staging_buffer.copy(&base, &vertex_buffer, vertex_buffer.size, 0, cmd_buf);
        staging_buffer.copy(&base, &index_buffer, index_buffer.size, vertex_buffer.size, cmd_buf);
        staging_buffer.copy(&base, &instance_buffer, instance_buffer.size, vertex_buffer.size + index_buffer.size, cmd_buf);

        let depth_image = Self::create_depth_resources(&base, cmd_buf, Extent3D { width: window_size.width, height: window_size.height, depth: 1 });
        let (mut texture_image, staging_buf) = Self::create_texture_image(&base, cmd_buf);
        let (mut font_atlas, staging_buf2) = Self::create_font_atlas(&base, cmd_buf);
        SinlgeTimeCommands::end(&base, single_time_command_pool, cmd_buf);
        
        staging_buf.destroy(&base.device);
        staging_buf2.destroy(&base.device);
        
        swapchain.create_framebuffer(&base, render_pass, depth_image.view, window_size);
        
        
        
        let ui_state = world.ui.clone();
        
        
        texture_image.create_view(&base, vk::ImageAspectFlags::COLOR);
        font_atlas.create_view(&base, vk::ImageAspectFlags::COLOR);
        
        let (uniform_buffers, uniform_buffers_mapped) = create_uniform_buffers(&base);
        let (ui_uniform_buffers, ui_uniform_buffers_mapped) = create_uniform_buffers(&base);
        
        let texture_sampler = Self::create_texture_sampler(&base.device);
        let descriptor_pool = create_descriptor_pool(&base.device);
        let ui_descriptor_pool = create_ui_descriptor_pool(&base.device);
        let descriptor_set_layout = create_descriptor_set_layout(&base.device);
        let ui_descriptor_set_layout = create_ui_descriptor_set_layout(&base.device);
        let (pipeline_layout, pipeline) = main_pipeline::create_main_pipeline(&base.device, window_size, render_pass, descriptor_set_layout);
        let descriptor_sets = create_descriptor_sets(&base.device, descriptor_pool, descriptor_set_layout, &uniform_buffers, texture_sampler, texture_image.view, size_of::<UniformBufferObject>() as _);
        let ui_descriptor_sets = create_ui_descriptor_sets(&base.device, ui_descriptor_pool, ui_descriptor_set_layout, &ui_uniform_buffers, texture_sampler, &[font_atlas.view, texture_image.view], size_of::<UniformBufferObject>() as _);
        
        unsafe { base.device.destroy_descriptor_set_layout(descriptor_set_layout, None) };
        
        let command_buffers = Self::create_command_buffers(&base.device, command_pool);
        let (image_available_semaphores, render_finsih_semaphores, in_flight_fences)= Self::create_sync_object(&base.device, swapchain.image_views.len());
        
        let world = world as *const World;
        
        println!("Vulkan time: {:?}", start_time.elapsed());
        
        let mut renderer = Self {
            window_size,
            base,
            swapchain,
            pipeline_layout,
            render_pass,
            graphics_pipeline: pipeline,

            command_pool,
            single_time_command_pool,
    
            vertex_count,
            vertex_buffer,

            index_count,
            index_buffer,

            instance_count: instances.len() as _,
            instance_buffer,
            staging_buffer,
    
            uniform_buffers,
            uniform_buffers_mapped,
            ui_uniform_buffers,
            ui_uniform_buffers_mapped,
    
            descriptor_pool,
            ui_descriptor_pool,
            descriptor_sets,
            ui_descriptor_sets,
            ui_descriptor_set_layout,
    
            command_buffers,
            image_available_semaphores,
            render_finsih_semaphores,
            in_flight_fences,
    
            current_frame: 0,
            texture_image,
    
            font_atlas,
    
            texture_sampler,
            depth_image,

            ui_state,
            world,
        };

        renderer.update_ui_uniform_buffer();

        renderer
    }

    pub fn recreate_swapchain(&mut self, new_size: PhysicalSize<u32>) {
        self.window_size = new_size;

        #[cfg(not(target_os = "android"))]
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        unsafe { self.base.device.device_wait_idle().unwrap_unchecked() };
        self.depth_image.destroy(&self.base.device);

        let cmd_buf = SinlgeTimeCommands::begin(&self.base, self.single_time_command_pool);
        self.depth_image = Self::create_depth_resources(&self.base, cmd_buf, Extent3D { width: self.window_size.width, height: self.window_size.height, depth: 1 });
        SinlgeTimeCommands::submit(&self.base, cmd_buf);

        self.swapchain.recreate(&self.base, new_size, self.render_pass, self.depth_image.view);
        self.update_ui_uniform_buffer();

        self.ui_state.borrow_mut().resize(new_size.into());

        SinlgeTimeCommands::end_after_submit(&self.base, self.single_time_command_pool, cmd_buf);
    }

    fn create_render_pass(base: &VkBase, format: vk::SurfaceFormatKHR, clear: bool, depth: bool, has_previus: bool, is_final: bool) -> vk::RenderPass {
        let color_attachment = vk::AttachmentDescription {
            format: format.format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: if clear {vk::AttachmentLoadOp::CLEAR} else { vk::AttachmentLoadOp::DONT_CARE },
            store_op: if is_final {vk::AttachmentStoreOp::STORE} else {vk::AttachmentStoreOp::DONT_CARE},
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: if has_previus {vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL} else { vk::ImageLayout::UNDEFINED },
            final_layout: if is_final {vk::ImageLayout::PRESENT_SRC_KHR} else { vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL },
            flags: vk::AttachmentDescriptionFlags::empty()
        };

        let depth_attachment = vk::AttachmentDescription {
            format: Format::D24_UNORM_S8_UINT,
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

        let subpasses = [
            vk::SubpassDescription {
                pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
                color_attachment_count: 1,
                p_color_attachments: &color_attachment_ref as _,
                p_depth_stencil_attachment: if depth { &depth_attachment_ref } else { null() },
                ..Default::default()
            },
            vk::SubpassDescription {
                pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
                color_attachment_count: 1,
                p_color_attachments: &color_attachment_ref as _,
                p_depth_stencil_attachment: if depth { &depth_attachment_ref } else { null() },
                ..Default::default()
            },
        ];

        let dependencies = [
            vk::SubpassDependency {
                src_subpass: vk::SUBPASS_EXTERNAL,
                dst_subpass: 0,
                src_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                src_access_mask: vk::AccessFlags::empty(),
                dst_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | PipelineStageFlags::LATE_FRAGMENT_TESTS,
                dst_access_mask: if depth {AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE | AccessFlags::COLOR_ATTACHMENT_WRITE} else {AccessFlags::COLOR_ATTACHMENT_WRITE},
                dependency_flags: vk::DependencyFlags::empty(),
            },
            vk::SubpassDependency {
                src_subpass: 0,
                dst_subpass: 1,
                src_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                src_access_mask: if depth {AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE | AccessFlags::COLOR_ATTACHMENT_WRITE} else {AccessFlags::COLOR_ATTACHMENT_WRITE},
                dst_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | PipelineStageFlags::LATE_FRAGMENT_TESTS,
                dst_access_mask: if depth {AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE | AccessFlags::COLOR_ATTACHMENT_WRITE} else {AccessFlags::COLOR_ATTACHMENT_WRITE},
                dependency_flags: vk::DependencyFlags::empty(),
            }
        ];

        let render_pass_info = vk::RenderPassCreateInfo {
            attachment_count: attachments.len() as _,
            p_attachments: attachments.as_ptr(),
            subpass_count: subpasses.len() as _,
            p_subpasses: subpasses.as_ptr(),
            dependency_count: dependencies.len() as _,
            p_dependencies: dependencies.as_ptr(),
            ..Default::default()
        };

        unsafe { base.device.create_render_pass(&render_pass_info, None).unwrap() }

    }

    fn create_command_pool(base: &VkBase) -> vk::CommandPool {
        let pool_info = vk::CommandPoolCreateInfo {
            flags: vk::CommandPoolCreateFlags::TRANSIENT,
            queue_family_index: base.queue_family_index,
            ..Default::default()
        };

        unsafe { base.device.create_command_pool(&pool_info, None).unwrap() }
    }

    fn create_single_time_command_pool(base: &VkBase) -> vk::CommandPool {
        let pool_info = vk::CommandPoolCreateInfo {
            flags: vk::CommandPoolCreateFlags::TRANSIENT,
            queue_family_index: base.queue_family_index,
            ..Default::default()
        };

        unsafe { base.device.create_command_pool(&pool_info, None).unwrap() }
    }

    fn create_command_buffers(device: &ash::Device, command_pool: vk::CommandPool) -> [vk::CommandBuffer; MAXFRAMESINFLIGHT] {
        let aloc_info = vk::CommandBufferAllocateInfo {
            command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: MAXFRAMESINFLIGHT as _,
            ..Default::default()
        };

        let vec = unsafe { device.allocate_command_buffers(&aloc_info).unwrap() };
        let mut buffers = [vk::CommandBuffer::null(); MAXFRAMESINFLIGHT];

        for i in 0..MAXFRAMESINFLIGHT {
            buffers[i] = vec[i];
        }

        buffers
    }

    pub fn draw_frame(&mut self) {
        if self.window_size.width == 0 || self.window_size.height == 0 {
            sleep(Duration::from_millis(100));
            return;
        }

        unsafe {
            self.base.device.wait_for_fences(&[self.in_flight_fences[self.current_frame]], true, u64::MAX).unwrap();
            self.base.device.reset_fences(&[self.in_flight_fences[self.current_frame]]).unwrap();
            self.base.device.reset_command_pool(self.command_pool, vk::CommandPoolResetFlags::empty()).unwrap();
        };

        let image_index = unsafe { 
            match self.swapchain.loader.acquire_next_image(self.swapchain.inner, u64::MAX, self.image_available_semaphores[self.current_frame], vk::Fence::null()) {
                Ok(result) => {
                    if result.1 {
                        return;
                    }
                    result.0
                }, 
                Err(_) => return
            }
        };


        self.update_ui();

        self.record_command_buffer(image_index);
        self.update_uniform_buffer();

        let submit_info = vk::SubmitInfo {
            p_wait_semaphores: &self.image_available_semaphores[self.current_frame],
            wait_semaphore_count: 1,
            p_wait_dst_stage_mask: &PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            command_buffer_count: 1,
            p_command_buffers: &self.command_buffers[self.current_frame],
            signal_semaphore_count: 1,
            p_signal_semaphores: &self.render_finsih_semaphores[image_index as usize],
            ..Default::default()
        };

        if self.base.queue_submit(&[submit_info], self.in_flight_fences[self.current_frame]).is_err() {
            return;
        }

        let present_info = vk::PresentInfoKHR {
            wait_semaphore_count: 1,
            p_wait_semaphores: &self.render_finsih_semaphores[image_index as usize],
            swapchain_count: 1,
            p_swapchains: &self.swapchain.inner,
            p_image_indices: &image_index,
            ..Default::default()
        };

        if unsafe { self.swapchain.loader.queue_present(self.base.queue, &present_info).is_err() } {
            return;
        }

        self.current_frame = (self.current_frame + 1) % MAXFRAMESINFLIGHT;
    }

    fn record_command_buffer(&mut self, index: u32) {
        let clear_values = [
            vk::ClearValue { color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 0.0] } },
            vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } },
        ];

        let render_pass_info = vk::RenderPassBeginInfo {
            render_pass: self.render_pass,
            framebuffer: self.swapchain.framebuffers[index as usize],
            render_area: vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D { width: self.window_size.width, height: self.window_size.height }},
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
            max_depth: 1.0
        };
        
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D { width: self.window_size.width, height: self.window_size.height },
        };

        let device = &self.base.device;
        
        let begin_info = vk::CommandBufferBeginInfo {
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            ..Default::default()
        };
        
        unsafe {
            device.begin_command_buffer(self.command_buffers[self.current_frame], &begin_info).unwrap();
            
            device.cmd_set_scissor(self.command_buffers[self.current_frame], 0, &[scissor]);
            device.cmd_set_viewport(self.command_buffers[self.current_frame], 0, &[view_port]);
            
            device.cmd_begin_render_pass(self.command_buffers[self.current_frame], &render_pass_info, vk::SubpassContents::INLINE);
            device.cmd_bind_pipeline(self.command_buffers[self.current_frame], vk::PipelineBindPoint::GRAPHICS, self.graphics_pipeline);
            device.cmd_bind_vertex_buffers(self.command_buffers[self.current_frame], 0, &[self.vertex_buffer.inner, self.instance_buffer.inner], &[0, 0]);
            device.cmd_bind_index_buffer(self.command_buffers[self.current_frame], self.index_buffer.inner, 0, vk::IndexType::UINT32);
            device.cmd_bind_descriptor_sets(self.command_buffers[self.current_frame], vk::PipelineBindPoint::GRAPHICS, self.pipeline_layout, 0, &[self.descriptor_sets[self.current_frame]], &[]);
            device.cmd_draw_indexed(self.command_buffers[self.current_frame], self.index_count, self.instance_count, 0, 0, 0);
            device.cmd_next_subpass(self.command_buffers[self.current_frame], vk::SubpassContents::INLINE);

            self.ui_state.borrow().draw(&self.base.device, self.command_buffers[self.current_frame], self.ui_descriptor_sets[self.current_frame]);
            device.cmd_end_render_pass(self.command_buffers[self.current_frame]);
            
            device.end_command_buffer(self.command_buffers[self.current_frame]).unwrap();
        };
    }

    fn create_sync_object(device: &ash::Device, swap_chain_images: usize) -> ([vk::Semaphore; MAXFRAMESINFLIGHT], Vec<vk::Semaphore>, [vk::Fence; MAXFRAMESINFLIGHT]) {
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo {
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        let mut image_available_semaphores = [vk::Semaphore::null(); MAXFRAMESINFLIGHT];
        let mut render_finsih_semaphores = vec![vk::Semaphore::null(); swap_chain_images];
        let mut in_flight_fences = [vk::Fence::null(); MAXFRAMESINFLIGHT];

        unsafe {
            for i in 0..MAXFRAMESINFLIGHT {
                image_available_semaphores[i] = device.create_semaphore(&semaphore_info, None).unwrap_unchecked();
                in_flight_fences[i] = device.create_fence(&fence_info, None).unwrap_unchecked();
            }
            
            for i in 0..swap_chain_images {
                render_finsih_semaphores[i] = device.create_semaphore(&semaphore_info, None).unwrap_unchecked();
            }
        }

        (image_available_semaphores, render_finsih_semaphores, in_flight_fences)

    }

    #[inline]
    fn update_uniform_buffer(&mut self) {
        let world = unsafe { &mut *(self.world as *mut World) }; 

        if !world.camera.moved {
            return;
        }

        let view = world.camera.view();
        let proj = world.camera.projection(self.window_size.width as f32 / self.window_size.height as f32);

        let ubo = proj * view;

        for uniform_buffer in self.uniform_buffers_mapped {
            unsafe { ptr::copy_nonoverlapping(&ubo as _, uniform_buffer as _, 1) };
        }
    }

    fn update_ui_uniform_buffer(&mut self) {
        let ubo: Matrix4<f32> = ortho(0.0, self.window_size.width as _, 0.0, self.window_size.height as _, -100.0, 100.0);

        for uniform_buffer in self.ui_uniform_buffers_mapped {
            unsafe { ptr::copy_nonoverlapping(&ubo as _, uniform_buffer as _, 1) };
        }
    }

    fn create_texture_image(base: &VkBase, cmd_buf: vk::CommandBuffer) -> (graphics::Image, Buffer) {
        let decoder = png::Decoder::new(&include_bytes!("../../textures/texture.png")[..]);

        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        let width = info.width;
        let height = info.height;
        let image_size = buf.len() as u64;
        let extent = Extent3D { width, height, depth: 1 };
        
        let staging_buffer = Buffer::create(base, image_size, vk::BufferUsageFlags::TRANSFER_SRC, MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT);

        let mapped_memory = staging_buffer.map_memory(&base.device, image_size, 0);
        unsafe { 
            ptr::copy_nonoverlapping(buf.as_ptr(), mapped_memory as _, image_size as usize);
        };
        staging_buffer.unmap_memory(&base.device);

        let mut texture_image = graphics::Image::create(base, extent, Format::R8G8B8A8_SRGB, vk::ImageTiling::OPTIMAL, ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::SAMPLED, MemoryPropertyFlags::DEVICE_LOCAL);

        texture_image.trasition_layout(base, cmd_buf, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        texture_image.copy_from_buffer(base, cmd_buf, &staging_buffer, extent, vk::ImageAspectFlags::COLOR);
        texture_image.trasition_layout(base, cmd_buf, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        (texture_image, staging_buffer)
    }

    fn create_font_atlas(base: &VkBase, cmd_buf: vk::CommandBuffer) -> (graphics::Image, Buffer) {
        let decoder = png::Decoder::new(&include_bytes!("../../font/default8.png")[..]);

        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        let width = info.width;
        let height = info.height;
        let image_size = height as u64 * width as u64;
        let extent = Extent3D { width, height, depth: 1 };
        
        let staging_buffer = Buffer::create(base, image_size, vk::BufferUsageFlags::TRANSFER_SRC, MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT);

        let mapped_memory = staging_buffer.map_memory(&base.device, image_size, 0);
        unsafe { 
            ptr::copy_nonoverlapping(buf.as_ptr(), mapped_memory as _, image_size as usize);
        };
        staging_buffer.unmap_memory(&base.device);

        let mut texture_image = graphics::Image::create(base, extent, Format::R8_UNORM, vk::ImageTiling::OPTIMAL, ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::SAMPLED, MemoryPropertyFlags::DEVICE_LOCAL);

        texture_image.trasition_layout(base, cmd_buf, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        texture_image.copy_from_buffer(base, cmd_buf, &staging_buffer, extent, vk::ImageAspectFlags::COLOR);
        texture_image.trasition_layout(base, cmd_buf, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        (texture_image, staging_buffer)
    }

    fn create_texture_sampler(device: &ash::Device) -> vk::Sampler {
        let create_info = vk::SamplerCreateInfo {
            mag_filter: vk::Filter::NEAREST,
            min_filter: vk::Filter::NEAREST,
            mipmap_mode: vk::SamplerMipmapMode::NEAREST,
            address_mode_u: vk::SamplerAddressMode::CLAMP_TO_BORDER,
            address_mode_v: vk::SamplerAddressMode::CLAMP_TO_BORDER,
            address_mode_w: vk::SamplerAddressMode::CLAMP_TO_BORDER,
            mip_lod_bias: 0.0,
            anisotropy_enable: vk::FALSE,
            max_anisotropy: 0.0,
            compare_enable: vk::FALSE,
            compare_op: CompareOp::ALWAYS,
            min_lod: 0.0,
            max_lod: vk::LOD_CLAMP_NONE,
            border_color: vk::BorderColor::FLOAT_TRANSPARENT_BLACK,
            unnormalized_coordinates: vk::FALSE,
            ..Default::default()
        };

        unsafe { device.create_sampler(&create_info, None).unwrap() }
    }

    fn create_depth_resources(base: &VkBase, cmd_buf: vk::CommandBuffer, extent: Extent3D) -> graphics::Image {
        let mut depth_image = graphics::Image::create(base, extent, Format::D24_UNORM_S8_UINT, vk::ImageTiling::OPTIMAL, ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT, MemoryPropertyFlags::DEVICE_LOCAL);
        depth_image.trasition_layout(base, cmd_buf, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
        depth_image.create_view(base, vk::ImageAspectFlags::DEPTH);
        depth_image
    }

    pub fn update_ui(&mut self) {
        self.ui_state.borrow_mut().update(&self.base, self.single_time_command_pool);
    }

    pub fn destroy(&mut self) {
        unsafe {
            let device = &self.base.device;
            #[cfg(debug_assertions)]
            self.base.debug_utils.destroy_debug_utils_messenger(self.base.utils_messenger, None);

            for i in 0..self.swapchain.image_views.len() {
                device.destroy_semaphore(self.render_finsih_semaphores[i], None);
            }

            for i in 0..MAXFRAMESINFLIGHT {
                device.destroy_semaphore(self.image_available_semaphores[i], None);
                device.destroy_fence(self.in_flight_fences[i], None);
                self.uniform_buffers[i].destroy(device);
                self.ui_uniform_buffers[i].destroy(device);
            }

            self.ui_state.borrow().destroy(device);
            device.destroy_descriptor_set_layout(self.ui_descriptor_set_layout, None);
            device.destroy_command_pool(self.command_pool, None);
            device.destroy_command_pool(self.single_time_command_pool, None);
            device.destroy_pipeline(self.graphics_pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
            device.destroy_descriptor_pool(self.descriptor_pool, None);
            device.destroy_descriptor_pool(self.ui_descriptor_pool, None);
            device.destroy_render_pass(self.render_pass, None);
            self.swapchain.destroy(device);
            device.destroy_sampler(self.texture_sampler, None);
            self.depth_image.destroy(device);
            self.texture_image.destroy(device);
            self.font_atlas.destroy(device);
            self.vertex_buffer.destroy(device);
            self.index_buffer.destroy(device);
            self.instance_buffer.destroy(device);
            self.staging_buffer.destroy(device);
            device.destroy_device(None);
            self.base.instance.destroy_instance(None);
        };
    }

}

fn create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
    let ubo_layout_binding = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_count: 1,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        stage_flags: ShaderStageFlags::VERTEX,
        p_immutable_samplers: null(),
        _marker: std::marker::PhantomData,
    };

    let sampler_layout_binding = vk::DescriptorSetLayoutBinding {
        binding: 1,
        descriptor_count: 1,
        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        stage_flags: ShaderStageFlags::FRAGMENT,
        p_immutable_samplers: null(),
        _marker: std::marker::PhantomData,
    };

    let bindings = [ubo_layout_binding, sampler_layout_binding];

    let layout_info = vk::DescriptorSetLayoutCreateInfo {
        binding_count: bindings.len() as _,
        p_bindings: bindings.as_ptr(),
        ..Default::default()
    };

    unsafe { device.create_descriptor_set_layout(&layout_info, None).unwrap() }
}

fn create_ui_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
    let ubo_layout_binding = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_count: 1,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        stage_flags: ShaderStageFlags::VERTEX,
        ..Default::default()
    };

    let sampler_layout_binding = vk::DescriptorSetLayoutBinding {
        binding: 1,
        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: 2,
        stage_flags: ShaderStageFlags::FRAGMENT,
        ..Default::default()
    };

    let bindings = [ubo_layout_binding, sampler_layout_binding];

    let layout_info = vk::DescriptorSetLayoutCreateInfo {
        binding_count: bindings.len() as _,
        p_bindings: bindings.as_ptr(),
        ..Default::default()
    };

    unsafe { device.create_descriptor_set_layout(&layout_info, None).unwrap() }
}

fn create_descriptor_pool(device: &ash::Device) -> vk::DescriptorPool {
    let pool_sizes = [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: MAXFRAMESINFLIGHT as _,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: MAXFRAMESINFLIGHT as _,
        }
    ];

    let pool_info = vk::DescriptorPoolCreateInfo {
        pool_size_count: pool_sizes.len() as _,
        p_pool_sizes: pool_sizes.as_ptr(),
        max_sets: MAXFRAMESINFLIGHT as _,
        ..Default::default()
    };

    unsafe { device.create_descriptor_pool(&pool_info, None).unwrap() }
}

fn create_ui_descriptor_pool(device: &ash::Device) -> vk::DescriptorPool {
    let pool_sizes = [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: MAXFRAMESINFLIGHT as _,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: MAXFRAMESINFLIGHT as u32 * 2,
        }
    ];

    let pool_info = vk::DescriptorPoolCreateInfo {
        pool_size_count: pool_sizes.len() as _,
        p_pool_sizes: pool_sizes.as_ptr(),
        max_sets: MAXFRAMESINFLIGHT as _,
        ..Default::default()
    };

    unsafe { device.create_descriptor_pool(&pool_info, None).unwrap() }
}

fn create_descriptor_sets(
    device: &ash::Device,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    uniform_buffers: &[Buffer],
    textures_sampler: vk::Sampler,
    texture_image_view: vk::ImageView,
    ubo_size: u64,
) -> Vec<vk::DescriptorSet> {
    let layouts: [vk::DescriptorSetLayout; MAXFRAMESINFLIGHT] = [descriptor_set_layout; MAXFRAMESINFLIGHT];

    let allocate_info = vk::DescriptorSetAllocateInfo {
        descriptor_pool,
        descriptor_set_count: MAXFRAMESINFLIGHT as u32,
        p_set_layouts: layouts.as_ptr(),
        ..Default::default()
    };

    let descriptor_sets = unsafe { device.allocate_descriptor_sets(&allocate_info).unwrap() };

    for i in 0..MAXFRAMESINFLIGHT {
        let buffer_info = vk::DescriptorBufferInfo {
            buffer: uniform_buffers[i].inner,
            offset: 0,
            range: ubo_size,
        };

        let image_info = vk::DescriptorImageInfo {
            sampler: textures_sampler,
            image_view: texture_image_view,
            image_layout: vk::ImageLayout::GENERAL,
        };

        let descriptor_writes = [
            vk::WriteDescriptorSet {
                dst_set: descriptor_sets[i],
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                p_buffer_info: &buffer_info,
                ..Default::default()
            },
            vk::WriteDescriptorSet {
                dst_set: descriptor_sets[i],
                dst_binding: 1,
                dst_array_element: 0,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                p_image_info: &image_info,
                ..Default::default()
            }
        ];

        unsafe { device.update_descriptor_sets(&descriptor_writes, &[]) };
    }

    descriptor_sets
}

fn create_ui_descriptor_sets(
    device: &ash::Device,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    uniform_buffers: &[Buffer],
    textures_sampler: vk::Sampler,
    texture_image_views: &[vk::ImageView],
    ubo_size: u64,
) -> Vec<vk::DescriptorSet> {
    let layouts: [vk::DescriptorSetLayout; MAXFRAMESINFLIGHT] = [descriptor_set_layout; MAXFRAMESINFLIGHT];

    let allocate_info = vk::DescriptorSetAllocateInfo {
        descriptor_pool,
        descriptor_set_count: MAXFRAMESINFLIGHT as u32,
        p_set_layouts: layouts.as_ptr(),
        ..Default::default()
    };

    let descriptor_sets = unsafe { device.allocate_descriptor_sets(&allocate_info).unwrap() };

    for i in 0..MAXFRAMESINFLIGHT {
        let buffer_info = vk::DescriptorBufferInfo {
            buffer: uniform_buffers[i].inner,
            offset: 0,
            range: ubo_size,
        };

        let mut image_infos = Vec::with_capacity(texture_image_views.len());

        for image_view in texture_image_views {
            image_infos.push(
                vk::DescriptorImageInfo {
                    sampler: textures_sampler,
                    image_view: *image_view,
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                }
            );
        }

        let descriptor_writes = [
            vk::WriteDescriptorSet {
                dst_set: descriptor_sets[i],
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                p_buffer_info: &buffer_info,
                ..Default::default()
            },
            vk::WriteDescriptorSet {
                dst_set: descriptor_sets[i],
                dst_binding: 1,
                dst_array_element: 0,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: image_infos.len() as _,
                p_image_info: image_infos.as_ptr(),
                ..Default::default()
            }
        ];

        unsafe { device.update_descriptor_sets(&descriptor_writes, &[]) };
    }

    descriptor_sets
}