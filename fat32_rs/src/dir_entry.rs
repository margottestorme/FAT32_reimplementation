#[derive(Debug)]
pub struct DirEntry {
    pub name: String,
    pub first_cluster: u32,
    pub size: u32,
    pub is_dir: bool,
}
