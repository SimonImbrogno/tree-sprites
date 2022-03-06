use std::mem::size_of;
use std::time::Duration;

use anyhow::Result;
use log::debug;
use winit::window::Window;

use crate::game::game_state::{self, GameState, GroundCover, SoilType};
use crate::game::{TileType, self};
use crate::timer::{AverageDurationTimer, TargetTimer, TimerState, Timer};
use crate::timer::measure;

use super::buffer::{QuadBuffer, DrawGeometryBuffer, WriteGeometryBuffer};
use super::buffer_usages::BufferUsages;
use super::camera::{Camera, CameraUniform};
use super::quad::{TexturedQuad, TexturedUvQuad, UntexturedQuad};
use super::sprite_sheet::{SpriteSheet};
use super::texture::Texture;
use super::utils::gpu::{ create_buffer, create_shader_module, create_render_pipeline };
use super::vertex::{Vertex, TexturedVertex, UvVertex};

pub struct RenderState {
    window_size: winit::dpi::PhysicalSize<u32>,
    camera: Camera,

    instance: wgpu::Instance,
    adapter: wgpu::Adapter,

    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,

    device: wgpu::Device,
    queue: wgpu::Queue,

    tile_quad_buffer: QuadBuffer<TexturedVertex, u16>,
    shadow_quad_buffer: QuadBuffer<UvVertex, u16>,
    entity_quad_buffer: QuadBuffer<TexturedVertex, u16>,

    sprite_sheet: SpriteSheet<TileType>,
    tile_sprite_sheet: Texture,
    tile_sprite_sheet_bind_group: wgpu::BindGroup,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    clear_color: [f64; 3],
    tile_render_pipeline: wgpu::RenderPipeline,
    shadow_render_pipeline: wgpu::RenderPipeline,

    //Timers...
    debug_log_timer: TargetTimer,
    ground_render_timer: AverageDurationTimer<20>,
    tree_render_timer: AverageDurationTimer<20>,
}

impl RenderState {
    pub async fn new(window: &Window, game_state: &GameState) -> Self {
        let window_size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());

