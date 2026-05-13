use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use std::{
    env,
    io::{self, BufRead, BufReader, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

const COMMANDS: &[(&str, &[&str])] = &[
    ("advancement", &["grant", "revoke"]),
    ("attribute", &[]),
    ("ban", &[]),
    ("ban-ip", &[]),
    ("banlist", &["ips", "players"]),
    ("bossbar", &["add", "remove", "list", "set", "get"]),
    ("clear", &[]),
    ("clone", &["from"]),
    ("damage", &[]),
    ("data", &["merge", "get", "remove", "modify"]),
    ("datapack", &["enable", "disable", "list", "create"]),
    ("debug", &["start", "stop", "function"]),
    (
        "defaultgamemode",
        &["survival", "creative", "adventure", "spectator"],
    ),
    ("deop", &[]),
    ("dialog", &["show", "clear"]),
    ("difficulty", &["peaceful", "easy", "normal", "hard"]),
    ("effect", &["clear", "give"]),
    ("enchant", &[]),
    (
        "execute",
        &[
            "run",
            "if",
            "unless",
            "as",
            "at",
            "store",
            "positioned",
            "rotated",
            "facing",
            "align",
            "anchored",
            "in",
            "summon",
            "on",
        ],
    ),
    ("experience", &["add", "set", "query"]),
    (
        "fill",
        &["outline", "hollow", "destroy", "strict", "replace", "keep"],
    ),
    ("fillbiome", &["replace"]),
    ("forceload", &["add", "remove", "query"]),
    ("function", &["with"]),
    (
        "gamemode",
        &["survival", "creative", "adventure", "spectator"],
    ),
    (
        "gamerule",
        &[
            "advance_time",
            "advance_weather",
            "block_drops",
            "block_explosion_drop_decay",
            "command_block_output",
            "command_blocks_work",
            "drowning_damage",
            "elytra_movement_check",
            "ender_pearls_vanish_on_death",
            "entity_drops",
            "fall_damage",
            "fire_damage",
            "fire_spread_radius_around_player",
            "forgive_dead_players",
            "freeze_damage",
            "global_sound_events",
            "immediate_respawn",
            "keep_inventory",
            "lava_source_conversion",
            "limited_crafting",
            "locator_bar",
            "log_admin_commands",
            "max_block_modifications",
            "max_command_forks",
            "max_command_sequence_length",
            "max_entity_cramming",
            "max_snow_accumulation_height",
            "mob_drops",
            "mob_explosion_drop_decay",
            "mob_griefing",
            "natural_health_regeneration",
            "player_movement_check",
            "players_nether_portal_creative_delay",
            "players_nether_portal_default_delay",
            "players_sleeping_percentage",
            "projectiles_can_break_blocks",
            "pvp",
            "raids",
            "random_tick_speed",
            "reduced_debug_info",
            "respawn_radius",
            "send_command_feedback",
            "show_advancement_messages",
            "show_death_messages",
            "spawn_mobs",
            "spawn_monsters",
            "spawn_patrols",
            "spawn_phantoms",
            "spawn_wandering_traders",
            "spawn_wardens",
            "spawner_blocks_work",
            "spectators_generate_chunks",
            "spread_vines",
            "tnt_explodes",
            "tnt_explosion_drop_decay",
            "universal_anger",
            "water_source_conversion",
        ],
    ),
    ("give", &[]),
    ("help", &[]),
    ("item", &["replace", "modify"]),
    ("jfr", &["start", "stop"]),
    ("kick", &[]),
    ("kill", &[]),
    ("list", &["uuids"]),
    ("locate", &["structure", "biome", "poi"]),
    ("loot", &["replace", "insert", "give", "spawn"]),
    ("me", &[]),
    ("msg", &[]),
    ("op", &[]),
    ("pardon", &[]),
    ("pardon-ip", &[]),
    ("particle", &[]),
    ("perf", &["start", "stop"]),
    ("place", &["feature", "jigsaw", "structure", "template"]),
    (
        "playsound",
        &[
            "master", "music", "record", "weather", "block", "hostile", "neutral", "player",
            "ambient", "voice", "ui",
        ],
    ),
    ("random", &["value", "roll", "reset"]),
    ("recipe", &["give", "take"]),
    ("reload", &[]),
    ("return", &["fail", "run"]),
    ("ride", &["mount", "dismount"]),
    ("rotate", &["facing"]),
    ("save-all", &["flush"]),
    ("save-off", &[]),
    ("save-on", &[]),
    ("say", &[]),
    ("schedule", &["function", "clear"]),
    ("scoreboard", &["objectives", "players"]),
    ("seed", &[]),
    ("setblock", &["destroy", "keep", "replace", "strict"]),
    ("setidletimeout", &[]),
    ("setworldspawn", &[]),
    ("spectate", &[]),
    ("spreadplayers", &["respectTeams", "under"]),
    ("spawnpoint", &[]),
    ("stop", &[]),
    ("stopwatch", &["create", "query", "restart", "remove"]),
    (
        "stopsound",
        &[
            "*", "master", "music", "record", "weather", "block", "hostile", "neutral", "player",
            "ambient", "voice", "ui",
        ],
    ),
    ("summon", &[]),
    ("tag", &["add", "remove", "list"]),
    (
        "team",
        &["list", "add", "remove", "empty", "join", "leave", "modify"],
    ),
    ("teammsg", &[]),
    ("teleport", &[]),
    ("tellraw", &[]),
    (
        "test",
        &[
            "run",
            "runmultiple",
            "runthese",
            "runclosest",
            "runthat",
            "runfailed",
            "verify",
            "locate",
            "resetclosest",
            "resetthese",
            "resetthat",
            "clearthat",
            "clearthese",
            "clearall",
            "stop",
            "pos",
            "create",
        ],
    ),
    (
        "tick",
        &["query", "rate", "step", "sprint", "unfreeze", "freeze"],
    ),
    (
        "time",
        &["set", "add", "pause", "resume", "rate", "query", "of"],
    ),
    (
        "title",
        &["clear", "reset", "title", "subtitle", "actionbar", "times"],
    ),
    ("tp", &[]),
    ("transfer", &[]),
    ("trigger", &["add", "set"]),
    ("version", &[]),
    ("waypoint", &["list", "modify"]),
    ("weather", &["clear", "rain", "thunder"]),
    (
        "whitelist",
        &["on", "off", "list", "add", "remove", "reload"],
    ),
    (
        "worldborder",
        &["add", "set", "center", "damage", "get", "warning"],
    ),
    ("w", &[]),
    ("xp", &["add", "set", "query"]),
];

fn classify_line(line: &str) -> Color {
    let l = line.to_lowercase();
    if l.contains("[error]") || l.contains("exception") || l.contains("stacktrace") {
        Color::Red
    } else if l.contains("[warn]") {
        Color::Yellow
    } else if l.contains("joined the game") || l.contains("left the game") {
        Color::Green
    } else {
        Color::White
    }
}

struct App {
    logs: Vec<String>,
    log_rx: mpsc::Receiver<String>,
    server_running: Arc<Mutex<bool>>,

    input: String,
    cursor_pos: usize,
    scroll_offset: usize,
    auto_scroll: bool,
    completions: Vec<String>,
    completion_idx: Option<usize>,
    history: Vec<String>,
    history_idx: Option<usize>,

    stdin_tx: mpsc::SyncSender<String>,
    exit: bool,
}

impl App {
    fn new(
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

    fn flush_logs(&mut self) {
        while let Ok(line) = self.log_rx.try_recv() {
            self.logs.push(line);
        }
    }

    fn send_command(&mut self) {
        let cmd = self.input.trim().to_string();
        if cmd.is_empty() {
            return;
        }
        match cmd.as_str() {
            ":clear" => {
                self.logs.clear();
                self.input.clear();
                self.cursor_pos = 0;
                return;
            }

            ":exit" | ":quit" => {
                self.exit = true;
                return;
            }

            _ => {}
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

    fn update_completions(&mut self) {
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

    fn apply_completion(&mut self) {
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

    fn history_up(&mut self) {
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

    fn history_down(&mut self) {
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

    fn scroll_up(&mut self, amount: usize) {
        if self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(amount);
            self.auto_scroll = false;
        }
    }

    fn scroll_down(&mut self, amount: usize, height: u16) {
        let visible = Self::visible_lines(height);
        let max_offset = self.logs.len().saturating_sub(visible);
        self.scroll_offset = (self.scroll_offset + amount).min(max_offset);
        if self.scroll_offset >= max_offset {
            self.auto_scroll = true;
        }
    }

    fn sync_auto_scroll(&mut self, height: u16) {
        if self.auto_scroll {
            let visible = Self::visible_lines(height);
            self.scroll_offset = self.logs.len().saturating_sub(visible);
        }
    }
}

fn draw(f: &mut Frame, app: &App) {
    let size = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(size);

    draw_logs(f, app, chunks[0]);
    draw_input(f, app, chunks[1]);
}

fn draw_logs(f: &mut Frame, app: &App, area: Rect) {
    let visible = area.height.saturating_sub(2) as usize;
    let total = app.logs.len();
    let start = app.scroll_offset.min(total.saturating_sub(visible));
    let end = (start + visible).min(total);

    let items: Vec<ListItem> = app.logs[start..end]
        .iter()
        .map(|line| {
            let color = classify_line(line);
            ListItem::new(Line::from(Span::styled(
                line.clone(),
                Style::default().fg(color),
            )))
        })
        .collect();

    let scroll_indicator = if app.auto_scroll {
        " [AUTO-SCROLL] ".to_string()
    } else {
        let max_offset = total.saturating_sub(visible);
        format!(" [{}/{}] ", app.scroll_offset + 1, max_offset + 1)
    };

    let running = *app.server_running.lock().unwrap();
    let status = if running {
        "● RUNNING"
    } else {
        "○ STOPPED"
    };
    let status_color = if running { Color::Green } else { Color::Red };

    let title = Line::from(vec![
        Span::styled(
            " Minecraft Console ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(status, Style::default().fg(status_color)),
        Span::styled(scroll_indicator, Style::default().fg(Color::DarkGray)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::DarkGray));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_input(f: &mut Frame, app: &App, area: Rect) {
    let before_cursor = &app.input[..app.cursor_pos];
    let cursor_char = app.input[app.cursor_pos..].chars().next().unwrap_or(' ');
    let after_cursor = if app.cursor_pos < app.input.len() {
        &app.input[app.cursor_pos + cursor_char.len_utf8()..]
    } else {
        ""
    };

    let spans = vec![
        Span::styled(before_cursor, Style::default().fg(Color::White)),
        Span::styled(
            cursor_char.to_string(),
            Style::default().bg(Color::White).fg(Color::Black),
        ),
        Span::styled(after_cursor, Style::default().fg(Color::White)),
    ];

    let completion_hint = if !app.completions.is_empty() {
        let shown: Vec<&str> = app.completions.iter().take(6).map(|s| s.as_str()).collect();
        let extra = if app.completions.len() > 6 {
            format!(" (+{})", app.completions.len() - 6)
        } else {
            String::new()
        };
        format!("  [{}{}]", shown.join("  "), extra)
    } else {
        String::new()
    };

    let title = Line::from(vec![
        Span::styled(" Command", Style::default().fg(Color::Cyan)),
        Span::styled(
            "  Tab=complete  ↑↓=history  PgUp/PgDn=scroll  Ctrl+C=quit",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let content = Line::from(spans);
    let hint_line = Line::from(Span::styled(
        completion_hint,
        Style::default().fg(Color::Yellow),
    ));

    let full_text = if app.completions.is_empty() {
        ratatui::text::Text::from(content)
    } else {
        ratatui::text::Text::from(vec![content, hint_line])
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Cyan));

    let widget = Paragraph::new(full_text)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(widget, area);
}

fn spawn_server(
    jar_path: &str,
    log_tx: mpsc::SyncSender<String>,
    stdin_rx: mpsc::Receiver<String>,
    server_running: Arc<Mutex<bool>>,
) -> io::Result<()> {
    let mut child: Child = Command::new("java")
        .args(["-jar", jar_path, "--nogui"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    *server_running.lock().unwrap() = true;

    let child_stdin: ChildStdin = child.stdin.take().unwrap();
    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    thread::spawn(move || {
        let mut stdin = child_stdin;
        for cmd in stdin_rx {
            let line = format!("{}\n", cmd);
            if stdin.write_all(line.as_bytes()).is_err() {
                break;
            }
            let _ = stdin.flush();
        }
    });

    let tx_out = log_tx.clone();
    thread::spawn(move || {
        let reader = BufReader::new(child_stdout);
        for line in reader.lines().flatten() {
            if tx_out.send(line).is_err() {
                break;
            }
        }
    });

    let tx_err = log_tx.clone();
    thread::spawn(move || {
        let reader = BufReader::new(child_stderr);
        for line in reader.lines().flatten() {
            if tx_err.send(line).is_err() {
                break;
            }
        }
    });

    thread::spawn(move || {
        let _ = child.wait();
        *server_running.lock().unwrap() = false;
        let _ = log_tx.send("─── Server process exited ───".into());
    });

    Ok(())
}

fn main() -> io::Result<()> {
    let jar_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "server.jar".to_string());

    let server_running: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    let (log_tx, log_rx) = mpsc::sync_channel::<String>(4096);
    let (stdin_tx, stdin_rx) = mpsc::sync_channel::<String>(64);

    match spawn_server(
        &jar_path,
        log_tx.clone(),
        stdin_rx,
        Arc::clone(&server_running),
    ) {
        Ok(_) => {
            let _ = log_tx.send(format!("─── Launched: java -jar {} --nogui ───", jar_path));
        }
        Err(e) => {
            let _ = log_tx.send(format!("[ERROR] Failed to start server: {}", e));
            let _ = log_tx.send("Is Java installed and is the jar path correct?".into());
        }
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(stdin_tx, log_rx, Arc::clone(&server_running));

    loop {
        app.flush_logs();

        if app.exit {
            let _ = app.stdin_tx.try_send("stop".into());
            thread::sleep(Duration::from_millis(300));
            app.flush_logs();
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;
            println!("Goodbye!");
            return Ok(());
        }

        let height = terminal.size()?.height;
        app.sync_auto_scroll(height);
        terminal.draw(|f| draw(f, &app))?;

        if event::poll(Duration::from_millis(50))? {
            loop {
                match event::read()? {
                    Event::Key(key) => match key.code {
                        KeyCode::Char('c')
                            if key.modifiers.contains(KeyModifiers::CONTROL) || app.exit =>
                        {
                            app.exit = true;
                        }
                        KeyCode::Enter => app.send_command(),
                        KeyCode::Tab => app.apply_completion(),
                        KeyCode::Up => app.history_up(),
                        KeyCode::Down => app.history_down(),
                        KeyCode::PageUp => app.scroll_up(10),
                        KeyCode::PageDown => app.scroll_down(10, height),
                        KeyCode::Left => {
                            if app.cursor_pos > 0 {
                                let mut pos = app.cursor_pos - 1;
                                while !app.input.is_char_boundary(pos) {
                                    pos -= 1;
                                }
                                app.cursor_pos = pos;
                            }
                        }
                        KeyCode::Right => {
                            if app.cursor_pos < app.input.len() {
                                let c = app.input[app.cursor_pos..].chars().next().unwrap();
                                app.cursor_pos += c.len_utf8();
                            }
                        }
                        KeyCode::Home => app.cursor_pos = 0,
                        KeyCode::End => app.cursor_pos = app.input.len(),
                        KeyCode::Backspace => {
                            if app.cursor_pos > 0 {
                                let mut pos = app.cursor_pos - 1;
                                while !app.input.is_char_boundary(pos) {
                                    pos -= 1;
                                }
                                app.input.remove(pos);
                                app.cursor_pos = pos;
                                app.completion_idx = None;
                                app.completions.clear();
                            }
                        }
                        KeyCode::Delete => {
                            if app.cursor_pos < app.input.len() {
                                app.input.remove(app.cursor_pos);
                                app.completion_idx = None;
                                app.completions.clear();
                            }
                        }
                        KeyCode::Char(c) => {
                            app.input.insert(app.cursor_pos, c);
                            app.cursor_pos += c.len_utf8();
                            app.completion_idx = None;
                            app.update_completions();
                        }
                        _ => {}
                    },
                    Event::Mouse(mouse) => match mouse.kind {
                        MouseEventKind::ScrollUp => app.scroll_up(3),
                        MouseEventKind::ScrollDown => app.scroll_down(3, height),
                        _ => {}
                    },
                    Event::Resize(_, _) => {}
                    _ => {}
                }

                if !event::poll(Duration::from_millis(0))? {
                    break;
                }
            }
        }
    }
}
