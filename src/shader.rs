use glow::HasContext;
use std::rc::Rc;

pub struct Shader {
    gl: Rc<glow::Context>,
    id: glow::Program,
}

impl Shader {
    pub fn new(
        gl: Rc<glow::Context>,
        vertex_src: &str,
        fragment_src: &str,
    ) -> Result<Self, String> {
        unsafe {
            let id = gl.create_program()?;

            let vs = gl.create_shader(glow::VERTEX_SHADER)?;
            gl.shader_source(vs, vertex_src);
            gl.compile_shader(vs);
            if !gl.get_shader_compile_status(vs) {
                let err = gl.get_shader_info_log(vs);
                gl.delete_shader(vs);
                return Err(format!("Vertex Shader Error: {err}"));
            }

            let fs = gl.create_shader(glow::FRAGMENT_SHADER)?;
            gl.shader_source(fs, fragment_src);
            gl.compile_shader(fs);
            if !gl.get_shader_compile_status(fs) {
                let err = gl.get_shader_info_log(fs);
                gl.delete_shader(vs);
                gl.delete_shader(fs);
                return Err(format!("Fragment Shader Error: {err}"));
            }

            gl.attach_shader(id, vs);
            gl.attach_shader(id, fs);
            gl.link_program(id);

            gl.delete_shader(vs);
            gl.delete_shader(fs);

            if !gl.get_program_link_status(id) {
                let err = gl.get_program_info_log(id);
                gl.delete_program(id);
                return Err(format!("Program Link Error: {err}"));
            }

            Ok(Self { gl, id })
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.use_program(Some(self.id));
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.use_program(None);
        }
    }

    pub fn set_uniform_1f(&self, name: &str, value: f32) {
        unsafe {
            let loc = self.gl.get_uniform_location(self.id, name);
            if let Some(loc) = loc {
                self.gl.uniform_1_f32(Some(&loc), value);
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.id);
        }
    }
}