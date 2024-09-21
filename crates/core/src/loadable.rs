pub enum BackgroundLoadable<T> {
    Loading,
    Loaded(T),
}
