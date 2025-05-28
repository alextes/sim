use sdl2::image::InitFlag;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;
use tracing::debug;

pub fn setup_sdl() -> (
    Sdl,
    Canvas<Window>,
    sdl2::render::TextureCreator<sdl2::video::WindowContext>,
) {
    let sdl_context = sdl2::init().unwrap();
    debug!("sdl initialized");
    let video_subsystem = sdl_context.video().unwrap();
    let _image_context = sdl2::image::init(InitFlag::PNG).unwrap();

    let window = video_subsystem
        .window("sim", 576, 576)
        .position_centered()
        .resizable()
        .build()
        .unwrap();
    debug!("window created");

    let canvas = window.into_canvas().software().build().unwrap();
    let texture_creator = canvas.texture_creator();

    (sdl_context, canvas, texture_creator)
}
