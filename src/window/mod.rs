pub mod win32;

pub trait Window {
    fn create(title: &str) -> Result<impl Window, &str>;
}
