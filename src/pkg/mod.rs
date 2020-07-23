pub trait Pkg {
    pub fn on_import() -> Self;
    pub fn static_methods() -> HashMap<String, Box<Fn(Pkg)>>;
}