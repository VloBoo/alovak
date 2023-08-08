pub mod render;
pub use render::Render;
fn main() {
    let _render = Render::new().unwrap();//.expect("Не удалось создать объект рендера");
}
