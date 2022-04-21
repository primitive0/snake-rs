// #![windows_subsystem = "windows"]

extern crate gl;
extern crate glfw;
extern crate rand;

mod math;
mod renderer;

use gl::types::*;
use glfw::{Action, Context, Key, OpenGlProfileHint, WindowEvent};
use math::*;
use rand::Rng;
use renderer::*;
use std::collections::VecDeque;
use std::ops::{Add, Neg, Sub};
use std::os::raw::*;
use std::ptr;
use std::time::{Duration, Instant};
use std::{mem, thread};

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 800;

const VERTEX_SHADER_SRC: &str = include_str!("../assets/vertex.glsl");
const FRAGMENT_SHADER_SRC: &str = include_str!("../assets/fragment.glsl");

const SQUARE_COLOR: Vector4f = Vector4f::new(0.26, 0.28, 0.32, 1.0);
const SNAKE_PART_COLOR: Vector4f = Vector4f::new(1.0, 1.0, 1.0, 1.0);
const FRUIT_COLOR: Vector4f = Vector4f::new(0.984, 0.11, 0.369, 1.0);

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("Unable to init glfw");

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::Resizable(false));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw
        .create_window(WINDOW_WIDTH, WINDOW_HEIGHT, "snake-rs", glfw::WindowMode::Windowed)
        .expect("Unable to create window");

    window.set_key_polling(true);
    window.make_current();

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let mut field = SnakeGameField::create();

    let mut renderer = SnakeGameRenderer::setup();
    renderer.prepare_renderer();

    loop {
        if window.should_close() {
            break;
        }

        glfw.poll_events();

        let mut pressed_key = None;
        for (_, event) in glfw::flush_messages(&events) {
            if let WindowEvent::Key(key, _, action, _) = event {
                pressed_key = handle_input(key, action).or(pressed_key);
            }
        }

        match pressed_key {
            Some(key) => {
                let snake_direction = match key {
                    GameKey::Up => SnakeDirection::Up,
                    GameKey::Right => SnakeDirection::Right,
                    GameKey::Down => SnakeDirection::Down,
                    GameKey::Left => SnakeDirection::Left,
                    GameKey::Exit => break,
                };

                field.snake.try_change_direction(snake_direction);
            }

            None => {}
        }

        field.handle_snake_fruit_collision();
        field.push_snake();
        if field.check_snake_collision() || field.check_win() {
            break;
        } else {
            renderer.render(&field);
        }

        window.swap_buffers();

        thread::sleep(Duration::from_millis(200));
    }
}

//INPUT

#[derive(Debug)]
enum GameKey {
    Up,
    Right,
    Down,
    Left,
    Exit,
}

fn handle_input(key: Key, action: Action) -> Option<GameKey> {
    if action != Action::Press {
        return Option::None;
    }

    return match key {
        Key::W | Key::Up => Option::Some(GameKey::Up),
        Key::D | Key::Right => Option::Some(GameKey::Right),
        Key::S | Key::Down => Option::Some(GameKey::Down),
        Key::A | Key::Left => Option::Some(GameKey::Left),
        Key::Escape => Option::Some(GameKey::Exit),
        _ => Option::None,
    };
}

struct SnakeGameRenderer {
    square: VertexArrayObject,
    quads: Box<[VertexArrayObject]>,
    shader_program: ShaderProgram,
    color_uniform: UniformLocation,
}

impl SnakeGameRenderer {
    fn setup() -> SnakeGameRenderer {
        let shader_program = create_shader_program();

        let vertices = gen_vertices();
        let (vbo, ebo) = gen_buffer_objects(vertices.as_slice());
        let (square, quads) = gen_vertex_array_objects(&vbo, &ebo);

        let color_uniform = UniformLocation::get(&shader_program, "inColor");

        return SnakeGameRenderer {
            square,
            quads,
            shader_program,
            color_uniform,
        };
    }

    fn prepare_renderer(&self) {
        self.shader_program.use_program();
        set_clear_color(&Vector4f::zeroed()); //black
    }

    fn render(&mut self, field: &SnakeGameField) {
        fn draw_quad() {
            unsafe {
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            }
        }

        const fn get_quad<'a>(quads: &'a [VertexArrayObject], point: &Point) -> &'a VertexArrayObject {
            &quads[(point.x + point.y * 10) as usize]
        }

        let color_uniform = &self.color_uniform;

        clear_color_buffer();

        //RENDER FIELD SQUARE
        self.shader_program.set_uniform_vec4(color_uniform, &SQUARE_COLOR);

        self.square.bind();
        draw_quad();

        //RENDER FRUIT
        let fruit = &field.fruit.0;
        let snake_head = &field.snake.head;

        if *snake_head != *fruit {
            self.shader_program.set_uniform_vec4(color_uniform, &FRUIT_COLOR);

            let fruit_quad = get_quad(&self.quads, fruit);
            fruit_quad.bind();
            draw_quad();
        }

        //RENDER SNAKE
        self.shader_program.set_uniform_vec4(color_uniform, &SNAKE_PART_COLOR);

        let snake_head_quad = get_quad(&self.quads, snake_head);
        snake_head_quad.bind();
        draw_quad();

        for tail_part in field.snake.tail.iter() {
            let part_quad = get_quad(&self.quads, tail_part);
            part_quad.bind();
            draw_quad();
        }
    }
}

