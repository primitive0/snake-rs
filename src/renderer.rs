use crate::math::Vector4f;
use gl::types::*;
use std::ffi::{CString, NulError};
use std::mem::MaybeUninit;
use std::ptr::null;

pub type GetInfoFn = unsafe fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar);

fn get_info_log(get_info_fn: GetInfoFn, descriptor: GLuint) -> String {
    let mut info_log = vec![0u8; 512];
    let mut info_log_length: GLsizei = 0;

    unsafe {
        get_info_fn(descriptor, 512, &mut info_log_length, info_log.as_mut_ptr() as *mut _);
    }

    info_log.truncate(info_log_length as usize);
    info_log.shrink_to_fit();

    return String::from_utf8(info_log).unwrap();
}

#[allow(dead_code)]
pub enum GlDrawType {
    Stream,
    Static,
    Dynamic,
}

impl GlDrawType {
    pub const fn into_raw(self) -> GLenum {
        match self {
            GlDrawType::Stream => gl::STREAM_DRAW,
            GlDrawType::Static => gl::STATIC_DRAW,
            GlDrawType::Dynamic => gl::DYNAMIC_DRAW,
        }
    }
}

pub enum ShaderType {
    Vertex,
    Fragment,
}

impl ShaderType {
    pub const fn into_raw(self) -> GLenum {
        match self {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
        }
    }
}

pub struct Shader(GLuint);

impl Shader {
    pub fn create(shader_type: ShaderType) -> Shader {
        let shader = unsafe { gl::CreateShader(shader_type.into_raw()) };
        return Shader(shader);
    }

    pub fn src(&mut self, src: &str) -> Result<(), NulError> {
        let shader_src = CString::new(src)?;

        unsafe {
            gl::ShaderSource(self.0, 1, &shader_src.as_ptr(), null());
        }

        Ok(())
    }

    pub fn compile(&mut self) -> Result<(), String> {
        unsafe {
            gl::CompileShader(self.0);

            let mut success: GLint = 0;
            gl::GetShaderiv(self.0, gl::COMPILE_STATUS, &mut success);

            if success == (gl::FALSE as GLint) {
                return Err(get_info_log(gl::GetShaderInfoLog, self.0));
            }
        }

        Ok(())
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.0);
        }
    }
}

pub struct UniformLocation(GLint);

impl UniformLocation {
    pub fn get(shader_program: &ShaderProgram, name: &str) -> UniformLocation {
        unsafe {
            let name = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(shader_program.0, name.as_ptr());
            return UniformLocation(location);
        }
    }
}

pub struct ShaderProgram(GLuint);

impl ShaderProgram {
    pub fn create() -> ShaderProgram {
        unsafe { ShaderProgram(gl::CreateProgram()) }
    }

    pub fn attach(&mut self, shader: &Shader) {
        unsafe {
            gl::AttachShader(self.0, shader.0);
        }
    }

    pub fn link(&mut self) -> Result<(), String> {
        unsafe {
            gl::LinkProgram(self.0);

            let mut success = MaybeUninit::<GLint>::uninit();
            gl::GetProgramiv(self.0, gl::LINK_STATUS, success.as_mut_ptr());

            if success.assume_init() == gl::FALSE as GLint {
                return Err(get_info_log(gl::GetProgramInfoLog, self.0));
            }
        }

        Ok(())
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.0);
        }
    }

    pub fn set_uniform_vec4(&mut self, location: &UniformLocation, value: &Vector4f) {
        unsafe {
            gl::Uniform4f(location.0, value.x, value.y, value.z, value.w);
        }
    }
}

pub enum BufferTarget {
    ArrayBuffer,
    ElementArrayBuffer,
}

impl BufferTarget {
    pub const fn into_raw(self) -> GLenum {
        match self {
            BufferTarget::ArrayBuffer => gl::ARRAY_BUFFER,
            BufferTarget::ElementArrayBuffer => gl::ELEMENT_ARRAY_BUFFER,
        }
    }
}

pub struct BufferObject(GLuint);

impl BufferObject {
    pub fn gen() -> BufferObject {
        let mut buffer_object = MaybeUninit::<GLuint>::uninit();

        unsafe {
            gl::GenBuffers(1, buffer_object.as_mut_ptr());
            return BufferObject(buffer_object.assume_init());
        }
    }

    pub fn bind(&self, target: BufferTarget) {
        unsafe {
            gl::BindBuffer(target.into_raw(), self.0);
        }
    }

    #[allow(dead_code)]
    pub const fn descriptor(&self) -> GLuint {
        self.0
    }
}

pub struct VertexArrayObject(GLuint);

impl VertexArrayObject {
    pub fn gen() -> VertexArrayObject {
        let mut vao = MaybeUninit::<GLuint>::uninit();

        unsafe {
            gl::GenVertexArrays(1, vao.as_mut_ptr());
            return VertexArrayObject(vao.assume_init());
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.0);
        }
    }

    #[allow(dead_code)]
    pub const fn descriptor(&self) -> GLuint {
        self.0
    }
}

pub fn unbind_vao() {
    unsafe {
        gl::BindVertexArray(0);
    }
}

pub fn unbind_buffer_object(target: BufferTarget) {
    unsafe {
        gl::BindBuffer(target.into_raw(), 0);
    }
}

pub fn set_clear_color(color: &Vector4f) {
    unsafe {
        gl::ClearColor(color.x, color.y, color.z, color.w);
    }
}

pub fn clear_color_buffer() {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}
