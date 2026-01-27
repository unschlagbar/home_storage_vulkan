use std::io::Cursor;

use ash::vk::{
    self, BufferUsageFlags, Extent3D, Format, ImageCreateInfo, ImageLayout, ImageTiling,
    ImageUsageFlags,
};
use iron_oxide::{
    graphics::{Material, VulkanImage},
    primitives::Matrix4,
    ui::{
        Ui,
        materials::{AtlasInstance, FontInstance, ShadowInstance, UiInstance},
    },
};

use crate::vulkan_render::{MemType, VulkanRender};

#[derive(Default, Debug)]
pub struct RenderAssets {
    pub font_atlas: VulkanImage,
}

impl RenderAssets {
    pub fn init(&mut self, renderer: &mut VulkanRender) {
        #[cfg(feature = "standalone")]
        let data = {
            renderer
                .ressources
                .texture_atlas
                .get_pngs_const(&crate::TEXTURES)
        };

        #[cfg(not(feature = "standalone"))]
        let data = {
            use crate::asset_manager::AssetMananger;

            let path = AssetMananger::path("assets/textures");
            renderer.ressources.texture_atlas.get_pngs(path)
        };

        renderer.ressources.texture_atlas.load_directory(
            data,
            &renderer.base,
            MemType::DeviceLocal as usize,
            renderer.cmd_pool,
            &mut renderer.ressources.mem_manager,
        );

        self.font_atlas = Self::create_font_atlas(renderer);

        let ressources = &mut renderer.ressources;
        let base = &renderer.base;
        let window_size = renderer.window_size;
        let render_pass = renderer.render_pass;
        let device = &renderer.base.device;

        self.font_atlas.create_view(base);

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

        for (i, (buffer, buffer_mapped)) in renderer
            .uniform_buffers
            .iter_mut()
            .zip(&mut renderer.uniform_buffers_mapped)
            .enumerate()
        {
            let buffer_size;
            (*buffer, buffer_size) = ressources.mem_manager.create_buffer(
                base,
                0,
                size_of::<Matrix4>() as u64,
                BufferUsageFlags::UNIFORM_BUFFER,
            );

            let mem = &ressources.mem_manager.memory_pool[0];
            *buffer_mapped = mem.get_ptr(buffer_size as usize * i) as _
        }

        let ubo_layout = Ui::create_ubo_desc_layout(device);
        let img_layout = Ui::create_img_desc_layout(device);

        ressources.add_mat(Material::new::<UiInstance>(
            base,
            window_size,
            render_pass,
            &[ubo_layout],
            false,
            base_shaders,
        ));

        ressources.add_mat(Material::new::<FontInstance>(
            base,
            window_size,
            render_pass,
            &[ubo_layout, img_layout],
            false,
            font_shaders,
        ));

        ressources.add_mat(Material::new::<ShadowInstance>(
            base,
            window_size,
            render_pass,
            &[ubo_layout],
            true,
            shadow_shaders,
        ));

        ressources.add_mat(Material::new::<AtlasInstance>(
            base,
            window_size,
            render_pass,
            &[ubo_layout, img_layout],
            false,
            atlas_shaders,
        ));

        ressources.create_desc_sets(
            device,
            &[ubo_layout, img_layout, img_layout],
            &[1, 3],
            renderer.uniform_buffers[0],
            self.font_atlas.view,
            ressources.texture_atlas.atlas.as_ref().unwrap().view,
        );

        unsafe {
            device.destroy_descriptor_set_layout(ubo_layout, None);
            device.destroy_descriptor_set_layout(img_layout, None);
        }

        renderer.update_uniform_buffer();
    }

    fn create_font_atlas(renderer: &mut VulkanRender) -> VulkanImage {
        let decoder = png::Decoder::new(Cursor::new(include_bytes!("../font/default8.png")));

        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut buf).unwrap();
        let width = info.width;
        let height = info.height;
        let extent = Extent3D {
            width,
            height,
            depth: 1,
        };

        let create_info = ImageCreateInfo {
            image_type: vk::ImageType::TYPE_2D,
            format: Format::R8_UNORM,
            extent,
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling: ImageTiling::OPTIMAL,
            usage: ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::SAMPLED,
            ..Default::default()
        };

        let mut image = VulkanImage::create(&renderer.base, &create_info);
        renderer.ressources.mem_manager.upload_image(
            &renderer.base,
            1,
            renderer.cmd_pool,
            &mut image,
            ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            &buf,
        );

        image
    }

    pub fn destroy(&self, device: &ash::Device) {
        unsafe { device.destroy_image_view(self.font_atlas.view, None) };
    }
}
