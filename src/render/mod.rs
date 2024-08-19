mod map;
mod shaders;

use core::slice;
use std::{ffi::c_void, mem::MaybeUninit, sync::Once};

use map::MapRenderer;
use shaders::compile_shader_to_blob;
use windows::Win32::Graphics::{
    Direct3D::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
    Direct3D11::{
        ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout, ID3D11PixelShader,
        ID3D11RenderTargetView, ID3D11Texture2D, ID3D11VertexShader, D3D11_BIND_VERTEX_BUFFER,
        D3D11_BUFFER_DESC, D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA,
        D3D11_SUBRESOURCE_DATA, D3D11_USAGE_DEFAULT, D3D11_VIEWPORT,
    },
    Dxgi::{Common::DXGI_FORMAT_R32G32B32_FLOAT, IDXGISwapChain},
};
use windows_strings::s;

pub(crate) struct Renderer {
    swap_chain: IDXGISwapChain,
    device: ID3D11Device,
    device_context: ID3D11DeviceContext,
    render_target_view: Option<ID3D11RenderTargetView>,
    state: MaybeUninit<RendererState>,

    map_renderer: MapRenderer,

    init_once: Once,
}

struct RendererState {
    vertex_shader: ID3D11VertexShader,
    pixel_shader: ID3D11PixelShader,
    input_layout: ID3D11InputLayout,
    vertex_buffer: Option<ID3D11Buffer>,
}

impl Renderer {
    pub fn new(swap_chain: &IDXGISwapChain) -> Self {
        let device = unsafe {
            swap_chain
                .GetDevice::<ID3D11Device>()
                .expect("Could not get d3d11 device from swap chain")
        };

        let device_context = unsafe {
            device
                .GetImmediateContext()
                .expect("Could not get device context")
        };

        let map_renderer = MapRenderer::new(swap_chain);

        Self {
            swap_chain: swap_chain.clone(),
            device,
            device_context,
            render_target_view: None,
            state: MaybeUninit::uninit(),

            map_renderer,

            init_once: Once::new(),
        }
    }

    pub fn request_recreate_render_target(&mut self) {
        drop(self.render_target_view.take());

        self.map_renderer.request_recreate_render_target();
    }

    pub unsafe fn render(&mut self, render_state: &RenderState) {
        self.init_once.call_once(|| {
            let (vertex_shader, input_layout) = create_vertex_shader_and_input_layout(&self.device)
                .expect("Could not create vertex shader and input layout");

            let pixel_shader =
                create_pixel_shader(&self.device).expect("Could not create pixel shader");

            let vertex_buffer =
                create_vertex_buffer(&self.device).expect("Could not create vertex buffer");

            self.state.write(RendererState {
                vertex_shader,
                pixel_shader,
                input_layout,
                vertex_buffer: Some(vertex_buffer),
            });
        });

        let render_target_view = self.render_target_view.get_or_insert_with(|| {
            let viewport = D3D11_VIEWPORT {
                TopLeftX: 0.0,
                TopLeftY: 0.0,
                Width: render_state.screen_width,
                Height: render_state.screen_height,
                MinDepth: 0.0,
                MaxDepth: 1.0,
            };

            self.device_context.RSSetViewports(Some(&[viewport]));

            let bb = self
                .swap_chain
                .GetBuffer::<ID3D11Texture2D>(0)
                // TODO: Error handling
                .expect("Could not get back buffer");

            let mut render_target_view = MaybeUninit::uninit();

            self.device
                .CreateRenderTargetView(&bb, None, Some(render_target_view.as_mut_ptr()))
                // TODO: Error handling
                .expect("Could not create render target view");

            render_target_view
                .assume_init()
                // TODO: Error handling
                .expect("Render target view is empty???")
        });

        self.device_context
            .OMSetRenderTargets(Some(&[Some(render_target_view.clone())]), None);

        let state = self.state.assume_init_ref();

        // TODO: Make this variable on data to render.
        let vertex_stride = 3 * size_of::<f32>() as u32;
        let vertex_offset = 0u32;
        let vertex_count = 3u32;

        self.device_context
            .IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
        self.device_context.IASetInputLayout(&state.input_layout);
        self.device_context.IASetVertexBuffers(
            0,
            1,
            Some(&state.vertex_buffer),
            Some(&vertex_stride),
            Some(&vertex_offset),
        );

        self.device_context.VSSetShader(&state.vertex_shader, None);

        self.device_context.PSSetShader(&state.pixel_shader, None);

        self.device_context.Draw(vertex_count, 0);

        self.map_renderer.render_path_on_map(render_state);
    }
}

