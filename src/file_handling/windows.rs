use std::process::Command;

use crate::explorer::ExplorerData;

impl ExplorerData {
    pub fn open_file(&mut self, clicked_id: u32) {
        let path = {
            let mut ui = self.ui.borrow_mut();
            let element = ui.get_element(clicked_id).unwrap();
            let element = element.child(1).unwrap();
            let text = element.get_text().unwrap();
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
