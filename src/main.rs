use comet::prelude::*;

#[derive(Component)]
struct Snake;

#[derive(Component)]
struct Controller {
    direction: v2,
    buffered_direction: v2,
}

impl Controller {
    pub fn direction(&self) -> v2 {
        self.direction
    }

    pub fn buffered_direction(&self) -> v2 {
        self.buffered_direction
    }

    pub fn set_direction(&mut self, direction: v2) {
        self.direction = direction;
    }

    pub fn set_buffered_direction(&mut self, buffered_direction: v2) {
        self.buffered_direction = buffered_direction;
    }
}

#[derive(Component)]
struct Grid {
    cell_size: f32,
    cells: u8,
    bounds: f32
}

impl Grid {
    pub fn new(cell_size: f32, cells: u8) -> Self {
        Self {
            cell_size,
            cells,
            bounds: 1.0,
        }
    }

    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    pub fn cells(&self) -> u8 {
        self.cells
    }

    pub fn bounds(&self) -> f32 {
        self.bounds
    }
}

bundle!(Camera {
    transform: Transform2D,
    camera: Camera2D
});

bundle!(SnakeSegment {
    snake: Snake,
    controller: Controller,
    transform: Transform2D,
    render: Render2D
});

bundle!(Field {
    grid: Grid,
    transform: Transform2D,
    collider: Rectangle2D,
    render: Render2D
});

fn setup(app: &mut App, renderer: &mut RenderHandle2D) {
    renderer.init_atlas();

    app.register_component::<Snake>();
    app.register_component::<Controller>();
    app.register_component::<Grid>();
    app.register_component::<Rectangle2D>();

    app.spawn_bundle(Camera {
        transform: Transform2D::new(),
        camera: Camera2D::new(v2::new(1.0, 1.0), 1.0, 1),
    });

    app.spawn_bundle(SnakeSegment {
        snake: Snake,
        controller: Controller {
            direction: v2::new(1.0, 0.0),
            buffered_direction: v2::new(0.0, 0.0),
        },
        transform: Transform2D::new(),
        render: Render2D::new("res/textures/snake_body.png", true, v2::new(1.0, 1.0), 1),
    });

    let cells: u8 = 16;
    let cell_size: f32 = 16.0;

    let mut grid_transform = Transform2D::new();

    // for pixel alignment
    grid_transform.position_mut().set_x(0.5);
    grid_transform.position_mut().set_y(0.5);
    
    let grid_pos = grid_transform.position().as_vec();
    let mut grid_collider = Rectangle2D::new();
    grid_collider.set_position(Position2D::from_vec(grid_pos));

    app.spawn_bundle(Field {
        grid: Grid::new(cell_size, cells),
        transform: grid_transform,
        collider: grid_collider,
        render: Render2D::with_texture("res/textures/field.png")
    });
}

fn update(app: &mut App, renderer: &mut RenderHandle2D, dt: f32) {
    resize_game_camera(app, renderer);
    handle_input(app, v2::ZERO);
    update_snake(app);
    renderer.render_scene_2d(app.scene_mut());
}

fn resize_game_camera(app: &mut App, renderer: &mut RenderHandle2D) {
    let (grid_cells, grid_cell_size) = match app.query::<Grid>().iter().next() {
        Some(g) => (g.cells(), g.cell_size()),
        None => return,
    };

    app.query_mut::<Camera2D>().for_each(|c| {
        let size = renderer.size();
        let scale_factor = renderer.scale_factor() as f32;

        let game_width = grid_cells as f32 * grid_cell_size;
        let game_height = grid_cells as f32 * grid_cell_size;

        let screen_width = size.width as f32 / scale_factor;
        let screen_height = size.height as f32 / scale_factor;

        let mut fit_scale = 1.0;
        for mult in 1..=10 {
            if game_width * mult as f32 <= screen_width && game_height * mult as f32 <= screen_height {
                fit_scale = mult as f32;
            } else {
                break;
            }
        }

        let target_zoom = 10.0 / fit_scale;
        let zoom = target_zoom.round().clamp(1.0, fit_scale);
        c.set_zoom(zoom);
    });
}

fn update_snake(app: &mut App) {
    app.query_mut::<(Transform2D, Controller)>().for_each(|t, c| {
        t.set_rotation(c.direction.angle(&v2::X));
    });
}

fn handle_input(app: &mut App, head_pos: v2) {
    let mut new_direction = v2::ZERO;

    if app.key_held(Key::KeyW) && head_pos.y() != 128.0 {
        new_direction = v2::new(0.0, 1.0);
    } else if app.key_held(Key::KeyS) && head_pos.y() != -128.0 {
        new_direction = v2::new(0.0, -1.0);
    } else if app.key_held(Key::KeyA) && head_pos.x() != -128.0 {
        new_direction = v2::new(-1.0, 0.0);
    } else if app.key_held(Key::KeyD) && head_pos.x() != 128.0 {
        new_direction = v2::new(1.0, 0.0);
    }

    if new_direction == v2::ZERO {
        return;
    }

    app.query_mut::<Controller>().for_each(|c| {
        if new_direction != -c.direction() {
            c.set_direction(new_direction);
        }
    });
}

fn main() {
    App::new()
        .with_preset(App2D)
        .with_title("Snake")
        //.with_clear_color(sRgba::<u8>::from_hex("1f6c1cff"))
        .run::<Renderer2D>(setup, update)
}
