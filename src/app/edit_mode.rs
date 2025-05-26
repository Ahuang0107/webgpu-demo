use isometric_engine::{Item, Package, Scene};

pub struct EditMode {
    package: Package,
    selected_scene: Option<Scene>,
    selected_box: Option<Item>,
}
