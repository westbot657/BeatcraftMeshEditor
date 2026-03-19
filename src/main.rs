// Control Scheme
//   Left-click             select vertex (shift = multi-select)
//   Left-drag (empty)      orbit
//   Shift+Left-click       add/remove select
//   Shift+Left-drag        marquee select
//   Right-drag             pan
//   Scroll                 zoom
//   Left-drag (vertex)     move along viewport-parallel plane
//   W                      wireframe toggle
//   G                      grid toggle
//   C                      vertex dots toggle
//   V (part-edit)          spawn vertex at cursor
//   E                      assembly <-> part-edit mode
//   [ / ]                  cycle active part
//   N                      create/remove triangle from selection
//   X                      flip winding of selected triangles
//   Ctrl+Z / Ctrl+Shift+Z  undo / redo
//   Ctrl+S                 save JSON
//   Ctrl+Shift+S           optimized save
//   Escape                 deselect all

pub mod data;
pub mod easing;
pub mod light_mesh;
pub mod math_interp;
pub mod render;
pub mod editor;

pub fn main() {

}