unsafe fn create_vertex_shader_and_input_layout(
    device: &ID3D11Device,
) -> windows::core::Result<(ID3D11VertexShader, ID3D11InputLayout)> {
    let blob = compile_shader_to_blob(s!("vs_main"), s!("vs_5_0"))
        // TODO: Error handling
        .expect("Could not compile vertex shader");

    let bytecode =
        slice::from_raw_parts(blob.GetBufferPointer() as *const u8, blob.GetBufferSize());

    let vertex_shader = unsafe {
        let mut vertex_shader = MaybeUninit::uninit();

        device
            .CreateVertexShader(bytecode, None, Some(vertex_shader.as_mut_ptr()))
            // TODO: Error handling
            .expect("Could not create vertex shader");

        vertex_shader
            .assume_init()
            // TODO: Error handling
            .expect("Vertex shader is empty???")
    };

    let local_layout = vec![D3D11_INPUT_ELEMENT_DESC {
        SemanticName: s!("POS"),
        SemanticIndex: 0,
        Format: DXGI_FORMAT_R32G32B32_FLOAT,
        InputSlot: 0,
        AlignedByteOffset: 0,
        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
        InstanceDataStepRate: 0,
    }];

    let input_layout = unsafe {
        let mut input_layout = MaybeUninit::uninit();

        device
            .CreateInputLayout(&local_layout, bytecode, Some(input_layout.as_mut_ptr()))
            // TODO: Error handling
            .expect("Could not create input layout");

        input_layout
            .assume_init()
            // TODO: Error handling
            .expect("Input layout is empty???")
    };

    Ok((vertex_shader, input_layout))
}

unsafe fn create_pixel_shader(device: &ID3D11Device) -> windows::core::Result<ID3D11PixelShader> {
    let blob = compile_shader_to_blob(s!("ps_main"), s!("ps_5_0"))
        // TODO: Error handling
        .expect("Could not compile pixel shader");

    let bytecode =
        slice::from_raw_parts(blob.GetBufferPointer() as *const u8, blob.GetBufferSize());

    let pixel_shader = unsafe {
        let mut pixel_shader = MaybeUninit::uninit();

        device
            .CreatePixelShader(bytecode, None, Some(pixel_shader.as_mut_ptr()))
            // TODO: Error handling
            .expect("Could not create pixel shader");
        pixel_shader
            .assume_init()
            // TODO: Error handling
            .expect("Pixel shader is empty???")
    };

    Ok(pixel_shader)
}

unsafe fn create_vertex_buffer(device: &ID3D11Device) -> windows::core::Result<ID3D11Buffer> {
    // TODO: Make this dynamic.
    let vertices: [f32; 9] = [
        0.0, 0.5, 0.0, // point at top
        0.5, -0.5, 0.0, // point at bottom-right
        -0.5, -0.5, 0.0, // point at bottom-left
    ];

    // TODO: Somehow calculate this based on the data to render.
    let size = size_of::<[f32; 9]>();

    let desc = D3D11_BUFFER_DESC {
        ByteWidth: size as u32,
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as u32,
        ..Default::default()
    };

    let resource = D3D11_SUBRESOURCE_DATA {
        pSysMem: vertices.as_ptr() as *const c_void,
        ..Default::default()
    };

    let buffer = unsafe {
        let mut buffer = MaybeUninit::uninit();

        device
            .CreateBuffer(&desc, Some(&resource), Some(buffer.as_mut_ptr()))
            // TODO: Error handling
            .expect("Could not create vertex buffer");

        buffer
            .assume_init()
            // TODO: Error handling
            .expect("Vertex buffer is empty???")
    };

    Ok(buffer)
}

pub struct RenderState {
    pub screen_width: f32,
    pub screen_height: f32,
    pub half_screen_width: f32,
    pub half_screen_height: f32,
    pub map_scale_factor: f32,
}

impl RenderState {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            screen_width,
            screen_height,
            half_screen_width: screen_width / 2.0,
            half_screen_height: screen_height / 2.0,
            map_scale_factor: 1.0,
        }
    }

    pub fn update_screen_size(&mut self, width: f32, height: f32) {
        self.screen_width = width;
        self.screen_height = height;
        self.half_screen_width = width / 2.0;
        self.half_screen_height = height / 2.0;
    }

    pub fn update_ui_size(&mut self, ui_size: u8) {
        self.map_scale_factor = match ui_size {
            0 => 1.111,
            1 => 1.0,
            2 => 0.9,
            3 => 0.82,
            _ => 1.0,
        };
    }
}
