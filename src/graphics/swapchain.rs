use ash::{khr::{swapchain, surface}, vk::{self, Format, Framebuffer, ImageView, PresentModeKHR, RenderPass, SurfaceCapabilitiesKHR, SurfaceFormatKHR, SurfaceKHR, SurfaceTransformFlagsKHR, SwapchainKHR}, Device};
use iron_oxide::graphics::VkBase;
use winit::dpi::PhysicalSize;

pub struct Swapchain {
    pub loader: swapchain::Device,
    pub inner: SwapchainKHR,
    pub surface_loader: surface::Instance,
    pub surface: SurfaceKHR,
    pub image_views: Vec<ImageView>,
    pub capabilities: SurfaceCapabilitiesKHR,
    pub format: SurfaceFormatKHR,
    pub present_mode: PresentModeKHR,
    pub composite_alpha: vk::CompositeAlphaFlagsKHR,
    pub framebuffers: Vec<vk::Framebuffer>,
}

impl Swapchain {
    pub fn create(base: &VkBase, window_size: PhysicalSize<u32>, present_mode: vk::PresentModeKHR, surface_loader: surface::Instance, surface: SurfaceKHR) -> Self {
        let loader = swapchain::Device::new(&base.instance, &base.device);
        let (capabilities, format, present_mode) = Self::query_swap_chain_support(base, present_mode, &surface_loader, surface);
        let composite_alpha = if capabilities.supported_composite_alpha.contains(vk::CompositeAlphaFlagsKHR::OPAQUE) {
            vk::CompositeAlphaFlagsKHR::OPAQUE
        } else {
            vk::CompositeAlphaFlagsKHR::INHERIT
        };
        let swapchain = Self::create_swap_chain(window_size, surface, &loader, &capabilities, composite_alpha, format, present_mode, base.queue_family_index);
        let image_views = Self::create_image_views(&loader, swapchain, &base.device, format.format);

        let framebuffers = vec![Framebuffer::null(); image_views.len()];

        Self {
            loader,
            inner: swapchain,
            surface_loader,
            surface,
            image_views,
            capabilities,
            format,
            present_mode,
            composite_alpha,
            framebuffers,
        }
    }

    pub fn create_framebuffer(&mut self, base: &VkBase, render_pass: RenderPass, attachment: ImageView, window_size: PhysicalSize<u32>) {
        for i in 0..self.image_views.len() {
            let attachments = [self.image_views[i], attachment];
            let main_create_info = vk::FramebufferCreateInfo {
                render_pass,
                attachment_count: attachments.len() as _,
                p_attachments: attachments.as_ptr(),
                width: window_size.width,
                height: window_size.height,
                layers: 1,
                ..Default::default()
            };

            self.framebuffers[i] = unsafe { base.device.create_framebuffer(&main_create_info, None).unwrap() };
        }
    }

    pub fn recreate(&mut self, base: &VkBase, window_size: PhysicalSize<u32>, render_pass: RenderPass, attachment: ImageView) {
        println!("recreate");
        unsafe  {
            self.capabilities = self.surface_loader.get_physical_device_surface_capabilities(base.physical_device, self.surface).unwrap();
        }

        let image_extent = if self.capabilities.current_extent.width != u32::MAX {
            self.capabilities.current_extent
        } else {
            vk::Extent2D { width: window_size.width, height: window_size.height }
        };

        let create_info = vk::SwapchainCreateInfoKHR {
            surface: self.surface,
            min_image_count: self.image_views.len() as _,
            image_format: self.format.format,
            image_color_space: self.format.color_space,
            image_extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 1,
            p_queue_family_indices: &base.queue_family_index,
            pre_transform: SurfaceTransformFlagsKHR::IDENTITY,
            composite_alpha: self.composite_alpha,
            present_mode: self.present_mode,
            clipped: vk::TRUE,
            old_swapchain: self.inner,
            ..Default::default()
        };

        unsafe {
            let new = self.loader.create_swapchain(&create_info, None).unwrap_unchecked();
            for i in 0..self.image_views.len() {
                base.device.destroy_framebuffer(self.framebuffers[i], None);
                base.device.destroy_image_view(self.image_views[i], None);
            }
            self.loader.destroy_swapchain(self.inner, None);
            self.inner = new;
        } 

        self.image_views = Self::create_image_views(&self.loader, self.inner, &base.device, self.format.format);
        self.create_framebuffer(base, render_pass, attachment, window_size);
    }

    fn query_swap_chain_support(base: &VkBase, present_mode: vk::PresentModeKHR, surface_loader: &surface::Instance, surface: SurfaceKHR) -> (SurfaceCapabilitiesKHR, vk::SurfaceFormatKHR, vk::PresentModeKHR) {
        unsafe {
            let capabilities = surface_loader.get_physical_device_surface_capabilities(base.physical_device, surface).unwrap_unchecked();
            let format = surface_loader.get_physical_device_surface_formats(base.physical_device, surface).unwrap_unchecked().into_iter().find(|format| {format.format == vk::Format::R8G8B8A8_UNORM && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR}).unwrap();
            let present_mode = surface_loader.get_physical_device_surface_present_modes(base.physical_device, surface).unwrap_unchecked().into_iter().find(|pm| {*pm == present_mode}).unwrap_or(PresentModeKHR::FIFO);
            (capabilities, format, present_mode)
        }

    }

    fn create_swap_chain(window_size: PhysicalSize<u32>, surface: SurfaceKHR, swapchain_loader: &swapchain::Device, capabilities: &SurfaceCapabilitiesKHR, composite_alpha: vk::CompositeAlphaFlagsKHR, format: SurfaceFormatKHR, present_mode: vk::PresentModeKHR, queue_family_index: u32) -> SwapchainKHR {
        let mut image_count = capabilities.min_image_count.max(3);
        if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
            image_count = capabilities.max_image_count;
        }

        let image_extent = if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D { width: window_size.width, height: window_size.height }
        };

        let create_info = vk::SwapchainCreateInfoKHR {
            surface,
            min_image_count: image_count,
            image_format: format.format,
            image_color_space: format.color_space,
            image_extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 1,
            p_queue_family_indices: &queue_family_index,
            pre_transform: SurfaceTransformFlagsKHR::IDENTITY,
            composite_alpha,
            present_mode,
            clipped: vk::TRUE,
            ..Default::default()
        };

        unsafe { swapchain_loader.create_swapchain(&create_info, None).unwrap_unchecked() }

    }

    fn create_image_views(swapchain_loader: &swapchain::Device, swapchain: SwapchainKHR, device: &ash::Device, format: Format) -> Vec<vk::ImageView> {
        let present_images = unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };
        let mut present_image_views = Vec::with_capacity(present_images.len());

        for present_image in present_images {
            let create_info = vk::ImageViewCreateInfo {
                image: present_image,
                view_type: vk::ImageViewType::TYPE_2D,
                format,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..Default::default()
            };
           present_image_views.push(unsafe { device.create_image_view(&create_info, None).unwrap() });
        }

        present_image_views
    }

    pub fn destroy(&mut self, device: &Device) {
        unsafe  {
            for i in 0..self.image_views.len() {
                device.destroy_framebuffer(self.framebuffers[i], None);
                device.destroy_image_view(self.image_views[i], None);
            }
        
            self.loader.destroy_swapchain(self.inner, None);
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}
