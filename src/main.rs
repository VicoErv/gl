extern crate gl;
extern crate image;
extern crate nalgebra_glm;
extern crate sdl2;

use image::GenericImageView;
use nalgebra_glm as glm;
use std::time::Instant;

struct Window {
    sdl: sdl2::Sdl,
    win: sdl2::video::Window,
    video: sdl2::VideoSubsystem,
}

impl Window {
    pub fn new() -> Self {
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();

        let win = video
            .window("Title", 800, 600)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        Self { sdl, win, video }
    }
}

fn load_image() -> (i32, i32, image::ImageBuffer<image::Rgba<u8>, Vec<u8>>) {
    let path = format!("{}/assets/image.jpg", env!("CARGO_MANIFEST_DIR"));
    let img = image::open(path).unwrap();
    let img = img.flipv();
    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8();

    return (width as i32, height as i32, rgba);
}

fn main() -> Result<(), String> {
    let window: Window = Window::new();
    let Window { win, sdl, video } = &window;

    let _gl_context = win.gl_create_context()?;
    let mut event_pump = sdl.event_pump()?;

    video
        .gl_set_swap_interval(sdl2::video::SwapInterval::VSync)
        .unwrap();

    const INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];

    unsafe {
        gl::load_with(|f_name| video.gl_get_proc_address(f_name) as *const _);
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);

        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        assert_ne!(vao, 0);

        gl::BindVertexArray(vao);

        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);
        assert_ne!(vbo, 0);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        type Vertex = [f32; 8];
        const VERTICES: [Vertex; 4] = [
            [-0.5, -0.5, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0],
            [0.5, -0.5, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0],
            [0.5, 0.5, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0],
            [-0.5, 0.5, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        ];

        gl::BufferData(
            gl::ARRAY_BUFFER,
            size_of_val(&VERTICES) as isize,
            VERTICES.as_ptr().cast(),
            gl::STATIC_DRAW,
        );

        let mut ebo: u32 = 0;
        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);

        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (INDICES.len() * std::mem::size_of::<u32>()) as isize,
            INDICES.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            size_of::<Vertex>().try_into().unwrap(),
            0 as *const _,
        );
        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            size_of::<Vertex>().try_into().unwrap(),
            (3 * std::mem::size_of::<f32>()) as *const _,
        );

        gl::EnableVertexAttribArray(1);

        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            size_of::<Vertex>().try_into().unwrap(),
            (6 * std::mem::size_of::<f32>()) as *const _,
        );

        gl::EnableVertexAttribArray(2);

        let (width, height, rgba) = load_image();
        let mut texture: u32 = 0;
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            width,
            height,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            rgba.as_ptr() as *const _,
        );

        gl::GenerateMipmap(gl::TEXTURE_2D);

        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        assert_ne!(vertex_shader, 0);

        const VERT_SHADER: &str = r#"#version 330 core
            layout (location = 0) in vec3 pos;
            layout (location = 1) in vec3 color;
            layout (location = 2) in vec2 texcoord;

            out vec3 ourColor;
            out vec2 TexCoord;

            uniform mat4 transform;
            void main() {
                gl_Position = transform * vec4(pos, 1.0);
                ourColor = color;
                TexCoord =  texcoord;
            }
        "#;

        gl::ShaderSource(
            vertex_shader,
            1,
            &(VERT_SHADER.as_bytes().as_ptr().cast()),
            &(VERT_SHADER.len().try_into().unwrap()),
        );

        gl::CompileShader(vertex_shader);

        let mut success = 0;
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);

        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl::GetShaderInfoLog(vertex_shader, 1024, &mut log_len, v.as_mut_ptr().cast());

            v.set_len(log_len.try_into().unwrap());
            panic!("Vertex Compile Error: {}", String::from_utf8_lossy(&v));
        }

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        assert_ne!(fragment_shader, 0);

        const FRAG_SHADER: &str = r#"#version 330 core
            in vec3 ourColor;
            in vec2 TexCoord;

            out vec4 final_color;

            uniform sampler2D tex0;

            void main() {
                final_color = texture(tex0, TexCoord) *  vec4(ourColor, 1.0);
            }
        "#;

        gl::ShaderSource(
            fragment_shader,
            1,
            &(FRAG_SHADER.as_bytes().as_ptr().cast()),
            &(FRAG_SHADER.len().try_into().unwrap()),
        );

        gl::CompileShader(fragment_shader);

        let mut success = 0;
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl::GetShaderInfoLog(fragment_shader, 1024, &mut log_len, v.as_mut_ptr().cast());
            v.set_len(log_len.try_into().unwrap());
            panic!("Fragment Compile Error: {}", String::from_utf8_lossy(&v));
        }

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        gl::UseProgram(shader_program);

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture);

        let tex0_loc = gl::GetUniformLocation(shader_program, b"tex0\0".as_ptr() as *const _);
        gl::Uniform1i(tex0_loc, 0);

        let mut success = 0;
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl::GetProgramInfoLog(shader_program, 1024, &mut log_len, v.as_mut_ptr().cast());
            v.set_len(log_len.try_into().unwrap());
            panic!("Program Link error: {}", String::from_utf8_lossy(&v));
        }

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        let start_time = Instant::now();

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    sdl2::event::Event::Quit { .. } => break 'running,
                    _ => {}
                }
            }

            let time = start_time.elapsed().as_secs_f32();
            let scale_factor = time.sin() * 0.25 + 0.75;

            let mut transform = glm::identity::<f32, 4>();
            transform = glm::scale(&transform, &glm::vec3(0.5, 0.5, 0.5));
            transform = glm::rotate(&transform, time, &glm::vec3(0.0, 0.0, 1.0));
            transform = glm::scale(
                &transform,
                &glm::vec3(scale_factor, scale_factor, scale_factor),
            );

            let transform_loc =
                gl::GetUniformLocation(shader_program, b"transform\0".as_ptr() as *const i8);
            gl::UniformMatrix4fv(transform_loc, 1, gl::FALSE, transform.as_ptr());

            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawElements(
                gl::TRIANGLES,
                INDICES.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );

            win.gl_swap_window();
        }
    }

    Ok(())
}
