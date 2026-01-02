use std::process::Command;

use crate::explorer::Explorer;

impl Explorer {
    fn open_file(&mut self, clicked_id: u32) {
        let path = {
            let ui = self.ui.borrow();
            let element = ui.get_element(clicked_id).unwrap();
            let text = element.get_text_at_pos(1).unwrap();
            self.path.join(text)
        };

        let _ = Command::new("cmd")
            .args(["/C", "start", "", path.to_str().unwrap()])
            .spawn()
            .expect("Datei konnte nicht geöffnet werden")
            .wait();
    }

    pub const HOME: &str = "USERPROFILE";
    pub const ROOT_PATH: &str = "C:/";
}