#[inline]
fn create_shader_program() -> ShaderProgram {
    let mut vertex_shader = Shader::create(ShaderType::Vertex);
    vertex_shader.src(VERTEX_SHADER_SRC).unwrap();
    vertex_shader.compile().unwrap();

    let mut fragment_shader = Shader::create(ShaderType::Fragment);
    fragment_shader.src(FRAGMENT_SHADER_SRC).unwrap();
    fragment_shader.compile().unwrap();

    let mut shader_program = ShaderProgram::create();
    shader_program.attach(&vertex_shader);
    shader_program.attach(&fragment_shader);
    shader_program.link().unwrap();

    return shader_program;
}

#[inline]
fn gen_vertices() -> Vec<f32> {
    // Vec<f32> - field square, field quads (for snake parts and fruit)

    // Vertices:
    //                \/ OFFSET
    // B1---------C1      B2---------C2
    // |           |      |           |
    // |           |      |           |
    // |           |      |           |
    // A1---------D1      A2---------D2
    //    ^^^^ OBJECT_SIZE

    let width = WINDOW_WIDTH as f32;
    let height = WINDOW_HEIGHT as f32;

    let mut vertices = Vec::<Vector4f>::with_capacity(10 * 10 * 4);

    const OFFSET: f32 = 5.0;
    const OBJECT_SIZE: f32 = 60.0;

    let field_size = OBJECT_SIZE * 10.0 + OFFSET * 9.0;

    let matrix = {
        let field_center = field_size / 2.0;

        let projection = ortho2d(0.0, width, height, 0.0);
        projection.translate(&Vector3f::new(width / 2.0 - field_center, height / 2.0 - field_center, 0.0))
    };

    //square vertices
    vertices.extend([
        Vector4f::new(0.0, 0.0, 0.0, 1.0),
        Vector4f::new(field_size, 0.0, 0.0, 1.0),
        Vector4f::new(0.0, field_size, 0.0, 1.0),
        Vector4f::new(field_size, field_size, 0.0, 1.0),
    ]);

    for j in (0..10).map(|x| x as f32) {
        for i in (0..10).map(|x| x as f32) {
            let b_vertex = {
                let start = OBJECT_SIZE + OFFSET;
                let x = i * start;
                let y = j * start;
                Vector4f::new(x, y, 0.0, 1.0)
            };

            let c_vertex = {
                let x = b_vertex.x + OBJECT_SIZE;
                let y = b_vertex.y;
                Vector4f::new(x, y, 0.0, 1.0)
            };

            let a_vertex = {
                let x = b_vertex.x;
                let y = b_vertex.y + OBJECT_SIZE;
                Vector4f::new(x, y, 0.0, 1.0)
            };

            let d_vertex = {
                let x = c_vertex.x;
                let y = a_vertex.y;
                Vector4f::new(x, y, 0.0, 1.0)
            };

            vertices.extend([b_vertex, c_vertex, a_vertex, d_vertex])
        }
    }

    vertices
        .into_iter()
        .map(|vec| vec * &matrix)
        .flat_map(|vec| [vec.x, vec.y])
        .collect::<Vec<f32>>()
}

#[inline]
fn gen_buffer_objects(vertices: &[f32]) -> (BufferObject, BufferObject) {
    let vbo = BufferObject::gen();
    vbo.bind(BufferTarget::ArrayBuffer);

    unsafe {
        gl::BufferData(
            BufferTarget::ArrayBuffer.into_raw(),
            (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            vertices.as_ptr() as *const c_void,
            GlDrawType::Static.into_raw(),
        );
    }

    let indices: [u32; 6] = [
        0, 1, 2, //
        1, 2, 3, //
    ];

    let ebo = BufferObject::gen();
    ebo.bind(BufferTarget::ElementArrayBuffer);

    unsafe {
        gl::BufferData(
            BufferTarget::ElementArrayBuffer.into_raw(),
            (indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
            indices.as_ptr() as *const c_void,
            GlDrawType::Static.into_raw(),
        );
    }

    unbind_buffer_object(BufferTarget::ArrayBuffer);
    unbind_buffer_object(BufferTarget::ElementArrayBuffer);

    (vbo, ebo)
}

#[inline]
fn gen_vertex_array_objects(vbo: &BufferObject, ebo: &BufferObject) -> (VertexArrayObject, Box<[VertexArrayObject]>) {
    vbo.bind(BufferTarget::ArrayBuffer);

    const VEC2_SIZE: GLsizei = (2 * mem::size_of::<GLfloat>()) as GLsizei;

    let field_square = {
        let vao = VertexArrayObject::gen();
        vao.bind();

        unsafe {
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                VEC2_SIZE,
                ptr::null(), //zero offset
            );
            gl::EnableVertexAttribArray(0);

            ebo.bind(BufferTarget::ElementArrayBuffer);
        }

        vao
    };

    let field_quads = {
        let mut quads = Vec::<VertexArrayObject>::new();

        for i in 0..(10 * 10) {
            let vao = VertexArrayObject::gen();
            vao.bind();

            unsafe {
                gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, VEC2_SIZE, ((i + 1) * 4 * VEC2_SIZE) as *const _);
                gl::EnableVertexAttribArray(0);

                ebo.bind(BufferTarget::ElementArrayBuffer);
            }

            quads.push(vao);
        }

        unbind_vao(); //unbind last VAO

        unbind_buffer_object(BufferTarget::ArrayBuffer);
        unbind_buffer_object(BufferTarget::ElementArrayBuffer);

        quads
    };

    (field_square, field_quads.into_boxed_slice())
}

