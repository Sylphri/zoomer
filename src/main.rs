#![windows_subsystem = "windows"]

use sdl2::event::{Event};
use sdl2::video::{GLProfile, SwapInterval};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use std::mem::size_of;
use gl::types::*;
use std::fs;
use std::process::exit;
use glm::{vec2, vec3, mat4};
use glm::ext::{translate, scale};
use sdl2_sys::{SDL_DisplayMode, SDL_GetMouseState, SDL_GetKeyboardState, SDL_Scancode};
use screenshots::Screen;

fn load_vertex_shader(path: &str) -> Result<GLuint, String> {
    let mut vertex_shader_source = fs::read_to_string(path).unwrap();
    vertex_shader_source.push('\0');
    unsafe {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(vertex_shader, 1, &(vertex_shader_source.as_ptr() as *const i8) as *const *const GLchar, std::ptr::null());
        gl::CompileShader(vertex_shader);

        let mut success = 0;
        let mut info_log = [1u8; 512];
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            gl::GetShaderInfoLog(vertex_shader, 512, std::ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
            return Err(format!("[ERROR]: vertex shader compilation failed\n{}\n",
                       String::from_utf8_lossy(&info_log[..info_log.iter().position(|x| *x == 0).unwrap() - 1])));
        }
        return Ok(vertex_shader);
    }
}

fn load_fragment_shader(path: &str) -> Result<GLuint, String> {
    let mut fragment_shader_source = fs::read_to_string(path).unwrap();
    fragment_shader_source.push('\0');
    unsafe {
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(fragment_shader, 1, &(fragment_shader_source.as_ptr() as *const i8) as *const *const GLchar, std::ptr::null());
        gl::CompileShader(fragment_shader);

        let mut success = 0;
        let mut info_log = [1u8; 512];
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            gl::GetShaderInfoLog(fragment_shader, 512, std::ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
            return Err(format!("[ERROR]: fragment shader compilation failed\n{}\n",
                       String::from_utf8_lossy(&info_log[..info_log.iter().position(|x| *x == 0).unwrap() - 1])));
        }
        return Ok(fragment_shader);
    }
}

fn load_program(vertex_shader_path: &str, fragment_shader_path: &str) -> Result<GLuint, String> {
    let vertex_shader = match load_vertex_shader(vertex_shader_path) {
        Err(err) => return Err(err),
        Ok(shader) => shader,
    };
    let fragment_shader = match load_fragment_shader(fragment_shader_path) {
        Err(err) => return Err(err),
        Ok(shader) => shader,
    };
    
    unsafe {
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        let mut success = 0;
        let mut info_log = [1u8; 512];
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            gl::GetProgramInfoLog(shader_program, 512, std::ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
            return Err(format!("[ERROR]: shader program linking failed\n{}\n",
                       String::from_utf8_lossy(&info_log[..info_log.iter().position(|x| *x == 0).unwrap() - 1])));
        }
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
        return Ok(shader_program);
    }
}

