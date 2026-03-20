use super::*;

impl TuiModel {
    pub(super) fn input_height(&self) -> u16 {
        use unicode_width::UnicodeWidthStr;

        let inner_w = self.width.saturating_sub(2) as usize;
        let prompt_w = 4;
        let text_w = inner_w.saturating_sub(prompt_w);
        if text_w <= 2 {
            return 3;
        }

        let visual_lines: usize = self
            .input
            .buffer()
            .split('\n')
            .map(|line| {
                let display_width = UnicodeWidthStr::width(line) + 1;
                if text_w == 0 {
                    1
                } else {
                    (display_width + text_w - 1) / text_w
                }
            })
            .sum();

        let attach_count = self.attachments.len();
        (visual_lines + attach_count + 2).clamp(3, 12) as u16
    }

    pub fn handle_paste(&mut self, text: String) {
        if self.focus != FocusArea::Input {
            self.focus = FocusArea::Input;
            self.input.set_mode(input::InputMode::Insert);
        }

        let trimmed = text.trim();
        if !trimmed.contains('\n')
            && (trimmed.starts_with('/')
                || trimmed.starts_with('~')
                || trimmed.starts_with("C:\\")
                || trimmed.starts_with("D:\\"))
        {
            let expanded = if trimmed.starts_with('~') {
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_default();
                trimmed.replacen('~', &home, 1)
            } else {
                trimmed.to_string()
            };

            if std::path::Path::new(&expanded).is_file() {
                self.attach_file(trimmed);
                return;
            }
        }

        if text.contains('\n') {
            self.input.insert_paste_block(text);
        } else {
            for c in text.chars() {
                self.input.reduce(input::InputAction::InsertChar(c));
            }
        }
    }

    pub(super) fn attach_file(&mut self, path: &str) {
        let expanded = if path.starts_with('~') {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_default();
            path.replacen('~', &home, 1)
        } else {
            path.to_string()
        };

        match std::fs::read_to_string(&expanded) {
            Ok(content) => {
                let size = content.len();
                let filename = std::path::Path::new(&expanded)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| expanded.clone());
                self.attachments.push(Attachment {
                    filename: filename.clone(),
                    content,
                    size_bytes: size,
                });
                self.status_line = format!("Attached: {} ({} bytes)", filename, size);
            }
            Err(e) => {
                self.status_line = format!("Failed to attach '{}': {}", path, e);
                self.last_error = Some(format!("Attach failed: {}", e));
                self.error_active = true;
                self.error_tick = self.tick_counter;
            }
        }
    }
}
