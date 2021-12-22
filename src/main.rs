mod shader;

extern crate gl;
extern crate glfw;

use std::ffi::c_void;
use std::mem;
use std::ptr;
use std::sync::mpsc::Receiver;

use gl::types::GLfloat;
use gl::types::GLsizei;
use gl::types::GLsizeiptr;
use shader::Shader;

use self::glfw::{Action, Context, Key};

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

#[allow(non_snake_case)]
fn main() {
    //Configure
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    //Window
    let (mut window, events) = glfw
        .create_window(
            SCR_WIDTH,
            SCR_HEIGHT,
            "OpenGL window",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window");
    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    //gl: Load all OpenGL function pointers
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (ourShader, VAO) = unsafe {
        let ourShader = Shader::new("src/shaders/shader.vs", "src/shaders/shader.fs");

        let vertices: [f32; 18] = [
            // positions    // colors
            0.5, -0.5, 0.0, 1.0, 0.0, 0.0, // bottom right
            -0.5, -0.5, 0.0, 0.0, 1.0, 0.0, // bottom left
            0.0, 0.5, 0.0, 0.0, 0.0, 1.0, // top
        ];

        let (mut VBO, mut VAO) = (0, 0);
        gl::GenVertexArrays(1, &mut VAO);
        gl::GenBuffers(1, &mut VBO);
        gl::BindVertexArray(VAO);
        gl::BindBuffer(gl::ARRAY_BUFFER, VBO);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            mem::size_of_val(&vertices) as GLsizeiptr,
            vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
        let stride = 6 * mem::size_of::<GLfloat>() as GLsizei;

        //Position
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);

        //Color
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * mem::size_of::<GLfloat>()) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);

        (ourShader, VAO)
    };

    // Render loop
    while !window.should_close() {
        //Events
        process_events(&mut window, &events);

        //Render
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            ourShader.UseProgram();
            gl::BindVertexArray(VAO);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        // GLFW: Swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        window.swap_buffers();
        glfw.poll_events();
    }
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
                gl::Viewport(0, 0, width, height)
            },
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }
            _ => {}
        }
    }
}