fn main() {
    let screen = Screen::all().unwrap()[0];
    let image = screen.capture().unwrap();
    
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_profile(GLProfile::Core);
    
    let mut display : SDL_DisplayMode = SDL_DisplayMode {
        format: 0,
        w: 0,
        h: 0,
        refresh_rate: 0,
        driverdata: std::ptr::null_mut(),
    };
    unsafe {
        let code = sdl2_sys::SDL_GetDesktopDisplayMode(0, &mut display);
        if code < 0 {
            eprintln!("ERROR: Can't get screen resolution");
            exit(1);
        }
    }
    
    let window = video_subsystem
        .window("OpenGL", display.w as u32, display.h as u32)
        .opengl()
        .fullscreen()
        .build()
        .unwrap();
    let context = window.gl_create_context().unwrap();
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const GLvoid);
    video_subsystem.gl_set_swap_interval(SwapInterval::VSync);

    let mut texture = 0;
    {
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, image.width() as i32, image.height() as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, image.rgba().as_ptr() as *const GLvoid);
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
    }

    let vertices = [
        -1.0f32, -1.0, 0.0, 0.0, 0.0,
        -1.0   ,  1.0, 0.0, 0.0, 1.0,
         1.0   , -1.0, 0.0, 1.0, 0.0,
         1.0   ,  1.0, 0.0, 1.0, 1.0,
    ];

    let shader_program = match load_program("./shaders/vertex.glsl", "./shaders/fragment.glsl") {
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        },
        Ok(program) => program,
    };

    let flashlight_shader = match load_program("./shaders/flashlight_vert.glsl", "./shaders/flashlight_frag.glsl") {
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        },
        Ok(program) => program,
    };
    
    let mut radius = 0.3;
    
    let mut vbo = 0;
    let mut vao = 0;
    unsafe {
        gl::Viewport(0, 0, display.w, display.h);
        
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * size_of::<f32>()) as GLsizeiptr, vertices.as_ptr() as *const GLvoid, gl::STATIC_DRAW);

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 5 * size_of::<f32>() as GLsizei, std::ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 5 * size_of::<f32>() as GLsizei, (3 * size_of::<f32>()) as *const GLvoid);
        gl::EnableVertexAttribArray(1);
        gl::BindVertexArray(0);

        gl::UseProgram(flashlight_shader);
        gl::Uniform1i(gl::GetUniformLocation(flashlight_shader, "texture1\0".as_ptr() as *const i8), 0);
        gl::Uniform1f(gl::GetUniformLocation(flashlight_shader, "radius\0".as_ptr() as *const i8), radius);
        gl::Uniform2f(gl::GetUniformLocation(flashlight_shader, "resolution\0".as_ptr() as *const i8), display.w as f32, display.h as f32);

        gl::UseProgram(shader_program);
        gl::Uniform1i(gl::GetUniformLocation(shader_program, "texture1\0".as_ptr() as *const i8), 0);
    }
    
    let mut moving = false;
    let mut flashlight = false;
    let mut offset = vec2(0.0, 0.0);
    let mut prev_cursor_pos = vec2(0.0, 0.0);
    let mut scale_factor = 1.0;
    
    let mut transform_loc = 0;
    let mut flashlight_transform_loc = 0;
    let mut cursor_pos_loc = 0;
    unsafe {
        gl::UseProgram(flashlight_shader);
        flashlight_transform_loc = gl::GetUniformLocation(flashlight_shader, "transform\0".as_ptr() as *const i8);
        cursor_pos_loc = gl::GetUniformLocation(flashlight_shader, "cursorPos\0".as_ptr() as *const i8);
        gl::UseProgram(shader_program);
        transform_loc = gl::GetUniformLocation(shader_program, "transform\0".as_ptr() as *const i8);
    };
    
    let mut event_pump = sdl.event_pump().unwrap();
    'outer: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'outer,
                Event::KeyDown {keycode, ..} => match keycode {
                    Some(key) => match key{
                        Keycode::Escape => {
                            break 'outer;
                        },
                        Keycode::F => {
                            if flashlight {
                                flashlight = false;
                                unsafe {gl::UseProgram(shader_program);}
                            } else {
                                flashlight = true;
                                unsafe {gl::UseProgram(flashlight_shader);}
                            }
                        }
                        _ => (),
                    },
                    _ => (),
                },
                Event::MouseWheel {y, ..} => {
                    unsafe {
                        let state = SDL_GetKeyboardState(std::ptr::null_mut());
                        if {&*std::ptr::slice_from_raw_parts(state, SDL_Scancode::SDL_SCANCODE_LCTRL as usize + 1)}[SDL_Scancode::SDL_SCANCODE_LCTRL as usize] == 1{
                            radius += 0.05 * y as f32;
                            gl::Uniform1f(gl::GetUniformLocation(flashlight_shader, "radius\0".as_ptr() as *const i8), radius);
                        } else {
                            scale_factor += 0.05 * y as f32;
                            if scale_factor < 0.0 {
                                scale_factor = 0.0;
                            }
                        }
                    }
                },
                Event::MouseButtonDown {mouse_btn, x, y, ..} => {
                    if mouse_btn == MouseButton::Left {
                        moving = true;
                        prev_cursor_pos = vec2(x as f32, y as f32);
                    }
                },
                Event::MouseButtonUp {mouse_btn, ..} => {
                    if mouse_btn == MouseButton::Left {
                        moving = false;
                    }
                }
                _ => (),
            }
        }

        unsafe {
            gl::ClearColor(0.18, 0.18, 0.18, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            
            gl::BindTexture(gl::TEXTURE_2D, texture);
            
            let mut dir = vec2(0.0, 0.0);
            let mut pos = vec2(0.0, 0.0);
            let mut x = 0;
            let mut y = 0;
            SDL_GetMouseState(&mut x, &mut y);
            pos = vec2(x as f32, y as f32);
            if moving {
                dir = pos - prev_cursor_pos;
                dir = vec2(dir.x, -dir.y);
            }
            
            let mut transform = mat4(1.0, 0.0, 0.0, 0.0,
                                     0.0, 1.0, 0.0, 0.0,
                                     0.0, 0.0, 1.0, 0.0,
                                     0.0, 0.0, 0.0, 1.0);
            transform = scale(&transform, vec3(scale_factor, scale_factor, 1.0));
            offset = offset + dir / vec2(display.w as f32, display.h as f32);
            transform = translate(&transform, vec3(offset.x, offset.y, 0.0));
            if moving {
                prev_cursor_pos = pos;
            }

            if flashlight {
                gl::Uniform2f(cursor_pos_loc, (pos.x / display.w as f32) * 2.0 - 1.0, (pos.y / display.h as f32) * 2.0 - 1.0);
                gl::UniformMatrix4fv(flashlight_transform_loc, 1, gl::FALSE, transform.as_array()[0].as_array().as_ptr());
            } else {
                gl::UniformMatrix4fv(transform_loc, 1, gl::FALSE, transform.as_array()[0].as_array().as_ptr());
            }
            
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);

            gl::BindVertexArray(0);
        }
        window.gl_swap_window();
    }
}
