use bevy::prelude::*;

pub fn tile_to_indices(tile_name: &str) -> (usize, usize) {
    let sub_strings: Vec<&str> = tile_name.split_terminator('_').collect();
    (
        sub_strings[1].parse::<usize>().unwrap(),
        sub_strings[2].parse::<usize>().unwrap(),
    )
}

pub fn idx_to_coordinates(row: usize, col: usize) -> (f32, f32) {
    let square_size = 0.05;
    let offset = 0.025;

    let row_offset: f32 = offset + 3.0 * square_size - ((row % 8) as f32) * square_size;
    let col_offset: f32 = -offset - 3.0 * square_size + ((col % 8) as f32) * square_size;
    (row_offset, col_offset)
}