        debug!("Requesting handle to device...");

        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            }
        ).await.expect("Failed to request suitable adapter.");

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("render_state.device"),
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None
        ).await.expect("Failed to request device.");

        debug!("Configuring surface.");

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(), //NOTE: Unwrap is safe here, we've created the adapter to specifically support the surface.
            width: window_size.width,
            height: window_size.height,

            // present_mode:
            //  Fifo      => VScync.
            //  Mailbox   => Submit eagerly, VScync, fallback to Fifo if unsupported.
            //  Immediate => Low latency, no wait for display, tearing may be observed.
            present_mode: wgpu::PresentMode::Mailbox,
        };

        surface.configure(&device, &surface_config);

        debug!("Compiling shaders...");

        let main_shader = create_shader_module(&device, "render_state.render_pipeline -> main_shader", include_str!("../../res/shaders/main_shader.wgsl"));
        let circle_shader = create_shader_module(&device, "render_state.render_pipeline -> circle_shader", include_str!("../../res/shaders/circle_shader.wgsl"));

        debug!("Loading textures...");

        let sprite_sheet_bytes = include_bytes!("../../res/textures/tile_sprite_sheet.png");
        let tile_sprite_sheet = Texture::try_from_bytes(Some("Goose Texture"), sprite_sheet_bytes, &device, &queue).unwrap();

        let sprite_sheet_layout = crate::game::get_sprite_sheet_layout();
        let sprite_sheet = SpriteSheet::try_load_from_bytes(sprite_sheet_bytes, &sprite_sheet_layout, &device, &queue).unwrap();

        debug!("Creating buffers...");

        let buffer_capacity = 50000;
        let tile_quad_buffer = QuadBuffer::new(&device, "render_state.tile_quad_buffer", buffer_capacity);
        let shadow_quad_buffer = QuadBuffer::new(&device, "render_state.shadow_quad_buffer", buffer_capacity);
        let entity_quad_buffer = QuadBuffer::new(&device, "render_state.entity_quad_buffer", buffer_capacity);

        let camera = Camera {
            aspect_ratio: 1.0,
            position: cgmath::Point3::new(0.0, 0.0, 1.0),
            y_axis_dim: 5.0,
        };

        let camera_buffer = create_buffer(&device, "render_state.camera_buffer", size_of::<CameraUniform>(), BufferUsages::UniformCopyDst.into());
        queue.write_buffer(&camera_buffer, 0, bytemuck::cast_slice(&[CameraUniform::from(camera)]));

        debug!("Creating render pipeline bind groups...");

        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("render_state.camera_bind_group -> layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
            }
        );

        let camera_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("render_state.camera_bind_group"),
                layout: &camera_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }
                ],
            }
        );

        let tile_sprite_sheet_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("render_state.tile_sprite_sheet_bind_group -> layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            filtering: true,
                            comparison: false,
                        },
                        count: None,
                    }
                ],
            }
        );

        let tile_sprite_sheet_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("render_state.tile_sprite_sheet_bind_group"),
                layout: &tile_sprite_sheet_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&sprite_sheet.texture.view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sprite_sheet.texture.sampler) },
                ],
            }
        );

        debug!("Creating render pipelines...");

        let tile_render_pipeline = {
            let label = "render_state.tile_render_pipeline";
            let bind_group_layouts = [&camera_bind_group_layout, &tile_sprite_sheet_bind_group_layout];
            let push_constant_ranges = [];
            let buffer_layouts = [TexturedVertex::describe_buffer()];

            create_render_pipeline(
                &device,
                label,
                &bind_group_layouts,
                &push_constant_ranges,
                &buffer_layouts,
                surface_config.format,
                &main_shader
            )
        };

        let shadow_render_pipeline = {
            let label = "render_state.shadow_render_pipeline";
            let bind_group_layouts = [&camera_bind_group_layout];
            let push_constant_ranges = [];
            let buffer_layouts = [UvVertex::describe_buffer()];

            create_render_pipeline(
                &device,
                label,
                &bind_group_layouts,
                &push_constant_ranges,
                &buffer_layouts,
                surface_config.format,
                &circle_shader
            )
        };

        Self {
            window_size,
            camera,

            instance,
            adapter,

            surface,
            surface_config,

            device,
            queue,

            tile_quad_buffer,
            shadow_quad_buffer,
            entity_quad_buffer,

            sprite_sheet,
            tile_sprite_sheet,
            tile_sprite_sheet_bind_group,

            camera_buffer,
            camera_bind_group,

            //render_pipeline,
            clear_color: [0.0, 0.0, 0.0],
            tile_render_pipeline,
            shadow_render_pipeline,

            debug_log_timer: TargetTimer::new(Duration::from_secs(1)),
            ground_render_timer: AverageDurationTimer::new(),
            tree_render_timer: AverageDurationTimer::new(),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.window_size = new_size;

            self.surface_config.width  = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn try_render(&mut self, game_state: &GameState) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let output_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("render_state -> encoder"),
            }
        );

        let [r, g, b] = self.clear_color;
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("render_state -> render_pass"),
            color_attachments: &[
                wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a: 1.0 }),
                        store: true
                    },
                }
            ],
            depth_stencil_attachment: None,
        };

        self.camera.update(&game_state.camera, self.window_size);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[CameraUniform::from(self.camera)]));

        self.tile_quad_buffer.reset();
        self.shadow_quad_buffer.reset();
        self.entity_quad_buffer.reset();

        measure!(self.ground_render_timer, {
            self.draw_ground(game_state);
        });

        self.draw_debug_grid(game_state);

        measure!(self.tree_render_timer, {
            self.draw_trees(game_state);
        });

        let mut render_pass = encoder.begin_render_pass(&render_pass_descriptor);

        self.queue.write_geometry_buffer(&mut self.tile_quad_buffer);
        self.queue.write_geometry_buffer(&mut self.shadow_quad_buffer);
        self.queue.write_geometry_buffer(&mut self.entity_quad_buffer);

        render_pass.set_pipeline(&self.tile_render_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.tile_sprite_sheet_bind_group, &[]);
        render_pass.draw_geometry_buffer(&self.tile_quad_buffer);

        render_pass.set_pipeline(&self.shadow_render_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.draw_geometry_buffer(&self.shadow_quad_buffer);

        render_pass.set_pipeline(&self.tile_render_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.tile_sprite_sheet_bind_group, &[]);
        render_pass.draw_geometry_buffer(&self.entity_quad_buffer);

        drop(render_pass);
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        if let TimerState::Ready(_) = self.debug_log_timer.check() {
            self.debug_log_timer.reset();
            debug!(
                "Ground + Trees took : {:?} + {:?}",
                self.ground_render_timer.average(),
                self.tree_render_timer.average()
            );
        }

        Ok(())
    }

    fn draw_ground(&mut self, game_state: &GameState) {
        use game_state::GRID_DIM;
        use game_state::TILE_DIM;
        use game_state::TILE_RAD;

        const GRID_DIM_I32: i32 = GRID_DIM as i32;
        const MAX_XY: i32 = GRID_DIM_I32 - 1;

        //NOTE:
        //  Because we're rendering the _dual of the grid_, we're (over/under)-iterating and then
        //  clamping to generate dual nodes for grid cells at the edge (i.e cells without neighbors on all sides).
        for tile_x in -1..=MAX_XY {
            for tile_y in -1..=MAX_XY {

                let bl_index = {
                    let x = tile_x.clamp(0, MAX_XY);
                    let y = tile_y.clamp(0, MAX_XY);
                    ((y * GRID_DIM_I32) + x) as usize
                };

                let br_index = {
                    let x = (tile_x + 1).clamp(0, MAX_XY);
                    let y =  tile_y     .clamp(0, MAX_XY);
                    ((y * GRID_DIM_I32) + x) as usize
                };

                let tl_index = {
                    let x =  tile_x     .clamp(0, MAX_XY);
                    let y = (tile_y + 1).clamp(0, MAX_XY);
                    ((y * GRID_DIM_I32) + x) as usize
                };

                let tr_index = {
                    let x = (tile_x + 1).clamp(0, MAX_XY);
                    let y = (tile_y + 1).clamp(0, MAX_XY);
                    ((y * GRID_DIM_I32) + x) as usize
                };

                // SAFETY:
                //  Indices are clamped [0, GRID_DIM-1]
                let (bl, br, tl, tr) = unsafe {
                    (
                        game_state.tiles.get(bl_index).unwrap(),
                        game_state.tiles.get(br_index).unwrap(),
                        game_state.tiles.get(tl_index).unwrap(),
                        game_state.tiles.get(tr_index).unwrap(),
                    )
                };

                let grass_cover = {
                    const GRASS_TILES: [Option<TileType>; 16] = [
                        None,                           //0000
                        Some(TileType::GrassBR),        //0001
                        Some(TileType::GrassBL),        //0010
                        Some(TileType::GrassB),         //0011
                        Some(TileType::GrassTL),        //0100
                        Some(TileType::GrassDiagDown),  //0101
                        Some(TileType::GrassL),         //0110
                        Some(TileType::GrassBTL),       //0111
                        Some(TileType::GrassTR),        //1000
                        Some(TileType::GrassR),         //1001
                        Some(TileType::GrassDiagUp),    //1010
                        Some(TileType::GrassBTR),       //1011
                        Some(TileType::GrassT),         //1100
                        Some(TileType::GrassTBR),       //1101
                        Some(TileType::GrassTBL),       //1110
                        Some(TileType::Grass),          //1111
                    ];

                    let mut i = 0;

                    if let GroundCover::Grass = tr.0 { i |= 0b1000 }
                    if let GroundCover::Grass = tl.0 { i |= 0b0100 }
                    if let GroundCover::Grass = bl.0 { i |= 0b0010 }
                    if let GroundCover::Grass = br.0 { i |= 0b0001 }

                    // SAFETY:
                    //  indices 0000 -> 1111 are saturated.
                    unsafe { GRASS_TILES.get_unchecked(i) }
                };

                let stone_cover = {
                    const STONE_TILES: [Option<TileType>; 16] = [
                        None,                           //0000
                        Some(TileType::StoneBR),        //0001
                        Some(TileType::StoneBL),        //0010
                        Some(TileType::StoneB),         //0011
                        Some(TileType::StoneTL),        //0100
                        Some(TileType::StoneDiagDown),  //0101
                        Some(TileType::StoneL),         //0110
                        Some(TileType::StoneBTL),       //0111
                        Some(TileType::StoneTR),        //1000
                        Some(TileType::StoneR),         //1001
                        Some(TileType::StoneDiagUp),    //1010
                        Some(TileType::StoneBTR),       //1011
                        Some(TileType::StoneT),         //1100
                        Some(TileType::StoneTBR),       //1101
                        Some(TileType::StoneTBL),       //1110
                        Some(TileType::Stone),          //1111
                    ];

                    let mut i = 0;

                    if let SoilType::Stony = tr.1 { i |= 0b1000 }
                    if let SoilType::Stony = tl.1 { i |= 0b0100 }
                    if let SoilType::Stony = bl.1 { i |= 0b0010 }
                    if let SoilType::Stony = br.1 { i |= 0b0001 }

                    // SAFETY:
                    //  indices 0000 -> 1111 are saturated.
                    unsafe { STONE_TILES.get_unchecked(i) }
                };

                //NOTE:
                //  Around the edge we just render a half size apron, these are fake "tiles",
                //  the grid cells here don't actually have neighbors. Clamping snaps the apron to the edge of the actual grid.
                let x = (((tile_x as f32) * TILE_DIM) + TILE_RAD).clamp(0.0, (TILE_DIM * GRID_DIM as f32));
                let y = (((tile_y as f32) * TILE_DIM) + TILE_RAD).clamp(0.0, (TILE_DIM * GRID_DIM as f32));

                let mut min_u = 0.0;
                let mut max_u = 1.0;
                let mut min_v = 0.0;
                let mut max_v = 1.0;

                {
                    let max = (TILE_DIM * GRID_DIM as f32);

                    //Janky.
                    if      (x - 0.0).abs() < f32::EPSILON { min_u = 0.5; }
                    else if ((x + TILE_RAD) - max).abs() < f32::EPSILON { max_u = 0.5; }

                    if      (y - 0.0).abs() < f32::EPSILON { min_v = 0.5; }
                    else if ((y + TILE_RAD) - max).abs() < f32::EPSILON { max_v = 0.5; }
                }

                let mut dim_x = TILE_DIM;
                let mut dim_y = TILE_DIM;

                if tile_x == -1 || tile_x == MAX_XY { dim_x *= 0.5; }
                if tile_y == -1 || tile_y == MAX_XY { dim_y *= 0.5; }

                if grass_cover.is_none() || grass_cover.unwrap() != TileType::Grass {
                    let quad = TexturedUvQuad {
                        pos: (x, y),
                        dim: (dim_x, dim_y),
                        uv_min: (min_u, min_v),
                        uv_max: (max_u, max_v),
                        tex_index: self.sprite_sheet.get_texture_index(TileType::Dirt) as i32,
                    };

                    self.tile_quad_buffer.push_quad(quad);
                }

                if let Some(cover_type) = grass_cover {
                    let quad = TexturedUvQuad {
                        pos: (x, y),
                        dim: (dim_x, dim_y),
                        uv_min: (min_u, min_v),
                        uv_max: (max_u, max_v),
                        tex_index: self.sprite_sheet.get_texture_index(*cover_type) as i32,
                    };

                    self.tile_quad_buffer.push_quad(quad);
                }

                if let Some(cover_type) = stone_cover {
                    let quad = TexturedUvQuad {
                        pos: (x, y),
                        dim: (dim_x, dim_y),
                        uv_min: (min_u, min_v),
                        uv_max: (max_u, max_v),
                        tex_index: self.sprite_sheet.get_texture_index(*cover_type) as i32,
                    };

                    self.tile_quad_buffer.push_quad(quad);
                }
            }
        }
    }

    fn draw_trees(&mut self, game_state: &GameState) {
        use game_state::GRID_DIM;
        use game_state::TILE_DIM;
        use game_state::TILE_RAD;

        if !game_state.debug.show_trees { return; }

        //TODO: Memory Arena
        let mut trees_to_render = Vec::with_capacity(game_state.count_trees);

        for tile_index in 0..game_state::GRID_SIZE {
            // SAFETY:
            //  tile index ranges from 0..GRID_SIZE
            let tree_iter = unsafe { game_state.iter_trees_on_tile_unchecked(tile_index) };

            for tree in tree_iter {
                //NOTE: Super hacky, because sprites don't encode "semantic" origin point, we're cheesing it.
                //      shadows for 1 px wide trees are offset by half the tile width PLUS half of one pixel (1/32 tiles).
                let do_hacky_shadow_offset = match tree.stage {
                    game::TreeGrowthStage::Sprout   |
                    game::TreeGrowthStage::Seedling => true,
                    _ => false,
                };

                let x = (tile_index % GRID_DIM) as f32 * TILE_DIM - (TILE_RAD) + (TILE_DIM * tree.position.offset.x);
                let y = (tile_index / GRID_DIM) as f32 * TILE_DIM              + (TILE_DIM * tree.position.offset.y);

                let tex_index = self.sprite_sheet.get_texture_index(TileType::from(tree)) as i32;
                let shadow_radius = tree.species.shadow_radius(tree.stage);

                trees_to_render.push((x, y, tex_index, shadow_radius, do_hacky_shadow_offset));
            }
        }

        trees_to_render.sort_unstable_by(|(_, a, ..), (_, b, ..)| b.partial_cmp(a).unwrap());

        for &(x, y, _, shadow_rad, do_hacky_shadow_offset) in trees_to_render.iter() {
            let dim_x = TILE_DIM * shadow_rad;
            let dim_y = TILE_DIM * shadow_rad * 0.25 ;

            let mut pos_x = x + ((1.0 - dim_x) * 0.5);
            let pos_y = y - (dim_y * 0.5);

            if do_hacky_shadow_offset {
                pos_x += (TILE_DIM / 32.0) / 2.0;
            }

            let quad = UntexturedQuad {
                pos: (pos_x, pos_y),
                dim: (dim_x, dim_y),
            };

            self.shadow_quad_buffer.push_quad(quad);
        }

        for &(x, y, tex_index, ..) in trees_to_render.iter() {
            let quad = TexturedQuad {
                pos: (x, y),
                dim: (TILE_DIM, TILE_DIM),
                tex_index
            };

            self.entity_quad_buffer.push_quad(quad);
        }
    }

    fn draw_debug_grid(&mut self, game_state: &GameState) {
        use game_state::GRID_DIM;
        use game_state::TILE_DIM;
        use game_state::TILE_RAD;


        const GRID_DIM_I32: i32 = GRID_DIM as i32;
        const MAX_XY: i32 = GRID_DIM_I32 - 1;

        if game_state.debug.show_dual {
            // render Dual Grid Lines
            for tile_x in -1..=MAX_XY {
                for tile_y in -1..=MAX_XY {
                    let x = (((tile_x as f32) * TILE_DIM) + TILE_RAD).clamp(0.0, (TILE_DIM * GRID_DIM as f32));
                    let y = (((tile_y as f32) * TILE_DIM) + TILE_RAD).clamp(0.0, (TILE_DIM * GRID_DIM as f32));

                    let mut dim_x = TILE_DIM;
                    let mut dim_y = TILE_DIM;

                    if tile_x == -1 || tile_x == MAX_XY { dim_x *= 0.5; }
                    if tile_y == -1 || tile_y == MAX_XY { dim_y *= 0.5; }

                    let quad = TexturedQuad {
                        pos: (x, y),
                        dim: (dim_x, dim_y),
                        tex_index: self.sprite_sheet.get_texture_index(TileType::GridLine) as i32,
                    };

                    self.tile_quad_buffer.push_quad(quad);
                }
            }
        }

        if game_state.debug.show_grid {
            // render Grid Lines
            for tile_x in 0..(GRID_DIM) {
                for tile_y in 0..(GRID_DIM) {
                    let x = ((tile_x as f32) * TILE_DIM);
                    let y = ((tile_y as f32) * TILE_DIM);

                    let quad = TexturedQuad {
                        pos: (x, y),
                        dim: (TILE_DIM, TILE_DIM),
                        tex_index: self.sprite_sheet.get_texture_index(TileType::GridLine) as i32,
                    };

                    self.tile_quad_buffer.push_quad(quad);
                }
            }
        }
    }
}
