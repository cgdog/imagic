use log::info;

pub struct HDRLoader {

}

impl HDRLoader {
    pub fn load(&mut self, path: &str) {
        info!("path: {}", path);
    }
}