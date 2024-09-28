use std::{ffi::c_void, mem::MaybeUninit, rc::Rc, slice};

use windows::Win32::Graphics::{
    Direct3D::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
    Direct3D11::{
        ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout, ID3D11PixelShader,
        ID3D11VertexShader, D3D11_BIND_VERTEX_BUFFER, D3D11_BUFFER_DESC, D3D11_INPUT_ELEMENT_DESC,
        D3D11_INPUT_PER_VERTEX_DATA, D3D11_SUBRESOURCE_DATA, D3D11_USAGE_DEFAULT,
    },
    Dxgi::Common::DXGI_FORMAT_R32G32B32_FLOAT,
};
use windows_strings::s;

use super::shaders::{compile_pixel_shader_to_blob, compile_vertex_shader_to_blob};

pub struct WorldRenderer {
    state: Option<State>,

    d3d11_device_context: Rc<ID3D11DeviceContext>,
}

struct State {
    vertex_shader: ID3D11VertexShader,
    pixel_shader: ID3D11PixelShader,
    input_layout: ID3D11InputLayout,
    vertex_buffer: ID3D11Buffer,
}

impl WorldRenderer {
    pub fn new(d3d11_device_context: Rc<ID3D11DeviceContext>) -> Self {
        Self {
            state: None,

            d3d11_device_context,
        }
    }

    pub unsafe fn render(&mut self) {
        if true {
            return;
        }

        let state = self.state.get_or_insert_with(|| {
            let device = &self
                .d3d11_device_context
                .GetDevice()
                .expect("Coudl not get d3d11 device from device context");

            let (vertex_shader, input_layout) = create_vertex_shader_and_input_layout(device)
                .expect("Could not create vertex shader and input layout");

            let pixel_shader = create_pixel_shader(device).expect("Could not create pixel shader");

            let vertex_buffer =
                create_vertex_buffer(device).expect("Could not create vertex buffer");

            State {
                vertex_shader,
                pixel_shader,
                input_layout,
                vertex_buffer,
            }
        });

        // TODO: Make this variable on data to render.
        let vertex_stride = 3 * size_of::<f32>() as u32;
        let vertex_offset = 0u32;
        let vertex_count = 3u32;

        self.d3d11_device_context
            .IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
        self.d3d11_device_context
            .IASetInputLayout(&state.input_layout);
        self.d3d11_device_context.IASetVertexBuffers(
            0,
            1,
            Some(&Some(state.vertex_buffer.clone())),
            Some(&vertex_stride),
            Some(&vertex_offset),
        );

        self.d3d11_device_context
            .VSSetShader(&state.vertex_shader, None);

        self.d3d11_device_context
            .PSSetShader(&state.pixel_shader, None);

        self.d3d11_device_context.Draw(vertex_count, 0);
    }
}

unsafe fn create_vertex_shader_and_input_layout(
    device: &ID3D11Device,
) -> windows::core::Result<(ID3D11VertexShader, ID3D11InputLayout)> {
    let blob = compile_vertex_shader_to_blob()
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
    let blob = compile_pixel_shader_to_blob()
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
