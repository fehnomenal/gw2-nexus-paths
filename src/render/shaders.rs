use std::{
    ffi::{c_void, CString},
    mem::MaybeUninit,
};

use windows::Win32::Graphics::Direct3D::{
    Fxc::{D3DCompile, D3DCOMPILE_ENABLE_STRICTNESS},
    ID3DBlob,
};
use windows_strings::PCSTR;

pub(crate) unsafe fn compile_shader_to_blob(
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

    let src = SHADER_SRC;
    let c_src = CString::new(src).expect("Could not convert shader src string");

    let mut shader_blob = MaybeUninit::uninit();

    D3DCompile(
        c_src.as_ptr() as *const c_void,
        src.len(),
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
    // TODO: Error handling
    .expect("Could not compile shader");

    let blob = shader_blob.assume_init().expect("Shader blob is empty???");

    Ok(blob)
}
