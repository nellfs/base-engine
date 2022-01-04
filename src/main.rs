mod macros;
mod shader;

extern crate gl;
extern crate glfw;

use std::ffi::c_void;
use std::ffi::CStr;
use std::mem;
use std::ptr;
use std::sync::mpsc::Receiver;

use cgmath::perspective;
use cgmath::vec3;
use cgmath::Deg;
use cgmath::InnerSpace;
use cgmath::Matrix4;
use cgmath::Point3;
use cgmath::Vector3;
use gl::types::GLfloat;
use gl::types::GLsizei;
use gl::types::GLsizeiptr;
use image::imageops;
use shader::Shader;

use self::glfw::{Action, Context, Key};

const SCALE: u32 = 1;
const SCR_WIDTH: u32 = 800 / SCALE;
const SCR_HEIGHT: u32 = 600 / SCALE;

const CAMERA_UP: Vector3<f32> = Vector3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};
const CAMERA_DOWN: Vector3<f32> = Vector3 {
    x: 0.0,
    y: -1.0,
    z: 0.0,
};

fn main() {
    //Camera
    let mut camera_pos = Point3::new(0.0, 0.0, 3.0);

    let mut camera_front: Vector3<f32> = Vector3 {
        x: 0.0,
        y: 0.0,
        z: -1.0,
    };

    let mut first_mouse = true;
    let mut yaw: f32 = -90.0;
    let mut pitch: f32 = 0.0;
    let mut last_x: f32 = SCR_WIDTH as f32 / 2.0;
    let mut last_y: f32 = SCR_HEIGHT as f32 / 2.0;
    let mut fov: f32 = 45.0;

    //Timing
    let mut delta_time: f32;
    let mut last_frame: f32 = 0.0;

    //Configure
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    //Window
    let (mut window, events) = glfw
        .create_window(SCR_WIDTH, SCR_HEIGHT, "Engine", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");
    window.make_current();
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);

    //Tell GLFW to capture our mouse
    window.set_cursor_mode(glfw::CursorMode::Disabled);

    //gl: Load all OpenGL function pointers
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (our_shader, vbo, vao, texture1, texture2, cube_positions) = unsafe {
        let our_shader = Shader::new("src/shaders/shader.vs", "src/shaders/shader.fs");

        gl::Enable(gl::DEPTH_TEST);

        #[rustfmt::skip]
        let vertices: [f32; 180] = [
             -0.5, -0.5, -0.5,  0.0, 0.0,
              0.5, -0.5, -0.5,  1.0, 0.0,
              0.5,  0.5, -0.5,  1.0, 1.0,
              0.5,  0.5, -0.5,  1.0, 1.0,
             -0.5,  0.5, -0.5,  0.0, 1.0,
             -0.5, -0.5, -0.5,  0.0, 0.0,

             -0.5, -0.5,  0.5,  0.0, 0.0,
              0.5, -0.5,  0.5,  1.0, 0.0,
              0.5,  0.5,  0.5,  1.0, 1.0,
              0.5,  0.5,  0.5,  1.0, 1.0,
             -0.5,  0.5,  0.5,  0.0, 1.0,
             -0.5, -0.5,  0.5,  0.0, 0.0,

             -0.5,  0.5,  0.5,  1.0, 0.0,
             -0.5,  0.5, -0.5,  1.0, 1.0,
             -0.5, -0.5, -0.5,  0.0, 1.0,
             -0.5, -0.5, -0.5,  0.0, 1.0,
             -0.5, -0.5,  0.5,  0.0, 0.0,
             -0.5,  0.5,  0.5,  1.0, 0.0,

              0.5,  0.5,  0.5,  1.0, 0.0,
              0.5,  0.5, -0.5,  1.0, 1.0,
              0.5, -0.5, -0.5,  0.0, 1.0,
              0.5, -0.5, -0.5,  0.0, 1.0,
              0.5, -0.5,  0.5,  0.0, 0.0,
              0.5,  0.5,  0.5,  1.0, 0.0,

             -0.5, -0.5, -0.5,  0.0, 1.0,
              0.5, -0.5, -0.5,  1.0, 1.0,
              0.5, -0.5,  0.5,  1.0, 0.0,
              0.5, -0.5,  0.5,  1.0, 0.0,
             -0.5, -0.5,  0.5,  0.0, 0.0,
             -0.5, -0.5, -0.5,  0.0, 1.0,

             -0.5,  0.5, -0.5,  0.0, 1.0,
              0.5,  0.5, -0.5,  1.0, 1.0,
              0.5,  0.5,  0.5,  1.0, 0.0,
              0.5,  0.5,  0.5,  1.0, 0.0,
             -0.5,  0.5,  0.5,  0.0, 0.0,
             -0.5,  0.5, -0.5,  0.0, 1.0
        ];

        let cube_positions: [Vector3<f32>; 10] = [
            vec3(0.0, 0.0, 0.0),
            vec3(2.0, 5.0, -15.0),
            vec3(-1.5, -2.2, -2.5),
            vec3(-3.8, -2.0, -12.3),
            vec3(2.4, -0.4, -3.5),
            vec3(-1.7, 3.0, -7.5),
            vec3(1.3, -2.0, -2.5),
            vec3(1.5, 2.0, -2.5),
            vec3(1.5, 0.2, -1.5),
            vec3(-1.3, 1.0, -1.5),
        ];

        let (mut vbo, mut vao) = (0, 0);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            &vertices[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );

        let stride = 5 * mem::size_of::<GLfloat>() as GLsizei;

        //Position
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);

        //Texture Coord
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * mem::size_of::<GLfloat>()) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);

        //Load and create textures
        let (mut texture1, mut texture2) = (0, 0);

        //Texture 1
        gl::GenTextures(1, &mut texture1);
        gl::BindTexture(gl::TEXTURE_2D, texture1);

        //Set the texture wrapping parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

        //Set texture filterting parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        //Texture 1 - Image
        let mut img = image::open("assets/dirt.jpg").unwrap().to_rgb8();
        let subimg = imageops::flip_vertical(&mut img);

        let img_width = subimg.width() as i32;
        let img_height = subimg.height() as i32;
        let data: Vec<u8> = subimg.into_raw();

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            img_width,
            img_height,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            &data[0] as *const u8 as *const c_void,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);

        //Texture2
        gl::GenTextures(1, &mut texture2);
        gl::BindTexture(gl::TEXTURE_2D, texture2);

        //Set the texture wrapping parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

        //Set texture filterting parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        //Texture 2 - Image
        let mut img = image::open("assets/awesomeface.png").unwrap().to_rgb8();
        let subimg = imageops::flip_vertical(&mut img);
        let img_width = subimg.width() as i32;
        let img_height = subimg.height() as i32;
        let data: Vec<u8> = subimg.into_raw();

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            img_width,
            img_height,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            &data[0] as *const u8 as *const c_void,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);

        our_shader.useprogram();
        our_shader.set_int(c_str!("texture1"), 0);
        our_shader.set_int(c_str!("texture2"), 1);

        let projection: Matrix4<f32> =
            perspective(Deg(45.0), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
        our_shader.set_mat4(c_str!("projection"), &projection);

        //Out
        (our_shader, vbo, vao, texture1, texture2, cube_positions)
    };

    // Render loop
    while !window.should_close() {
        //Loop
        let current_frame = glfw.get_time() as f32;
        delta_time = current_frame - last_frame;
        last_frame = current_frame;

        //Events
        process_events(
            &events,
            &mut first_mouse,
            &mut last_x,
            &mut last_y,
            &mut yaw,
            &mut pitch,
            &mut camera_front,
            &mut fov,
        );

        //Inputs
        process_input(&mut window, delta_time, &mut camera_pos, &mut camera_front);

        //Render
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            //Bind texture
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture1);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, texture2);

            our_shader.useprogram();

            let view: Matrix4<f32> =
                Matrix4::look_at_rh(camera_pos, camera_pos + camera_front, CAMERA_UP);
            our_shader.set_mat4(c_str!("view"), &view);

            // render boxes
            gl::BindVertexArray(vao);
            for (i, position) in cube_positions.iter().enumerate() {
                // calculate the model matrix for each object and pass it to shader before drawing
                let mut model: Matrix4<f32> = Matrix4::from_translation(*position);
                let angle = 20.0 * i as f32;
                model =
                    model * Matrix4::from_axis_angle(vec3(1.0, 0.3, 0.5).normalize(), Deg(angle));
                our_shader.set_mat4(c_str!("model"), &model);

                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            }
        }

        // GLFW: Swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        window.swap_buffers();
        glfw.poll_events();
    }
    unsafe {
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
    }
}

