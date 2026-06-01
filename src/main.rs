mod prelude;
use std::{io::Stdout, path::Path};

use prelude::*;
mod commands;

mod app;
use app::App;
mod helpers;
use helpers::*;

fn spawn_server(
    jar_path: &Path,
    log_tx: mpsc::SyncSender<String>,
    stdin_rx: mpsc::Receiver<String>,
    server_running: Arc<Mutex<bool>>,
) -> io::Result<()> {
    let mut child: Child = Command::new("java")
        .args([
            "-jar",
            jar_path.to_str().expect("Path to str failed."),
            "--nogui",
        ])
        .current_dir(jar_path.parent().unwrap())
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
        for line in reader.lines().map_while(Result::ok) {
            if tx_out.send(line).is_err() {
                break;
            }
        }
    });

    let tx_err = log_tx.clone();
    thread::spawn(move || {
        let reader = BufReader::new(child_stderr);
        for line in reader.lines().map_while(Result::ok) {
            if tx_err.send(line).is_err() {
                break;
            }
        }
    });

    thread::spawn(move || {
        let _ = child.wait();
        *server_running.lock().unwrap() = false;
        let _ = log_tx.send("[mcman] Server process exited.".into());
    });

    Ok(())
}

pub fn input_poll(
    app: &mut App,
    terminal: &Terminal<CrosstermBackend<Stdout>>,
) -> std::io::Result<()> {
    let height = terminal.size()?.height;
    app.sync_auto_scroll(height);
    if event::poll(Duration::from_millis(50))? {
        loop {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.logs.push("─── Shutting down server ───".into());
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

                            assert!(
                                app.cursor_pos <= app.input.len(),
                                "cursor={} len={} input={:?}",
                                app.cursor_pos,
                                app.input.len(),
                                app.input
                            );

                            assert!(
                                app.input.is_char_boundary(app.cursor_pos),
                                "cursor={} len={} input={:?}",
                                app.cursor_pos,
                                app.input.len(),
                                app.input
                            );
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

                        assert!(
                            app.cursor_pos <= app.input.len(),
                            "cursor={} len={} input={:?}",
                            app.cursor_pos,
                            app.input.len(),
                            app.input
                        );

                        assert!(
                            app.input.is_char_boundary(app.cursor_pos),
                            "cursor={} len={} input={:?}",
                            app.cursor_pos,
                            app.input.len(),
                            app.input
                        );
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
    Ok(())
}

fn main() -> io::Result<()> {
    let jar_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "./server.jar".to_string());
    let jar_path = Path::new(&jar_path);

    let server_running: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    let (log_tx, log_rx) = mpsc::sync_channel::<String>(4096);
    let (stdin_tx, stdin_rx) = mpsc::sync_channel::<String>(64);

    match spawn_server(
        jar_path,
        log_tx.clone(),
        stdin_rx,
        Arc::clone(&server_running),
    ) {
        Ok(_) => {
            let _ = log_tx.send(format!(
                "[mcman] Launched: java -jar {:#?} --nogui",
                jar_path
            ));
        }
        Err(e) => {
            let _ = log_tx.send(format!("[error] Core::FailedToStartServer: {}", e));
            let _ = log_tx.send("[error] Is Java installed and is the jar path correct?".into());
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
            if *app.server_running.lock().unwrap() {
                let _ = app.stdin_tx.send("stop".into());

                let start = std::time::Instant::now();

                while *app.server_running.lock().unwrap() {
                    app.flush_logs();

                    terminal.draw(|f| draw(f, &mut app))?;

                    if start.elapsed() > Duration::from_secs(30) {
                        break;
                    }

                    thread::sleep(Duration::from_millis(50));
                }
            }

            disable_raw_mode()?;

            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;

            terminal.show_cursor()?;

            println!("\x1b[33mmcman exited.\x1b[0m");

            return Ok(());
        }

        terminal.draw(|f| draw(f, &mut app))?;
        match input_poll(&mut app, &terminal) {
            Ok(_) => {}
            Err(e) => {
                app.logs.push(format!("[error] Input: {e}"));
            }
        }
    }
}
