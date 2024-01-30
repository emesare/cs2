use std::slice::from_raw_parts;

use windows::{
    core::PCSTR,
    Win32::Graphics::{
        Direct3D::{
            Fxc::{D3DCompile, D3DCOMPILE_DEBUG, D3DCOMPILE_ENABLE_STRICTNESS},
            ID3DBlob,
        },
        Direct3D11::{ID3D11Device, ID3D11PixelShader, ID3D11VertexShader},
    },
};

use super::DirectX11Error;

trait Shader: Sized {
    const ENTRY: PCSTR;
    const TARGET: PCSTR;

    unsafe fn create_shader(
        device: &ID3D11Device,
        blob: &ShaderData,
    ) -> Result<Self, DirectX11Error>;
}

impl Shader for ID3D11VertexShader {
    const ENTRY: PCSTR = PCSTR(c"vs_main".as_ptr() as _);
    const TARGET: PCSTR = PCSTR(c"vs_5_0".as_ptr() as _);

    unsafe fn create_shader(
        device: &ID3D11Device,
        blob: &ShaderData,
    ) -> Result<Self, DirectX11Error> {
        let mut output = None;
        match blob {
            ShaderData::EmbeddedData(arr) => {
                device.CreateVertexShader(arr, None, Some(&mut output))?;
                output.ok_or(DirectX11Error::General(
                    "Unable to create vertex shader for embedded data",
                ))
            }
            ShaderData::CompiledBlob(blob) => {
                device.CreateVertexShader(
                    from_raw_parts(blob.GetBufferPointer() as _, blob.GetBufferSize()),
                    None,
                    Some(&mut output),
                )?;
                output.ok_or(DirectX11Error::General(
                    "Unable to create vertex shader for compiled blob",
                ))
            }
        }
    }
}

impl Shader for ID3D11PixelShader {
    const ENTRY: PCSTR = PCSTR(c"ps_main".as_ptr() as _);
    const TARGET: PCSTR = PCSTR(c"ps_5_0".as_ptr() as _);

    unsafe fn create_shader(
        device: &ID3D11Device,
        blob: &ShaderData,
    ) -> Result<Self, DirectX11Error> {
        let mut output = None;
        match blob {
            ShaderData::EmbeddedData(arr) => {
                device.CreatePixelShader(arr, None, Some(&mut output))?;
                output.ok_or(DirectX11Error::General(
                    "Unable to create pixel shader for embedded data",
                ))
            }
            ShaderData::CompiledBlob(blob) => {
                device.CreatePixelShader(
                    from_raw_parts(blob.GetBufferPointer() as _, blob.GetBufferSize()),
                    None,
                    Some(&mut output),
                )?;
                output.ok_or(DirectX11Error::General(
                    "Unable to create pixel shader for compiled blob",
                ))
            }
        }
    }
}

#[allow(unused)]
pub enum ShaderData {
    EmbeddedData(&'static [u8]),
    CompiledBlob(ID3DBlob),
}

pub struct CompiledShaders {
    pub vertex: ID3D11VertexShader,
    pub pixel: ID3D11PixelShader,
    cache: ShaderData,
}

impl CompiledShaders {
    pub fn new(device: &ID3D11Device) -> Result<Self, DirectX11Error> {
        let shader_code = include_str!("shader.hlsl");
        let (vcache, vertex) = Self::compile_shader::<ID3D11VertexShader>(device, shader_code)?;
        let (_pcache, pixel) = Self::compile_shader::<ID3D11PixelShader>(device, shader_code)?;

        Ok(Self {
            vertex,
            pixel,
            cache: ShaderData::CompiledBlob(vcache),
        })
    }

    pub fn bytecode(&self) -> &[u8] {
        match &self.cache {
            ShaderData::EmbeddedData(arr) => arr,
            ShaderData::CompiledBlob(blob) => unsafe {
                from_raw_parts(blob.GetBufferPointer() as _, blob.GetBufferSize())
            },
        }
    }

    fn compile_shader<S: Shader>(
        device: &ID3D11Device,
        shader: &str,
    ) -> Result<(ID3DBlob, S), DirectX11Error> {
        let mut flags = D3DCOMPILE_ENABLE_STRICTNESS;
        if cfg!(debug_assertions) {
            flags |= D3DCOMPILE_DEBUG;
        }

        let mut code = None;
        let mut error = None;

        unsafe {
            D3DCompile(
                shader.as_ptr() as _,
                shader.len(),
                None,
                None,
                None,
                S::ENTRY,
                S::TARGET,
                flags,
                0,
                &mut code,
                Some(&mut error),
            )
            .unwrap();

            Ok((
                code.clone().unwrap(),
                S::create_shader(device, &ShaderData::CompiledBlob(code.unwrap()))?,
            ))
        }
    }
}
