use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use epistle::Epistle;
use std::{
    fs::File,
    io::{self, Write},
    net::TcpStream,
    path::Path,
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc, Mutex,
    },
    time::Duration,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use tui_input::{backend::crossterm as input_backend, Input, InputResponse};

const DOWNLOAD_PREFIX: &str = "Downloads";

struct App {
    messages: Vec<ChatMessage>,
    should_quit: bool,
    input: Input,
    is_editing: bool,
}

struct ChatMessage {
    username: String,
    message: String,
}

fn draw<B: Backend>(app: &App, f: &mut Frame<B>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    // Help messages
    let block = Block::default().title("Block").borders(Borders::ALL);
    f.render_widget(block, chunks[0]);

    // Chat logs
    let block = Block::default().title("Block 2").borders(Borders::ALL);
    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .map(|msg| {
            let color = Style::default().fg(Color::Blue);
            let content = vec![Spans::from(vec![
                Span::styled(format!("{:>9}  ", msg.username), color),
                Span::raw(&msg.message),
            ])];
            ListItem::new(content)
        })
        .collect();
    let messages = List::new(messages).block(block);
    f.render_widget(messages, chunks[1]);

    // Chat input
    let width = chunks[2].width.max(3) - 3;
    let scroll = (app.input.cursor() as u16).max(width) - width;
    let input = Paragraph::new(app.input.value())
        .style(if app.is_editing {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        })
        .scroll((0, scroll)) // What is this?
        .block(Block::default().borders(Borders::ALL).title("Input"));

    f.render_widget(input, chunks[2]);

    // Set input cursor location
    if app.is_editing {
        f.set_cursor(
            chunks[2].x + (app.input.cursor() as u16).min(width) + 1,
            chunks[2].y + 1,
        );
    }
}

fn run(rx: Receiver<ChatMessage>, stream: Arc<Mutex<TcpStream>>) -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App {
        messages: vec![ChatMessage {
            username: "tornado".into(),
            message: "hey".into(),
        }],
        should_quit: false,
        input: Input::default(),
        is_editing: false,
    };

    run_draw_loop(&mut terminal, &mut app, rx, stream)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_draw_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    rx: Receiver<ChatMessage>,
    stream: Arc<Mutex<TcpStream>>,
) -> Result<(), io::Error> {
    loop {
        terminal.draw(|f| draw(&app, f))?;
        std::thread::sleep(Duration::from_micros(100));

        if crossterm::event::poll(Duration::from_micros(10))? {
            if let Event::Key(key) = event::read()? {
                if app.is_editing {
                    let resp = input_backend::to_input_request(Event::Key(key))
                        .and_then(|req| app.input.handle(req));

                    match resp {
                        Some(InputResponse::StateChanged(_)) => {}
                        Some(InputResponse::Submitted) => {
                            let msg = Epistle::Message(app.input.value().into());
                            app.input = Input::default();
                            let stream = &mut *stream.lock().unwrap();
                            println!("before write");
                            rmp_serde::encode::write(stream, &msg).unwrap();
                            println!("after write");
                        }
                        Some(InputResponse::Escaped) => {
                            app.is_editing = false;
                        }
                        None => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Char('i') => app.is_editing = true,
                        _ => {}
                    }
                }
            }
        }

        if let Ok(msg) = rx.try_recv() {
            app.messages.push(msg);
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn process_epistle(msg: Epistle, tx: &SyncSender<ChatMessage>) {
    match msg {
        Epistle::Handshake => println!("Handshake!"),
        Epistle::Message(message) => {
            tx.send(ChatMessage {
                username: "he".into(),
                message,
            })
            .unwrap();
        }
        Epistle::Document(epistle::Document {
            filename,
            filesize: _,
            data,
        }) => {
            let saved_path = Path::new(DOWNLOAD_PREFIX).join(filename);
            println!("Received file: {:?}", &saved_path);
            let mut file = File::create(saved_path).unwrap();

            file.write_all(&data).unwrap();
        }
    }
}

fn main() -> Result<(), io::Error> {
    let (tx, rx) = sync_channel::<ChatMessage>(3);

    let stream = TcpStream::connect("127.0.0.1:4444").expect("Connection failed");
    let stream = Arc::new(Mutex::new(stream));

    let my_stream = stream.clone();
    let socket_handle = std::thread::spawn(move || {
        loop {
            let msg;
            {
                let stream = &mut *my_stream.lock().unwrap();
                msg = rmp_serde::decode::from_read(stream);
            }

            match msg {
                Ok(msg) => process_epistle(msg, &tx),
                Err(_) => (),
            };

            std::thread::sleep(Duration::from_millis(100));
        }
    });

    let my_stream = stream.clone();
    let ui_handle = std::thread::spawn(move || run(rx, my_stream));

    ui_handle.join().ok();
    socket_handle.join().ok();

    Ok(())
}