fn process_events(
    events: &Receiver<(f64, glfw::WindowEvent)>,
    first_mouse: &mut bool,
    last_x: &mut f32,
    last_y: &mut f32,
    yaw: &mut f32,
    pitch: &mut f32,
    camera_front: &mut Vector3<f32>,
    fov: &mut f32,
) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
                gl::Viewport(0, 0, width, height)
            },
            glfw::WindowEvent::CursorPos(xpos, ypos) => {
                let (xpos, ypos) = (xpos as f32, ypos as f32);
                if *first_mouse {
                    *last_x = xpos;
                    *last_y = ypos;
                    *first_mouse = false;
                }

                let mut xoffset = xpos - *last_x;
                let mut yoffset = *last_y - ypos;
                *last_x = xpos;
                *last_y = ypos;

                let sensitivity: f32 = 0.1;
                xoffset *= sensitivity;
                yoffset *= sensitivity;

                *yaw += xoffset;
                *pitch += yoffset;

                if *pitch > 89.0 {
                    *pitch = 89.0;
                }

                if *pitch < -89.0 {
                    *pitch = -89.0;
                }

                let front = Vector3 {
                    x: yaw.to_radians().cos() * pitch.to_radians().cos(),
                    y: pitch.to_radians().sin(),
                    z: yaw.to_radians().sin() * pitch.to_radians().cos(),
                };
                *camera_front = front.normalize();
            }
            glfw::WindowEvent::Scroll(_xoffset, yoffset) => {
                if *fov >= 1.0 && *fov <= 45.0 {
                    *fov -= yoffset as f32;
                }
                if *fov <= 1.0 {
                    *fov = 1.0;
                }
                if *fov >= 45.0 {
                    *fov = 45.0;
                }
            }
            _ => {}
        }
    }
}

fn process_input(
    window: &mut glfw::Window,
    delta_time: f32,
    camera_pos: &mut Point3<f32>,
    camera_front: &mut Vector3<f32>,
) {
    if window.get_key(Key::Escape) == Action::Press {
        window.set_should_close(true)
    }

    let camera_speed = 2.5 * delta_time;

    if window.get_key(Key::W) == Action::Press {
        *camera_pos += camera_speed * *camera_front;
    }
    if window.get_key(Key::S) == Action::Press {
        *camera_pos += -(camera_speed * *camera_front);
    }
    if window.get_key(Key::A) == Action::Press {
        *camera_pos += -(camera_front.cross(CAMERA_UP).normalize() * camera_speed);
    }
    if window.get_key(Key::D) == Action::Press {
        *camera_pos += camera_front.cross(CAMERA_UP).normalize() * camera_speed;
    }
    if window.get_key(Key::Space) == Action::Press {
        *camera_pos += camera_speed * CAMERA_UP;
    }

    if window.get_key(Key::LeftShift) == Action::Press {
        *camera_pos += camera_speed * CAMERA_DOWN;
    }
}
