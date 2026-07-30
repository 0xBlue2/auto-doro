use bevy::prelude::*;
fn test(mut windows: Query<&mut Window>) {
    let mut w = windows.single_mut();
    w.start_drag_move();
}
fn main() {}
