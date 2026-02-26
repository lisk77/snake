use comet::prelude::*;

fn setup(app: &mut App, renderer: &mut RenderHandle2D) {}

fn update(app: &mut App, renderer: &mut RenderHandle2D, dt: f32) {}

fn main() {
    App::new()
        .with_preset(App2D)
        .with_title("Snake")
        .with_clear_color(sRgba::<u8>::from_hex("1f6c1cff"))
        .run::<Renderer2D>(setup, update)
}
