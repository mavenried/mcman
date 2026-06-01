use crate::{commands::COMMANDS, prelude::*};

pub struct App {
    pub logs: Vec<String>,
    pub log_rx: mpsc::Receiver<String>,
    pub server_running: Arc<Mutex<bool>>,

    pub input: String,
    pub cursor_pos: usize,
    pub scroll_offset: usize,
    pub auto_scroll: bool,
    pub completions: Vec<String>,
    pub completion_idx: Option<usize>,
    pub history: Vec<String>,
    pub history_idx: Option<usize>,

    pub stdin_tx: mpsc::SyncSender<String>,
    pub exit: bool,
}

impl App {
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

                ":exit" | ":quit" => {
                    self.logs
                        .push("\x1b[33m─── Shutting down server ───\x1b[0m".into());
                    self.exit = true;
                }

                _ => {
                    self.logs
                        .push("[\x1b[31mERROR\x1b[0m] Command::Invalid\x1b[0m".into());
                }
            }
            return;
        }

        if self.history.last().map(|s| s.as_str()) != Some(&cmd) {
            self.history.push(cmd.clone());
        }
        self.history_idx = None;
        self.logs.push(format!("> {}", cmd));
        let _ = self.stdin_tx.try_send(cmd);
        self.input.clear();
        self.cursor_pos = 0;
        self.completions.clear();
        self.completion_idx = None;
        self.auto_scroll = true;
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

    fn visible_lines(height: u16) -> usize {
        height.saturating_sub(5) as usize
    }

    pub fn scroll_up(&mut self, amount: usize) {
        if self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(amount);
            self.auto_scroll = false;
        }
    }
    pub fn scroll_down(&mut self, amount: usize, height: u16) {
        let visible = Self::visible_lines(height);
        let max_offset = self.logs.len().saturating_sub(visible);
        self.scroll_offset = (self.scroll_offset + amount).min(max_offset);
        if self.scroll_offset >= max_offset {
            self.auto_scroll = true;
        }
    }

    pub fn sync_auto_scroll(&mut self, height: u16) {
        if self.auto_scroll {
            let visible = Self::visible_lines(height);
            self.scroll_offset = self.logs.len().saturating_sub(visible);
        }
    }
}
