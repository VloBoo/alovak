pub trait Surface<T> {
    fn create_surface() -> Result<T, String>;
}
