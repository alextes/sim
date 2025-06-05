use crate::render::Viewport;
use crate::world::World;

use crate::location::PointF64;

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
    let clicked_world_coords: PointF64 = viewport.screen_to_world_coords(screen_x, screen_y);

    // Integer world tile coordinate containing the clicked point.
    let clicked_world_tile_x_i32 = clicked_world_coords.x.floor() as i32;
    let clicked_world_tile_y_i32 = clicked_world_coords.y.floor() as i32;

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
