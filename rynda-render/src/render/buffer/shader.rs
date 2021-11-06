use gl::types::*;
use rynda_format::types::{pointermap::PointerColumn, volume::RleVolume};
use std::marker::PhantomData;
use std::mem;
use std::os::raw::c_void;

/// A Shader Storage buffer Object (SSBO)
pub struct ShaderBuffer<T> {
    /// Makes compiler happy about T usage
    phantom: PhantomData<T>,
    /// OpenGL id of the buffer
    pub id: GLuint,
    /// Dynamic size of the buffer
    pub buffer_size: usize,
    /// Amount of elements currently in the buffer
    pub elements: usize,
}

impl<T> ShaderBuffer<T> {
    /// Create SSBO with given preallocated size
    pub fn new(size: usize) -> Self {
        let mut id = 0;
        let buffer_size = mem::size_of::<T>() * size;
        let dummy_data = Vec::<T>::with_capacity(size);

        unsafe {
            gl::GenBuffers(1, &mut id);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, id);
            gl::BufferData(
                gl::SHADER_STORAGE_BUFFER,
                buffer_size as isize,
                dummy_data.as_ptr() as *const c_void,
                gl::DYNAMIC_COPY,
            );
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }

        ShaderBuffer {
            phantom: PhantomData,
            id,
            buffer_size,
            elements: 0,
        }
    }

    /// Create SSBO and fill it with the data from range
    pub fn from(data: &[T]) -> Self {
        let mut id = 0;
        let buffer_size = mem::size_of::<T>() * data.len();

        unsafe {
            gl::GenBuffers(1, &mut id);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, id);
            gl::BufferData(
                gl::SHADER_STORAGE_BUFFER,
                buffer_size as isize,
                data.as_ptr() as *const c_void,
                gl::DYNAMIC_COPY,
            );
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }

        ShaderBuffer {
            phantom: PhantomData,
            id,
            buffer_size,
            elements: data.len(),
        }
    }

    /// Update new data from CPU to GPU
    pub fn upload(&mut self, data: &[T]) {
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.id);

            let d: *mut T = gl::MapBuffer(gl::SHADER_STORAGE_BUFFER, gl::WRITE_ONLY) as *mut T;
            let elements = data.len().min(self.buffer_size);
            std::ptr::copy(data.as_ptr() as *const T, d, elements);
            self.elements = elements.max(self.elements);
            gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER);

            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }
    }
}

impl ShaderBuffer<PointerColumn> {
    /// Create SSBO for RLE volume pointermap
    pub fn from_pointermap(volume: &RleVolume) -> Self {
        let mut id = 0;
        let elements = (volume.xsize * volume.zsize) as usize;
        let buffer_size = mem::size_of::<PointerColumn>() * elements;

        unsafe {
            gl::GenBuffers(1, &mut id);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, id);
            gl::BufferData(
                gl::SHADER_STORAGE_BUFFER,
                buffer_size as isize,
                volume.pointers as *const c_void,
                gl::DYNAMIC_COPY,
            );
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }

        ShaderBuffer {
            phantom: PhantomData,
            id,
            buffer_size,
            elements,
        }
    }

    /// Bind buffer to the given slot
    pub fn bind(&self, slot: u32) {
        unsafe {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, slot, self.id);
        }
    }
}

impl<T> Drop for ShaderBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}
