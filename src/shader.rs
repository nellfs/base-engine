use std::{
    ffi::{CStr, CString},
    fs::File,
    io::Read,
    ptr,
};

use cgmath::{Matrix, Matrix4};
use gl::types::{GLchar, GLint};

pub struct Shader {
    pub id: u32,
}

impl Shader {
    pub fn new(vertex_path: &str, fragment_path: &str) -> Shader {
        let mut shader = Shader { id: 0 };

        // Retrive the vertex/fragment source code from filesystem
        let mut v_shaderfile =
            File::open(vertex_path).unwrap_or_else(|_| panic!("Failed to open {}", vertex_path));
        let mut f_shaderfile = File::open(fragment_path)
            .unwrap_or_else(|_| panic!("Failed to open {}", fragment_path));

        let mut vertex_code = String::new();
        let mut fragment_code = String::new();

        v_shaderfile
            .read_to_string(&mut vertex_code)
            .expect("Failed to read vertex shader");
        f_shaderfile
            .read_to_string(&mut fragment_code)
            .expect("Failed to read fragment shader");

        let v_shadercode = CString::new(vertex_code.as_bytes()).unwrap();
        let f_shadercode = CString::new(fragment_code.as_bytes()).unwrap();

        //Compile shaders
        unsafe {
            //Vertex
            let vertex = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vertex, 1, &v_shadercode.as_ptr(), ptr::null());
            gl::CompileShader(vertex);
            shader.checkcompileerros(vertex, "VERTEX");

            //Fragment
            let fragment = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fragment, 1, &f_shadercode.as_ptr(), ptr::null());
            gl::CompileShader(fragment);
            shader.checkcompileerros(fragment, "FRAGMENT");

            //Program
            let id = gl::CreateProgram();
            gl::AttachShader(id, vertex);
            gl::AttachShader(id, fragment);
            gl::LinkProgram(id);
            shader.checkcompileerros(id, "PROGRAM");

            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);
            shader.id = id;
        }
        shader
    }

    //Activate the shader
    pub unsafe fn useprogram(&self) {
        gl::UseProgram(self.id)
    }

    pub unsafe fn set_int(&self, name: &CStr, value: i32) {
        gl::Uniform1i(gl::GetUniformLocation(self.id, name.as_ptr()), value);
    }

    pub unsafe fn set_mat4(&self, name: &CStr, mat: &Matrix4<f32>) {
        gl::UniformMatrix4fv(
            gl::GetUniformLocation(self.id, name.as_ptr()),
            1,
            gl::FALSE,
            mat.as_ptr(),
        );
    }

    ///Utility function for checking shader compilation/linking errors.
    unsafe fn checkcompileerros(&self, shader: u32, type_: &str) {
        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(1024);
        info_log.set_len(1024 - 1); // subtract 1 to skip the trailing null character
        if type_ != "PROGRAM" {
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                gl::GetShaderInfoLog(
                    shader,
                    1024,
                    ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut GLchar,
                );
                println!(
                    "ERROR::SHADER_COMPILATION_ERROR of type: {}\n{}\n \
                          -- --------------------------------------------------- -- ",
                    type_,
                    std::str::from_utf8(&info_log).unwrap()
                );
            }
        } else {
            gl::GetProgramiv(shader, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                gl::GetProgramInfoLog(
                    shader,
                    1024,
                    ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut GLchar,
                );
                println!(
                    "ERROR::PROGRAM_LINKING_ERROR of type: {}\n{}\n \
                          -- --------------------------------------------------- -- ",
                    type_,
                    std::str::from_utf8(&info_log).unwrap()
                );
            }
        }
    }
}
