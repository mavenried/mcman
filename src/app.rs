use crate::{commands::COMMANDS, prelude::*};

pub struct App {
    pub logs: Vec<String>,
    pub log_rx: mpsc::Receiver<String>,
    pub server_running: Arc<Mutex<bool>>,

    pub input: String,
    pub cursor_pos: usize,
    pub scroll_offset: usize,
    pub max_scroll: usize,
    pub wrapped_logs: Vec<String>,
    pub wrap_width: usize,
    pub auto_scroll: bool,
    pub completions: Vec<String>,
    pub completion_idx: Option<usize>,
    pub history: Vec<String>,
    pub history_idx: Option<usize>,

    pub stdin_tx: mpsc::SyncSender<String>,
    pub exit: bool,
}

impl App {
    pub fn rebuild_wrapped_logs(&mut self, width: usize) {
        if width == 0 {
            return;
        }

        self.wrap_width = width;
        self.wrapped_logs.clear();

        for line in &self.logs {
            let mut current = String::new();

            for word in line.split_whitespace() {
                let extra = if current.is_empty() { 0 } else { 1 };

                if current.len() + word.len() + extra > width {
                    if !current.is_empty() {
                        self.wrapped_logs.push(current);
                    }
                    current = word.to_string();
                } else {
                    if !current.is_empty() {
                        current.push(' ');
                    }
                    current.push_str(word);
                }
            }

            if current.is_empty() {
                self.wrapped_logs.push(String::new());
            } else {
                self.wrapped_logs.push(current);
            }
        }
    }

    pub fn new(
        stdin_tx: mpsc::SyncSender<String>,
        log_rx: mpsc::Receiver<String>,
        server_running: Arc<Mutex<bool>>,
    ) -> Self {
        Self {
            logs: Vec::new(),
            log_rx,
            server_running,
            input: String::new(),
            cursor_pos: 0,
            scroll_offset: 0,
            max_scroll: 0,
            wrapped_logs: Vec::new(),
            wrap_width: 0,
            auto_scroll: true,
            completions: Vec::new(),
            completion_idx: None,
            history: Vec::new(),
            history_idx: None,
            stdin_tx,
            exit: false,
        }
    }

    pub fn flush_logs(&mut self) {
        while let Ok(line) = self.log_rx.try_recv() {
            self.logs
                .push(line.replace('\t', "    ").replace('\r', " "));

            if self.wrap_width > 0 {
                self.rebuild_wrapped_logs(self.wrap_width);
            }
        }
    }

    pub fn send_command(&mut self) {
        let cmd = self.input.trim().to_string();
        if cmd.is_empty() {
            return;
        }
        if cmd.starts_with(':') {
            match cmd.as_str() {
                ":clear" => {
                    self.logs.clear();
                    self.input.clear();
                    self.cursor_pos = 0;
                }

                ":exit" | ":quit" | ":q" => {
                    self.logs.push("[mcman] Shutting down server.".into());
                    self.exit = true;
                    self.input.clear();
                    self.cursor_pos = 0;
                }
                ":save" => {
                    if let Err(e) = std::fs::write("mcman.log", self.logs.join("\n")) {
                        self.logs.push(format!("[error] Save Failed: {e}"));
                    } else {
                        self.logs.push("[mcman] Saved to mcman.log".into());
                    }
                    self.input.clear();
                    self.cursor_pos = 0;
                }
                _ => {
                    self.logs.push("[error] Command::Invalid.".into());
                    self.input.clear();
                    self.cursor_pos = 0;
                }
            }

            if self.wrap_width > 0 {
                self.rebuild_wrapped_logs(self.wrap_width);
            }

            return;
        }

        if self.history.last().map(|s| s.as_str()) != Some(&cmd) {
            self.history.push(cmd.clone());
        }
        self.history_idx = None;
        self.logs.push(format!("[mcman] > {}", cmd));

        if self.wrap_width > 0 {
            self.rebuild_wrapped_logs(self.wrap_width);
        }

        let _ = self.stdin_tx.try_send(cmd);
        self.input.clear();
        self.cursor_pos = 0;
        self.completions.clear();
        self.completion_idx = None;
        self.auto_scroll = true;
        self.scroll_offset = self.max_scroll;
    }

    pub fn update_completions(&mut self) {
        self.completions.clear();
        self.completion_idx = None;

        let raw = if self.input.starts_with('/') {
            &self.input[1..]
        } else {
            &self.input
        };
        let parts: Vec<&str> = raw.splitn(2, ' ').collect();

        if parts.len() == 1 {
            let prefix = parts[0].to_lowercase();
            for (cmd, _) in COMMANDS {
                if cmd.starts_with(prefix.as_str()) {
                    self.completions.push(cmd.to_string());
                }
            }
        } else {
            let cmd_name = parts[0].to_lowercase();
            let sub_prefix = parts[1].to_lowercase();
            if let Some((_, subs)) = COMMANDS.iter().find(|(c, _)| *c == cmd_name.as_str()) {
                for sub in *subs {
                    if sub.starts_with(sub_prefix.as_str()) {
                        self.completions.push(format!("{} {}", cmd_name, sub));
                    }
                }
            }
        }
    }

    pub fn apply_completion(&mut self) {
        if self.completions.is_empty() {
            self.update_completions();
        }
        if self.completions.is_empty() {
            return;
        }
        let idx = match self.completion_idx {
            None => 0,
            Some(i) => (i + 1) % self.completions.len(),
        };
        self.completion_idx = Some(idx);
        let had_slash = self.input.starts_with('/');
        let completed = self.completions[idx].clone();
        self.input = if had_slash {
            format!("/{} ", completed)
        } else {
            format!("{} ", completed)
        };
        self.cursor_pos = self.input.len();
    }

    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let new_idx = match self.history_idx {
            None => self.history.len() - 1,
            Some(0) => 0,
            Some(i) => i - 1,
        };
        self.history_idx = Some(new_idx);
        self.input = self.history[new_idx].clone();
        self.cursor_pos = self.input.len();
        self.completions.clear();
        self.completion_idx = None;
    }

    pub fn history_down(&mut self) {
        match self.history_idx {
            None => {}
            Some(i) if i + 1 >= self.history.len() => {
                self.history_idx = None;
                self.input.clear();
                self.cursor_pos = 0;
            }
            Some(i) => {
                self.history_idx = Some(i + 1);
                self.input = self.history[i + 1].clone();
                self.cursor_pos = self.input.len();
            }
        }
        self.completions.clear();
        self.completion_idx = None;
    }

    pub fn scroll_up(&mut self, amount: usize) {
        if self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(amount);
            self.auto_scroll = false;
        }
    }
    pub fn scroll_down(&mut self, amount: usize, _height: u16) {
        self.scroll_offset = (self.scroll_offset + amount).min(self.max_scroll);

        if self.scroll_offset >= self.max_scroll {
            self.auto_scroll = true;
        }
    }

    pub fn sync_auto_scroll(&mut self, _height: u16) {
        if self.auto_scroll {
            self.scroll_offset = self.max_scroll;
        }
    }
}
