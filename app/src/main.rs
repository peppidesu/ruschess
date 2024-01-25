use notan::prelude::*;
use notan::draw::*;

use ruschess_app::state::*;

#[notan_main]
fn main() -> Result<(), String> {
    let window_config = WindowConfig::new()
        .set_title("ruschess")
        .set_size(800, 600)
        .set_vsync(true);
    
    notan::init_with(setup)
        .draw(draw)
        .add_config(DrawConfig)        
        .add_config(window_config)
        .build()
}

fn setup(gfx: &mut Graphics) -> State {

    State::new(gfx)
}

fn draw(gfx: &mut Graphics, state: &mut State) {
    let mut draw = gfx.create_draw();
    draw.clear(Color::BLACK);
    
    draw.triangle((400.0, 100.0), (100.0, 500.0), (700.0, 500.0));
    gfx.render(&draw);
}