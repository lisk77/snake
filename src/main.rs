use comet::prelude::*;

#[derive(Component)]
struct Snake;

#[derive(Component)]
struct Direction {
    direction: v2,
    buffered_dir: v2,
}

impl Direction {
    pub fn direction(&self) -> v2 {
        self.direction
    }

    pub fn set_direction(&mut self, direction: v2) {
        self.direction = direction;
    }

    pub fn update(&mut self) {
        self.direction = self.buffered_dir
    }

    pub fn buffered_dir(&self) -> v2 {
        self.buffered_dir
    }

    pub fn set_buffered_dir(&mut self, dir: v2) {
        self.buffered_dir = dir;
    }
}

#[derive(Component)]
struct Grid {
    cell_size: f32,
    cells: u8,
    bounds: f32,
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
    dir: Direction,
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
    app.register_component::<Direction>();
    app.register_component::<Grid>();
    app.register_component::<Rectangle2D>();
    app.register_component::<Timer>();

    let timer_entity = app.new_entity();
    let mut timer_component = Timer::new();

    timer_component.set_interval(0.5);
    app.add_component(timer_entity, timer_component);

    app.add_tick_system(update_timers);

    app.spawn_bundle(Camera {
        transform: Transform2D::new(),
        camera: Camera2D::new(v2::new(1.0, 1.0), 1.0, 1),
    });

    let mut snake_timer = Timer::new();
    snake_timer.set_interval(0.5);

    let mut dir = Direction::new();
    dir.set_buffered_dir(v2::Y);

    app.spawn_bundle(SnakeSegment {
        snake: Snake,
        dir,
        transform: Transform2D::new(),
        render: Render2D::new("res/textures/snake_head.png", true, v2::new(1.0, 1.0), 1),
    });

    let cells: u8 = 16;
    let cell_size: f32 = 16.0;

    app.spawn_bundle(Field {
        grid: Grid::new(cell_size, cells),
        transform: Transform2D::new(),
        collider: Rectangle2D::new(),
        render: Render2D::with_texture("res/textures/field.png"),
    });
}

fn update(app: &mut App, renderer: &mut RenderHandle2D, dt: f32) {
    let head_pos = app
        .query::<Transform2D>()
        .with::<Snake>()
        .iter()
        .next()
        .unwrap()
        .position()
        .as_vec();

    resize_game_camera(app, renderer);
    handle_input(app, head_pos);
    update_snake(app);
    renderer.render_scene_2d(app.scene_mut());
}

fn update_timers(app: &mut App, dt: f32) {
    app.query_mut::<Timer>().for_each(|t| {
        t.update_timer(dt);
    });
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
            if game_width * mult as f32 <= screen_width
                && game_height * mult as f32 <= screen_height
            {
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
    let is_done = app
        .query::<Timer>()
        .iter()
        .next()
        .map(|t| t.is_done())
        .unwrap_or(false);

    if !is_done {
        return;
    }

    update_snake_direction(app);
    update_snake_orientation(app);
    update_snake_textures(app);
    update_snake_position(app);

    app.query_mut::<Timer>().iter().next().unwrap().reset();
}

fn update_snake_direction(app: &mut App) {
    let mut directions = app
        .query::<Direction>()
        .iter()
        .map(|d| d.direction())
        .collect::<Vec<v2>>();
    directions.pop();

    let head_dir = app.query_mut::<Direction>().iter().next().unwrap();
    head_dir.set_direction(head_dir.buffered_dir());

    for (i, d) in app.query_mut::<Direction>().iter().skip(1).enumerate() {
        d.set_direction(directions[i]);
    }
}

fn update_snake_orientation(app: &mut App) {
    app.query_mut::<(Transform2D, Direction)>()
        .for_each(|t, c| {
            let dir = c.direction();
            let angle = dir.y().atan2(dir.x());
            t.set_rotation((angle.to_degrees() - 90.0).to_radians());
        });
}

fn update_snake_textures(app: &mut App) {
    let directions = app
        .query::<Direction>()
        .iter()
        .map(|d| d.direction())
        .collect::<Vec<v2>>();

    let mut renders = app
        .query_mut::<Render2D>()
        .with::<Snake>()
        .iter()
        .collect::<Vec<&mut Render2D>>();
    for i in 0..renders.len() {
        if i == 0 {
            renders[i].set_texture("res/textures/snake_head.png");
            continue;
        }

        if i == renders.len() {
            renders[i].set_texture("res/textures/snake_tail.png");
            continue;
        }

        let det =
            directions[i].x() * directions[i + 1].y() - directions[i].y() * directions[i + 1].x();
        if det < 0.0 {
            renders[i].set_texture("res/textures/snake_turn_left.png");
            continue;
        } else if det > 0.0 {
            renders[i].set_texture("res/textures/snake_turn_right.png");
            continue;
        } else {
            renders[i].set_texture("res/textures/snake_body.png");
        }
    }
}

fn update_snake_position(app: &mut App) {
    let cell_size = app.query::<Grid>().iter().next().unwrap().cell_size();

    app.query_mut::<(Transform2D, Direction)>()
        .with::<Snake>()
        .for_each(|t, d| {
            t.translate(d.direction() * cell_size);
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

    let head_dir = app.query_mut::<Direction>().iter().next().unwrap();
    if new_direction != -head_dir.direction() {
        head_dir.set_buffered_dir(new_direction);
    }
}

fn main() {
    App::new()
        .with_preset(App2D)
        .with_title("Snake")
        .with_clear_color(sRgba::<u8>::from_hex("1f6c1cff"))
        .run::<Renderer2D>(setup, update)
}
