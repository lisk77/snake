use comet::prelude::*;

#[derive(Component)]
struct Snake;

#[derive(Component)]
struct Apple;

#[derive(Component)]
struct GameOverText;

#[derive(Component)]
struct WinText;

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
}

struct GameState {
    game_over: bool,
}

impl GameState {
    pub fn new() -> Self {
        Self { game_over: false }
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn set_game_over(&mut self, game_over: bool) {
        self.game_over = game_over;
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
    collider: Rectangle2D,
    render: Render2D,
});

bundle!(AppleEntity {
    apple: Apple,
    transform: Transform2D,
    collider: Rectangle2D,
    render: Render2D
});

bundle!(Field {
    grid: Grid,
    transform: Transform2D,
    collider: Rectangle2D,
    render: Render2D
});

bundle!(GameText {
    transform: Transform2D,
    render: Text
});

fn setup(app: &mut App, renderer: &mut RenderHandle2D) {
    renderer.init_atlas();

    app.register_component::<Snake>();
    app.register_component::<Apple>();
    app.register_component::<Direction>();
    app.register_component::<Grid>();
    app.register_component::<Rectangle2D>();
    app.register_component::<Timer>();
    app.register_component::<GameOverText>();
    app.register_component::<WinText>();

    app.load_audio("nom", "res/sounds/nom.wav");
    app.load_audio("bonk", "res/sounds/bonk.wav");

    renderer.load_font("res/fonts/PressStart2P-Regular.ttf", 77.0);

    let cells: u8 = 16;
    let cell_size: f32 = 16.0;
    let grid_size: f32 = (cells as f32) * cell_size;

    let timer_entity = app.new_entity();
    let mut timer_component = Timer::new();

    timer_component.set_interval(0.5);
    app.add_component(timer_entity, timer_component);

    app.add_tick_system(update_timers);

    app.spawn_bundle(Camera {
        transform: Transform2D::new(),
        camera: Camera2D::new(v2::new(1.0, 1.0), 1.0, 1),
    });

    let mut dir = Direction::new();
    dir.set_direction(v2::Y);
    dir.set_buffered_dir(v2::Y);

    app.spawn_bundle(SnakeSegment {
        snake: Snake,
        dir: dir.clone(),
        transform: Transform2D::new(),
        collider: Rectangle2D::with_size(cell_size, cell_size),
        render: Render2D::new("res/textures/snake_head.png", true, v2::new(1.0, 1.0), 1),
    });

    let tail_pos = Position2D::from_vec(v2::new(0.0, -cell_size));
    let tail_transform = Transform2D::with_position(tail_pos.clone());
    let mut tail_collider = Rectangle2D::with_size(cell_size, cell_size);
    tail_collider.set_position(tail_pos);

    let mut tail_dir = Direction::new();
    tail_dir.set_direction(v2::Y);

    app.spawn_bundle(SnakeSegment {
        snake: Snake,
        dir: tail_dir,
        transform: tail_transform,
        collider: tail_collider,
        render: Render2D::new("res/textures/snake_tail.png", true, v2::new(1.0, 1.0), 1),
    });

    app.spawn_bundle(Field {
        grid: Grid::new(cell_size, cells),
        transform: Transform2D::new(),
        collider: Rectangle2D::with_size(grid_size, grid_size),
        render: Render2D::with_texture("res/textures/field.png"),
    });
    
    app.spawn_bundle(AppleEntity {
        apple: Apple,
        transform: Transform2D::new(),
        collider: Rectangle2D::with_size(cell_size, cell_size),
        render: Render2D::new("res/textures/apple.png", true, v2::new(1.0, 1.0), 1),
    });

    let game_over_text = app.spawn_bundle(GameText {
        transform: Transform2D::new(),
        render: Text::new("Game Over!", "res/fonts/PressStart2P-Regular.ttf", 16.0, false, sRgba::<f32>::from_hex("#ff0000ff")),
    });

    app.add_component(game_over_text, GameOverText);

    let win_text = app.spawn_bundle(GameText {
        transform: Transform2D::new(),
        render: Text::new("You Win!", "res/fonts/PressStart2P-Regular.ttf", 16.0, false, sRgba::<f32>::from_hex("#0000ffff")),
    });

    app.add_component(win_text, WinText);

    move_apple(app);
}

fn update(app: &mut App, renderer: &mut RenderHandle2D, _dt: f32) {
    let head_pos = app
        .query::<Transform2D>()
        .with::<Snake>()
        .iter()
        .next()
        .unwrap()
        .position()
        .as_vec();

    resize_game_camera(app, renderer);
    if !snake_out_of_bounds(app) && !is_snake_body_colliding(app) {
        handle_input(app, head_pos);
        update_snake(app);
    }
    else {
        if !app.game_state::<GameState>().unwrap().is_game_over() {
            app.play_audio("bonk", false);
            app.set_volume("bonk", 0.5);
            app.game_state_mut::<GameState>().unwrap().set_game_over(true);
            app.query_mut::<Text>().with::<GameOverText>().for_each(|t| t.set_visibility(true));
        }
    }
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
    update_snake_colliders(app);
    
    if !snake_out_of_bounds(app) && !is_snake_body_colliding(app) {
        update_snake_orientation(app);
        handle_apple_collision(app);
        update_snake_position(app);
        update_snake_textures(app);
    }

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
    head_dir.update();

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
    let snake_size = app.query::<Snake>().iter().count();
    let directions = app
        .query::<Direction>()
        .iter()
        .map(|d| d.direction())
        .collect::<Vec<v2>>();
    
    for (i, (r, t)) in app.query_mut::<(Render2D, Transform2D)>().with::<Snake>().iter().enumerate() {
        if i == 0 {
            r.set_texture("res/textures/snake_head.png");
            continue;
        }

        if i == (snake_size - 1) {
            r.set_texture("res/textures/snake_tail.png");
            t.set_rotation(directions[i-1].y().atan2(directions[i-1].x()) - std::f32::consts::FRAC_PI_2);
            continue;
        }

        r.set_texture("res/textures/snake_body.png");

        let det = directions[i].x() * directions[i-1].y() - directions[i].y() * directions[i-1].x();
        if det > 0.0 {
            r.set_texture("res/textures/snake_turn_left.png");
        } else if det == 0.0 {
            r.set_texture("res/textures/snake_body.png");
        } else {
            r.set_texture("res/textures/snake_turn_right.png");
        }
    }
}

fn update_snake_position(app: &mut App) {
    let cell_size = app.query::<Grid>().iter().next().unwrap().cell_size();
    let positions = app
        .query::<Transform2D>()
        .with::<Snake>()
        .iter()
        .map(|t| t.position().as_vec())
        .collect::<Vec<v2>>();

    for (i, (t, d)) in app
        .query_mut::<(Transform2D, Direction)>()
        .with::<Snake>()
        .iter()
        .enumerate()
    {
        t.set_position(Position2D::from_vec(d.direction * cell_size + positions[i]));
    }
}

fn update_snake_colliders(app: &mut App) {
    let cell_size = app.query::<Grid>().iter().next().unwrap().cell_size();
    let positions = app
        .query::<Transform2D>()
        .with::<Snake>()
        .iter()
        .map(|t| t.position().as_vec())
        .collect::<Vec<v2>>();

    for (i, (r, d)) in app
        .query_mut::<(Rectangle2D, Direction)>()
        .with::<Snake>()
        .iter()
        .enumerate()
    {
        r.set_position(Position2D::from_vec(d.direction * cell_size + positions[i]));
    }
}

fn snake_out_of_bounds(app: &mut App) -> bool {
    let field_collider = app.query::<(Rectangle2D, Grid)>().iter().next().unwrap().0;

    let snake_head_collider = app
        .query::<Rectangle2D>()
        .with::<Snake>()
        .iter()
        .next()
        .unwrap();

    !snake_head_collider.is_colliding(field_collider)
}

fn move_apple(app: &mut App) {
    let cell_size = app.query::<Grid>().iter().next().unwrap().cell_size();
    let half_cells = (app.query::<Grid>().iter().next().unwrap().cells()/2) as i8;
    let snake = app.query::<Transform2D>().with::<Snake>().iter().map(|t| t.position().as_vec()).collect::<Vec<v2>>();

    let x = rand::random_range(-half_cells..half_cells) as f32 * cell_size;
    let y = rand::random_range(-half_cells..half_cells) as f32 * cell_size;
    
    let mut apple_pos = Position2D::from_vec(v2::new(x, y));
    while snake.iter().any(|&s| s == apple_pos.as_vec()) {
        let x = rand::random_range(-half_cells..half_cells) as f32 * cell_size;
        let y = rand::random_range(-half_cells..half_cells) as f32 * cell_size;
        apple_pos = Position2D::from_vec(v2::new(x, y));
    }

    app.query_mut::<(Transform2D, Rectangle2D)>()
        .with::<Apple>()
        .for_each(|t, r| {
            t.set_position(apple_pos.clone());
            r.set_position(apple_pos.clone());
        });
}

fn handle_apple_collision(app: &mut App) {
    let grid_cell_size = app.query::<Grid>().iter().next().unwrap().cell_size();
    let apple_collider = app.query::<Rectangle2D>().with::<Apple>().iter().next();
    let snake_head_collider = app
        .query::<Rectangle2D>()
        .with::<Snake>()
        .iter()
        .next()
        .unwrap();

    snake_head_collider
        .is_colliding(apple_collider.unwrap())
        .then(|| {
            app.play_audio("nom", false);
            app.set_volume("nom", 0.5);

            move_apple(app);

            let (tail_transform, tail_dir) = app.query::<(Transform2D, Direction)>().with::<Snake>().iter().last().unwrap();
            let tail_pos = tail_transform.position().as_vec();
            let new_tail_pos = tail_pos - tail_dir.direction() * grid_cell_size;
            let mut new_tail_dir = Direction::new();
            new_tail_dir.set_direction(tail_dir.direction());

            app.spawn_bundle(SnakeSegment {
                snake: Snake,
                dir: new_tail_dir,
                transform: Transform2D::with_position(Position2D::from_vec(new_tail_pos)),
                collider: Rectangle2D::with_size(16.0, 16.0),
                render: Render2D::new("res/textures/snake_body.png", true, v2::new(1.0, 1.0), 1),
            });
    });
}

fn is_snake_body_colliding(app: &mut App) -> bool {
    let head_collider = app.query::<Rectangle2D>().with::<Snake>().iter().next().unwrap();
    let mut body_colliders = app.query::<Rectangle2D>().with::<Snake>().iter().skip(1);

    body_colliders.any(|b| head_collider.is_colliding(b))
}

fn winning_condition_met(app: &mut App) -> bool {
    let snake_size = app.query::<Snake>().iter().count();
    let grid_cells = app.query::<Grid>().iter().next().unwrap().cells();

    snake_size as u8 == grid_cells * grid_cells
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
        .with_game_state(GameState::new())
        .run::<Renderer2D>(setup, update)
}
