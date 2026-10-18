mod mesh;
mod shader;

use glow::HasContext;
use glutin::context::NotCurrentGlContext;
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::prelude::*;
use glutin_winit::{DisplayBuilder, GlWindow};
use mesh::{Mesh, Vertex};
use shader::Shader;
use std::rc::Rc;
use std::time::Instant;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const VERTEX_SHADER_SRC: &str = r#"#version 330 core
layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec3 a_color;

out vec3 v_color;

void main() {
    v_color = a_color;
    gl_Position = vec4(a_pos, 1.0);
}
"#;

const FRAGMENT_SHADER_SRC: &str = r#"#version 330 core
in vec3 v_color;
out vec4 FragColor;

uniform float u_time;

void main() {
    // Subtle pulsating effect using uniform time
    float pulse = (sin(u_time * 3.0) + 1.0) * 0.5;
    FragColor = vec4(v_color * pulse, 1.0);
}
"#;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window_builder = WindowBuilder::new()
        .with_title("HuWiEng OpenGL Engine - Indexed Quad")
        .with_inner_size(LogicalSize::new(1280.0, 720.0));

    let template = glutin::config::ConfigTemplateBuilder::new().with_alpha_size(8);
    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    let (window, gl_config) = display_builder
        .build(&event_loop, template, |configs| {
            configs
                .into_iter()
                .max_by_key(|c| c.num_samples())
                .expect("No valid GL configuration found")
        })
        .unwrap();

    let window = window.expect("Failed to create window");

    let context_attributes = glutin::context::ContextAttributesBuilder::new()
        .with_context_api(glutin::context::ContextApi::OpenGl(Some(
            glutin::context::Version::new(3, 3),
        )))
        .build(None);

    let not_current_gl_context = unsafe {
        gl_config
            .display()
            .create_context(&gl_config, &context_attributes)
            .expect("Failed to create GL context")
    };

    let attrs = window.build_surface_attributes(Default::default());
    let gl_surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &attrs)
            .expect("Failed to create surface")
    };

    let gl_context = not_current_gl_context
        .make_current(&gl_surface)
        .expect("Failed to make GL context current");

    let gl = Rc::new(unsafe {
        glow::Context::from_loader_function_cstr(|s| {
            gl_config.display().get_proc_address(s)
        })
    });

    // -------------------------------------------------------------
    // Resources: Shader & Mesh (Quad with 4 vertices + 6 indices)
    // -------------------------------------------------------------
    let shader = Shader::new(gl.clone(), VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC)
        .expect("Shader failed to compile");

    #[rustfmt::skip]
    let vertices = [
        Vertex::new([ 0.5,  0.5, 0.0], [1.0, 0.2, 0.2]), // Top-right (reddish)
        Vertex::new([ 0.5, -0.5, 0.0], [0.2, 1.0, 0.2]), // Bottom-right (greenish)
        Vertex::new([-0.5, -0.5, 0.0], [0.2, 0.2, 1.0]), // Bottom-left (blueish)
        Vertex::new([-0.5,  0.5, 0.0], [1.0, 1.0, 0.2]), // Top-left (yellowish)
    ];

    #[rustfmt::skip]
    let indices: [u32; 6] = [
        0, 1, 3, // First triangle
        1, 2, 3, // Second triangle
    ];

    let quad = Mesh::new(gl.clone(), &vertices, &indices).expect("Mesh creation failed");

    let start_time = Instant::now();
    event_loop.set_control_flow(ControlFlow::Poll);

    let _ = event_loop.run(move |event, target| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => target.exit(),
            WindowEvent::Resized(physical_size) => {
                if physical_size.width > 0 && physical_size.height > 0 {
                    gl_surface.resize(
                        &gl_context,
                        std::num::NonZeroU32::new(physical_size.width).unwrap(),
                        std::num::NonZeroU32::new(physical_size.height).unwrap(),
                    );
                    unsafe {
                        gl.viewport(0, 0, physical_size.width as i32, physical_size.height as i32);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                unsafe {
                    gl.clear_color(0.08, 0.08, 0.12, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                }

                let elapsed = start_time.elapsed().as_secs_f32();
                shader.bind();
                shader.set_uniform_1f("u_time", elapsed);
                quad.draw();
                shader.unbind();

                gl_surface.swap_buffers(&gl_context).unwrap();
            }
            _ => (),
        },
        Event::AboutToWait => {
            window.request_redraw();
        }
        _ => (),
    });
}