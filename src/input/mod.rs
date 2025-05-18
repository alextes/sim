use crate::render::Viewport;
use crate::render::TILE_PIXEL_WIDTH;
use crate::world::World;

/// Calculates which entity index is at the given screen coordinates, if any.
///
/// # Arguments
/// * `screen_x` - The x screen coordinate of the click.
/// * `screen_y` - The y screen coordinate of the click.
/// * `viewport` - A reference to the current viewport.
/// * `world` - A reference to the game world.
///
/// # Returns
/// * `Option<usize>` - The index of the entity in `world.entities` if one is found at the click, otherwise `None`.
pub fn get_entity_index_at_screen_coords(
    screen_x: i32,
    screen_y: i32,
    viewport: &Viewport,
    world: &World,
) -> Option<usize> {
    // Target pixel dimensions of the viewport on screen.
    let target_screen_pixel_width = viewport.screen_pixel_width;
    let target_screen_pixel_height = viewport.screen_pixel_height;

    // Actual pixel size one full world tile would take on screen, as a float.
    let world_tile_actual_pixel_size_on_screen =
        (TILE_PIXEL_WIDTH as f64 * viewport.zoom).max(0.001);

    // World coordinates (floating point) of the top-left pixel (0,0) of our viewport rendering area.
    let view_world_origin_x = viewport.anchor.x
        - (target_screen_pixel_width as f64 / 2.0) / world_tile_actual_pixel_size_on_screen;
    let view_world_origin_y = viewport.anchor.y
        - (target_screen_pixel_height as f64 / 2.0) / world_tile_actual_pixel_size_on_screen;

    // Clicked world coordinate (float)
    let clicked_world_x_float =
        view_world_origin_x + (screen_x as f64 / world_tile_actual_pixel_size_on_screen);
    let clicked_world_y_float =
        view_world_origin_y + (screen_y as f64 / world_tile_actual_pixel_size_on_screen);

    // Integer world tile coordinate containing the clicked point.
    let clicked_world_tile_x_i32 = clicked_world_x_float.floor() as i32;
    let clicked_world_tile_y_i32 = clicked_world_y_float.floor() as i32;

    world
        .iter_entities()
        .enumerate()
        .find_map(|(idx, entity_id)| {
            world.get_location(entity_id).and_then(|loc| {
                if loc.x == clicked_world_tile_x_i32 && loc.y == clicked_world_tile_y_i32 {
                    Some(idx)
                } else {
                    None
                }
            })
        })
}