//GAME

#[derive(Eq, PartialEq, Clone)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    const fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    const fn origin() -> Point {
        Point::new(0, 0)
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Neg for Point {
    type Output = Point;

    fn neg(self) -> Self::Output {
        Point { x: -self.x, y: -self.y }
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

#[derive(Eq, PartialEq, Debug)]
enum SnakeDirection {
    Up,
    Right,
    Down,
    Left,
}

struct Snake {
    head: Point,
    tail: VecDeque<Point>,
    direction: SnakeDirection,
    ate_fruit: bool,
}

impl Snake {
    fn try_change_direction(&mut self, direction: SnakeDirection) {
        match self.direction {
            SnakeDirection::Up if direction == SnakeDirection::Down => return,
            SnakeDirection::Right if direction == SnakeDirection::Left => return,
            SnakeDirection::Down if direction == SnakeDirection::Up => return,
            SnakeDirection::Left if direction == SnakeDirection::Right => return,

            _ => {
                self.direction = direction;
            }
        }
    }

    #[inline]
    fn ate_fruit(&mut self) -> bool {
        let ate_fruit = self.ate_fruit;
        self.ate_fruit = false;
        return ate_fruit;
    }
}

struct Fruit(Point);

impl Fruit {
    fn random_from_field(field: &SnakeGameField) -> Fruit {
        let snake = &field.snake;
        'generation_loop: loop {
            let fruit = Fruit::random((0, field.size_x), (0, field.size_y));

            if snake.head == fruit.0 {
                continue;
            }

            for tail_part in &snake.tail {
                if *tail_part == fruit.0 {
                    continue 'generation_loop;
                }
            }

            return fruit;
        }
    }

    fn random(x_bounds: (i32, i32), y_bounds: (i32, i32)) -> Fruit {
        let mut random = rand::thread_rng();
        let x = random.gen_range((x_bounds.0)..(x_bounds.1));
        let y = random.gen_range((y_bounds.0)..(y_bounds.1));
        return Fruit(Point::new(x, y));
    }
}

struct SnakeGameField {
    size_x: i32,
    size_y: i32,
    snake: Snake,
    fruit: Fruit,
}

impl SnakeGameField {
    fn create() -> SnakeGameField {
        let size_x = 10;
        let size_y = 10;

        return SnakeGameField {
            size_x,
            size_y,
            snake: Snake {
                head: Point::origin(),
                tail: VecDeque::new(),
                direction: SnakeDirection::Right,
                ate_fruit: false,
            },
            fruit: Fruit::random((1, size_x), (1, size_y)),
        };
    }

    fn handle_snake_fruit_collision(&mut self) {
        if self.snake.head == self.fruit.0 {
            self.snake.ate_fruit = true;
            self.fruit = Fruit::random_from_field(self);
        }
    }

    fn push_snake(&mut self) {
        let snake = &mut self.snake;
        let head = &mut snake.head;

        let old_head = head.clone();
        match snake.direction {
            SnakeDirection::Up => {
                head.y -= 1;
                if head.y == -1 {
                    head.y = self.size_y - 1;
                }
            }

            SnakeDirection::Right => {
                head.x += 1;
                if head.x == self.size_x {
                    head.x = 0;
                }
            }

            SnakeDirection::Down => {
                head.y += 1;
                if head.y == self.size_y {
                    head.y = 0;
                }
            }

            SnakeDirection::Left => {
                head.x -= 1;
                if head.x == -1 {
                    head.x = self.size_x - 1;
                }
            }
        }

        if !snake.tail.is_empty() {
            snake.tail.push_back(old_head);

            if !snake.ate_fruit() {
                snake.tail.pop_front();
            }
        } else if snake.ate_fruit() {
            snake.tail.push_back(old_head);
        }
    }

    fn check_snake_collision(&self) -> bool {
        let head = &self.snake.head;
        for tail_part in &self.snake.tail {
            if *head == *tail_part {
                return true;
            }
        }

        return false;
    }

    fn check_win(&self) -> bool {
        return self.size_x * self.size_y == (self.snake.tail.len() + 1) as i32; //+ HEAD_LENGTH
    }
}
