use std::process::Command;

use crate::explorer::Explorer;

impl Explorer {
    pub fn open_file(&mut self, clicked_id: u32) {
        let path = {
            let mut ui = self.ui.borrow_mut();
            let element = ui.get_element(clicked_id).unwrap();
            let text = element.get_text_at_pos(1).unwrap();
            self.path.join(text)
        };

        let _ = Command::new("xdg-open")
            .arg(path)
            .spawn()
            .expect("Datei konnte nicht geöffnet werden")
            .wait();
    }

    pub const HOME: &str = "HOME";
    pub const ROOT_PATH: &str = "";
}
