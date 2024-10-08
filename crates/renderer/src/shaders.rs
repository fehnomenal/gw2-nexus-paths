use std::{
    ffi::{c_void, CString},
    mem::MaybeUninit,
};

use log_err::{LogErrOption, LogErrResult};
use windows::Win32::Graphics::Direct3D::{
    Fxc::{D3DCompile, D3DCOMPILE_ENABLE_STRICTNESS},
    ID3DBlob,
};
use windows_strings::{s, PCSTR};

pub unsafe fn compile_vertex_shader_to_blob() -> windows::core::Result<ID3DBlob> {
    compile_any_shader_to_blob(s!("vs_main"), s!("vs_5_0"))
}

pub unsafe fn compile_pixel_shader_to_blob() -> windows::core::Result<ID3DBlob> {
    compile_any_shader_to_blob(s!("ps_main"), s!("ps_5_0"))
}

unsafe fn compile_any_shader_to_blob(
    main: PCSTR,
    target: PCSTR,
) -> windows::core::Result<ID3DBlob> {
    const SHADER_SRC: &str = "
/* vertex attributes go here to input to the vertex shader */
struct vs_in {
    float3 position_local : POS;
};

/* outputs from vertex shader go here. can be interpolated to pixel shader */
struct vs_out {
    float4 position_clip : SV_POSITION; // required output of VS
};

vs_out vs_main(vs_in input) {
  vs_out output = (vs_out)0; // zero the memory first
  output.position_clip = float4(input.position_local, 1.0);
  return output;
}

float4 ps_main(vs_out input) : SV_TARGET {
  return float4( 1.0, 0.0, 1.0, 1.0 ); // must return an RGBA colour
}
    ";

    let c_src = CString::new(SHADER_SRC).log_expect("could not convert shader src string");

    let mut shader_blob = MaybeUninit::uninit();

    D3DCompile(
        c_src.as_ptr() as *const c_void,
        SHADER_SRC.len(),
        None,
        None,
        None,
        main,
        target,
        D3DCOMPILE_ENABLE_STRICTNESS,
        0,
        shader_blob.as_mut_ptr(),
        None,
    )
    .log_expect("could not compile shader");

    let blob = shader_blob
        .assume_init()
        .log_expect("shader blob is empty???");

    Ok(blob)
}
