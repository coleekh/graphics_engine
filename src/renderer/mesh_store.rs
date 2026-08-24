use wgpu::util::DeviceExt;
use crate::scene::{Mesh, MeshHandle, Vertex};

/// GPU-side mesh: vertex + index buffers.
pub struct GpuMesh {
    pub vertex_buf:  wgpu::Buffer,
    pub index_buf:   wgpu::Buffer,
    pub index_count: u32,
}

/// Manages all uploaded meshes, indexed by MeshHandle.
pub struct MeshStore {
    meshes: Vec<GpuMesh>,
}

impl MeshStore {
    pub fn new() -> Self {
        Self { meshes: Vec::new() }
    }

    /// Upload vertex + index data to the GPU and return a handle.
    pub fn upload(&mut self, device: &wgpu::Device, mesh: &Mesh) -> MeshHandle {
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("vertex_buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage:    wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("index_buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage:    wgpu::BufferUsages::INDEX,
        });

        let handle = MeshHandle(self.meshes.len());
        self.meshes.push(GpuMesh {
            vertex_buf,
            index_buf,
            index_count: mesh.indices.len() as u32,
        });

        log::debug!(
            "Uploaded mesh {:?}: {} verts, {} indices",
            handle,
            mesh.vertices.len(),
            mesh.indices.len()
        );

        handle
    }

    /// Retrieve a GPU mesh by handle.
    pub fn get(&self, handle: MeshHandle) -> Option<&GpuMesh> {
        self.meshes.get(handle.0)
    }
}
