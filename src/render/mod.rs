mod shaders;

use core::slice;
use std::{ffi::c_void, mem::MaybeUninit, sync::Once};

use shaders::compile_shader_to_blob;
use windows::Win32::Graphics::{
    Direct3D::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
    Direct3D11::{
        ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout, ID3D11PixelShader,
        ID3D11Texture2D, ID3D11VertexShader, D3D11_BIND_VERTEX_BUFFER, D3D11_BUFFER_DESC,
        D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA, D3D11_SUBRESOURCE_DATA,
        D3D11_USAGE_DEFAULT, D3D11_VIEWPORT,
    },
    Dxgi::{Common::DXGI_FORMAT_R32G32B32_FLOAT, IDXGISwapChain},
};
use windows_strings::s;

pub(crate) struct Renderer {
    swap_chain: IDXGISwapChain,
    device: ID3D11Device,
    device_context: ID3D11DeviceContext,
    state: MaybeUninit<RendererState>,

    screen_height: f32,
    screen_width: f32,

    init_once: Once,
}

struct RendererState {
    vertex_shader: ID3D11VertexShader,
    pixel_shader: ID3D11PixelShader,
    input_layout: ID3D11InputLayout,
    vertex_buffer: Option<ID3D11Buffer>,
}

impl Renderer {
    pub fn new(device: ID3D11Device, swap_chain: &IDXGISwapChain) -> windows::core::Result<Self> {
        let device_context = unsafe {
            device
                .GetImmediateContext()
                .expect("Could not get device context")
        };

        Ok(Self {
            swap_chain: swap_chain.clone(),
            device,
            device_context,
            state: MaybeUninit::uninit(),

            // TODO: Retrieve these value from somewhere and update on window resize
            screen_height: 1000.0,
            screen_width: 1000.0,

            init_once: Once::new(),
        })
    }

    pub unsafe fn render(&mut self) {
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

        let bb = unsafe {
            self.swap_chain
                .GetBuffer::<ID3D11Texture2D>(0)
                // TODO: Error handling
                .expect("Could not get back buffer")
        };

        let mut render_target_view = None;
        unsafe {
            self.device
                .CreateRenderTargetView(&bb, None, Some(&mut render_target_view))
                // TODO: Error handling
                .expect("Could not create render target view");
        };
        let render_target_view = render_target_view
            // TODO: Error handling
            .expect("Render target view is empty???");

        let state = self.state.assume_init_ref();

        // TODO: Make this variable on data to render.
        let vertex_stride = 3 * size_of::<f32>() as u32;
        let vertex_offset = 0u32;
        let vertex_count = 3u32;

        let viewport = D3D11_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Height: self.screen_height,
            Width: self.screen_width,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        };
        self.device_context.RSSetViewports(Some(&[viewport]));

        self.device_context
            .OMSetRenderTargets(Some(&[Some(render_target_view)]), None);

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

    let mut vertex_shader = None;

    device
        .CreateVertexShader(bytecode, None, Some(&mut vertex_shader))
        // TODO: Error handling
        .expect("Could not create vertex shader");

    let vertex_shader = vertex_shader
        // TODO: Error handling
        .expect("Vertex shader is empty???");

    let local_layout = vec![D3D11_INPUT_ELEMENT_DESC {
        SemanticName: s!("POS"),
        SemanticIndex: 0,
        Format: DXGI_FORMAT_R32G32B32_FLOAT,
        InputSlot: 0,
        AlignedByteOffset: 0,
        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
        InstanceDataStepRate: 0,
    }];

    let mut input_layout = None;

    device
        .CreateInputLayout(&local_layout, bytecode, Some(&mut input_layout))
        // TODO: Error handling
        .expect("Could not create input layout");

    let input_layout = input_layout
        // TODO: Error handling
        .expect("Input layout is empty???");

    Ok((vertex_shader, input_layout))
}

unsafe fn create_pixel_shader(device: &ID3D11Device) -> windows::core::Result<ID3D11PixelShader> {
    let blob = compile_shader_to_blob(s!("ps_main"), s!("ps_5_0"))
        // TODO: Error handling
        .expect("Could not compile pixel shader");

    let bytecode =
        slice::from_raw_parts(blob.GetBufferPointer() as *const u8, blob.GetBufferSize());

    let mut pixel_shader = None;

    device
        .CreatePixelShader(bytecode, None, Some(&mut pixel_shader))
        // TODO: Error handling
        .expect("Could not create pixel shader");

    let pixel_shader = pixel_shader
        // TODO: Error handling
        .expect("Pixel shader is empty???");

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

    let mut buffer = None;

    device
        .CreateBuffer(&desc, Some(&resource), Some(&mut buffer))
        // TODO: Error handling
        .expect("Could not create vertex buffer");

    let buffer = buffer
        // TODO: Error handling
        .expect("Vertex buffer is empty???");

    Ok(buffer)
}
