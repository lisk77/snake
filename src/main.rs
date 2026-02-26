use comet::prelude::*;

#[derive(Component)]
struct Snake;

#[derive(Component)]
struct Direction {
    x: i8,
    y: i8,
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

fn setup(app: &mut App, renderer: &mut RenderHandle2D) {
    renderer.init_atlas();

    app.register_component::<Snake>();
    app.register_component::<Direction>();

    app.spawn_bundle(Camera {
        transform: Transform2D::new(),
        camera: Camera2D::new(v2::new(1.0, 1.0), 1.0, 1),
    });

    app.spawn_bundle(SnakeSegment {
        snake: Snake,
        dir: Direction::new(),
        transform: Transform2D::new(),
        render: Render2D::with_texture("res/textures/snake_body.png"),
    });
}

fn update(app: &mut App, renderer: &mut RenderHandle2D, dt: f32) {
    renderer.render_scene_2d(app.scene_mut());
}

fn main() {
    App::new()
        .with_preset(App2D)
        .with_title("Snake")
        .with_clear_color(sRgba::<u8>::from_hex("1f6c1cff"))
        .run::<Renderer2D>(setup, update)
}
