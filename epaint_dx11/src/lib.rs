use std::mem::size_of;

use backup::BackupState;
use epaint::{textures::TexturesDelta, ClippedPrimitive, Primitive};
use mesh::{GpuMesh, GpuVertex};
use shaders::CompiledShaders;
use texture::TextureAllocator;
use windows::{
    core::{HRESULT, PCSTR},
    Win32::{
        Foundation::RECT,
        Graphics::{
            Direct3D::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
            Direct3D11::*,
            Dxgi::{
                Common::{
                    DXGI_FORMAT_R32G32B32A32_FLOAT, DXGI_FORMAT_R32G32_FLOAT, DXGI_FORMAT_R32_UINT,
                },
                IDXGISwapChain, DXGI_SWAP_CHAIN_DESC,
            },
        },
    },
};

mod backup;
mod mesh;
mod shaders;
mod texture;

#[derive(Debug, thiserror::Error)]
pub enum DirectX11Error {
    // TODO: Get rid of `DirectX11Error::General` and specialize.
    #[error("Unrecoverable error occured {0}")]
    General(&'static str),

    #[error("Windows error {0}")]
    Win(#[from] windows::core::Error),
}

const INPUT_ELEMENTS_DESC: [D3D11_INPUT_ELEMENT_DESC; 3] = [
    D3D11_INPUT_ELEMENT_DESC {
        SemanticName: PCSTR(c"POSITION".as_ptr() as _),
        SemanticIndex: 0,
        Format: DXGI_FORMAT_R32G32_FLOAT,
        InputSlot: 0,
        AlignedByteOffset: 0,
        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
        InstanceDataStepRate: 0,
    },
    D3D11_INPUT_ELEMENT_DESC {
        SemanticName: PCSTR(c"TEXCOORD".as_ptr() as _),
        SemanticIndex: 0,
        Format: DXGI_FORMAT_R32G32_FLOAT,
        InputSlot: 0,
        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
        InstanceDataStepRate: 0,
    },
    D3D11_INPUT_ELEMENT_DESC {
        SemanticName: PCSTR(c"COLOR".as_ptr() as _),
        SemanticIndex: 0,
        Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
        InputSlot: 0,
        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
        InstanceDataStepRate: 0,
    },
];

pub struct DirectX11Renderer {
    render_view: Option<ID3D11RenderTargetView>,
    tex_alloc: TextureAllocator,
    input_layout: ID3D11InputLayout,
    shaders: CompiledShaders,
    pub backup: BackupState,
}

impl DirectX11Renderer {
    /// Create a new directx11 renderer from a swapchain
    pub unsafe fn init_from_swapchain(swapchain: &IDXGISwapChain) -> Result<Self, DirectX11Error> {
        let mut swap_chain_desc = DXGI_SWAP_CHAIN_DESC::default();
        swapchain.GetDesc(&mut swap_chain_desc)?;

        let dev: ID3D11Device = swapchain.GetDevice()?;
        let backbuffer: ID3D11Texture2D = swapchain.GetBuffer(0)?;

        let mut render_view = None;
        dev.CreateRenderTargetView(&backbuffer, None, Some(&mut render_view))?;

        let shaders = CompiledShaders::new(&dev)?;
        let mut input_layout = None;
        dev.CreateInputLayout(
            &INPUT_ELEMENTS_DESC,
            shaders.bytecode(),
            Some(&mut input_layout),
        )?;
        let input_layout =
            input_layout.ok_or(DirectX11Error::General("failed to initialize input layout"))?;

        Ok(Self {
            tex_alloc: TextureAllocator::default(),
            backup: BackupState::default(),
            input_layout,
            render_view,
            shaders,
        })
    }

    /// Paints the primitives to the swapchain.
    ///
    /// NOTE: This should be called _ONCE_ per frame.
    #[allow(clippy::cast_ref_to_mut)]
    pub unsafe fn paint_primitives(
        &mut self,
        // TODO: Add window size as param?
        swap_chain: &IDXGISwapChain,
        target_size: (f32, f32),
        textures_delta: TexturesDelta,
        primitives: Vec<ClippedPrimitive>,
    ) -> Result<(), DirectX11Error> {
        #[cfg(feature = "puffin")]
        puffin::profile_function!();

        let device: ID3D11Device = swap_chain.GetDevice()?;
        let context = device.GetImmediateContext()?;
        self.backup.save(&context);

        if !textures_delta.is_empty() {
            self.tex_alloc
                .process_deltas(&device, &context, textures_delta)?;
        }

        if primitives.is_empty() {
            self.backup.restore(&context);
            return Ok(());
        }

        self.set_blend_state(&device, &context)?;
        //self.set_depth_stencil_state(&device, &context)?;
        self.set_raster_options(&device, &context)?;
        self.set_sampler_state(&device, &context)?;

        // `ClippedPrimitive` -> `GpuMesh`
        let meshes = primitives
            .into_iter()
            .filter_map(|clip_prim| match clip_prim.primitive {
                Primitive::Mesh(mesh) => GpuMesh::from_mesh(target_size, mesh, clip_prim.clip_rect),
                Primitive::Callback(_) => {
                    panic!("custom rendering callbacks are not implemented")
                }
            })
            .collect::<Vec<_>>();

        context.RSSetViewports(Some(&[D3D11_VIEWPORT {
            TopLeftX: 0.,
            TopLeftY: 0.,
            Width: target_size.0,
            Height: target_size.1,
            MinDepth: 0.,
            MaxDepth: 1.,
        }]));
        context.OMSetRenderTargets(Some(&[self.render_view.clone()]), None);
        context.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
        context.IASetInputLayout(&self.input_layout);

        for mesh in meshes {
            let idx = mesh::create_index_buffer(&device, &mesh)?;
            let vtx = mesh::create_vertex_buffer(&device, &mesh)?;

            let texture = self.tex_alloc.get_by_id(mesh.texture_id);

            context.RSSetScissorRects(Some(&[RECT {
                left: mesh.clip.left() as _,
                top: mesh.clip.top() as _,
                right: mesh.clip.right() as _,
                bottom: mesh.clip.bottom() as _,
            }]));

            if texture.is_some() {
                context.PSSetShaderResources(0, Some(&[texture]));
            }

            context.IASetVertexBuffers(
                0,
                1,
                Some(&Some(vtx)),
                Some(&(size_of::<GpuVertex>() as _)),
                Some(&0),
            );
            context.IASetIndexBuffer(&idx, DXGI_FORMAT_R32_UINT, 0);
            context.VSSetShader(&self.shaders.vertex, Some(&[]));
            context.PSSetShader(&self.shaders.pixel, Some(&[]));

            context.DrawIndexed(mesh.indices.len() as _, 0, 0);
        }

        self.backup.restore(&context);

        Ok(())
    }

    // TODO: Fix this so we dont need to know about original.
    /// Call when resizing buffers.
    /// Do not call the original function before it, instead call it inside of the `original` closure.
    /// # Behavior
    /// In `origin` closure make sure to call the original `ResizeBuffers`.
    pub unsafe fn on_resize_buffers(
        &mut self,
        swap_chain: IDXGISwapChain,
        original: impl FnOnce() -> HRESULT,
    ) -> Result<HRESULT, DirectX11Error> {
        // TODO: This fucking sucks.
        drop(self.render_view.take());
        let result = original();
        let backbuffer: ID3D11Texture2D = swap_chain.GetBuffer(0)?;
        let device: ID3D11Device = swap_chain.GetDevice()?;
        device.CreateRenderTargetView(&backbuffer, None, Some(&mut self.render_view))?;
        Ok(result)
    }

    fn set_blend_state(
        &self,
        dev: &ID3D11Device,
        ctx: &ID3D11DeviceContext,
    ) -> Result<(), DirectX11Error> {
        // For premultipled alpha
        let mut targets: [D3D11_RENDER_TARGET_BLEND_DESC; 8] = Default::default();
        targets[0].BlendEnable = true.into();
        targets[0].SrcBlend = D3D11_BLEND_SRC_ALPHA;
        targets[0].DestBlend = D3D11_BLEND_INV_SRC_ALPHA;
        targets[0].BlendOp = D3D11_BLEND_OP_ADD;
        targets[0].SrcBlendAlpha = D3D11_BLEND_ONE;
        targets[0].DestBlendAlpha = D3D11_BLEND_INV_SRC_ALPHA;
        targets[0].BlendOpAlpha = D3D11_BLEND_OP_ADD;
        targets[0].RenderTargetWriteMask = D3D11_COLOR_WRITE_ENABLE_ALL.0 as _;

        let blend_desc = D3D11_BLEND_DESC {
            AlphaToCoverageEnable: false.into(),
            IndependentBlendEnable: false.into(),
            RenderTarget: targets,
        };

        unsafe {
            let mut blend_state = None;
            dev.CreateBlendState(&blend_desc, Some(&mut blend_state))?;
            let blend_state =
                blend_state.ok_or(DirectX11Error::General("Unable to set blend state"))?;
            ctx.OMSetBlendState(&blend_state, Some(&[0., 0., 0., 0.]), 0xffffffff);
        }

        Ok(())
    }

    fn set_raster_options(
        &self,
        dev: &ID3D11Device,
        ctx: &ID3D11DeviceContext,
    ) -> Result<(), DirectX11Error> {
        let raster_desc = D3D11_RASTERIZER_DESC {
            FillMode: D3D11_FILL_SOLID,
            CullMode: D3D11_CULL_NONE,
            FrontCounterClockwise: false.into(),
            DepthBias: false.into(),
            DepthBiasClamp: 0.,
            SlopeScaledDepthBias: 0.,
            DepthClipEnable: false.into(),
            ScissorEnable: true.into(),
            MultisampleEnable: false.into(),
            AntialiasedLineEnable: false.into(),
        };

        unsafe {
            let mut options = None;
            dev.CreateRasterizerState(&raster_desc, Some(&mut options))?;
            let options = options.ok_or(DirectX11Error::General("Unable to set options"))?;
            ctx.RSSetState(&options);
            Ok(())
        }
    }

    fn set_sampler_state(
        &self,
        dev: &ID3D11Device,
        ctx: &ID3D11DeviceContext,
    ) -> Result<(), DirectX11Error> {
        let desc = D3D11_SAMPLER_DESC {
            Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
            AddressU: D3D11_TEXTURE_ADDRESS_BORDER,
            AddressV: D3D11_TEXTURE_ADDRESS_BORDER,
            AddressW: D3D11_TEXTURE_ADDRESS_BORDER,
            MipLODBias: 0.,
            ComparisonFunc: D3D11_COMPARISON_ALWAYS,
            MinLOD: 0.,
            MaxLOD: 0.,
            BorderColor: [1., 1., 1., 1.],
            ..Default::default()
        };

        unsafe {
            let mut sampler = None;
            dev.CreateSamplerState(&desc, Some(&mut sampler))?;
            ctx.PSSetSamplers(0, Some(&[sampler]));
            Ok(())
        }
    }

    fn set_depth_stencil_state(
        &self,
        dev: &ID3D11Device,
        ctx: &ID3D11DeviceContext,
    ) -> Result<(), DirectX11Error> {
        let desc = D3D11_DEPTH_STENCIL_DESC {
            DepthEnable: false.into(),
            DepthWriteMask: D3D11_DEPTH_WRITE_MASK_ALL,
            DepthFunc: D3D11_COMPARISON_LESS,
            StencilEnable: true.into(),
            StencilReadMask: 0xFF,
            StencilWriteMask: 0xFF,
            FrontFace: D3D11_DEPTH_STENCILOP_DESC {
                StencilFailOp: D3D11_STENCIL_OP_KEEP,
                StencilDepthFailOp: D3D11_STENCIL_OP_INCR,
                StencilPassOp: D3D11_STENCIL_OP_KEEP,
                StencilFunc: D3D11_COMPARISON_ALWAYS,
            },
            BackFace: D3D11_DEPTH_STENCILOP_DESC {
                StencilFailOp: D3D11_STENCIL_OP_KEEP,
                StencilDepthFailOp: D3D11_STENCIL_OP_DECR,
                StencilPassOp: D3D11_STENCIL_OP_KEEP,
                StencilFunc: D3D11_COMPARISON_ALWAYS,
            },
        };

        unsafe {
            let mut state = None;
            dev.CreateDepthStencilState(&desc, Some(&mut state))?;
            let state =
                state.ok_or(DirectX11Error::General("Unable to set depth stencil state"))?;
            ctx.OMSetDepthStencilState(&state, 1);
            Ok(())
        }
    }
}
