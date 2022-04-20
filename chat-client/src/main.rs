use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use epistle::Epistle;
use std::{
    fs::{create_dir, File},
    io::{self, Write},
    net::TcpStream,
    path::Path,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
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
#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref USERNAME: String = {
        let args: Vec<String> = std::env::args().collect();
        if args.len() != 2 {
            panic!("please provide a single argument as the username")
        }
        let name = &args[1];
        name.to_string()
    };
}

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
        .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
        .split(f.size());

    // Chat logs
    let block = Block::default().title("💬 Chat").borders(Borders::ALL);
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
    f.render_widget(messages, chunks[0]);

    // Chat input
    let width = chunks[1].width.max(3) - 3;
    let scroll = (app.input.cursor() as u16).max(width) - width;
    let input = Paragraph::new(app.input.value())
        .style(if app.is_editing {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        })
        .scroll((0, scroll)) // What is this?
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("⌨️  Say ({})", *USERNAME)),
        );

    f.render_widget(input, chunks[1]);

    // Set input cursor location
    if app.is_editing {
        f.set_cursor(
            chunks[1].x + (app.input.cursor() as u16).min(width) + 1,
            chunks[1].y + 1,
        );
    }
}

fn run(rx: Receiver<ChatMessage>, stream: TcpStream) -> Result<(), io::Error> {
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
    mut stream: TcpStream,
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
                            let input: String = app.input.value().into();
                            const SEND_FILE_COMMAND: &str = "@file ";

                            let msg;

                            if input.starts_with(SEND_FILE_COMMAND) {
                                let filename = Path::new(&input[SEND_FILE_COMMAND.len()..]);
                                let data = std::fs::read(filename).unwrap();
                                msg = Epistle::Document(epistle::Document {
                                    filename: filename.to_str().unwrap().to_string(),
                                    filesize: data.len(),
                                    data,
                                });
                            } else {
                                msg = Epistle::Message(epistle::Message {
                                    username: (*USERNAME).clone(),
                                    content: app.input.value().into(),
                                });
                            }

                            app.input = Input::default();
                            rmp_serde::encode::write(&mut stream, &msg).unwrap();
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
                username: message.username,
                message: message.content,
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

    create_dir(Path::new(DOWNLOAD_PREFIX)).ok();

    let reader_stream = TcpStream::connect("127.0.0.1:4444").expect("Connection failed");
    let writer_stream = reader_stream.try_clone().expect("TcpStream clone failed");

    std::thread::spawn(move || loop {
        let msg;
        {
            msg = rmp_serde::decode::from_read(&reader_stream);
        }

        match msg {
            Ok(msg) => process_epistle(msg, &tx),
            Err(_) => (),
        };

        std::thread::sleep(Duration::from_millis(100));
    });

    let ui_handle = std::thread::spawn(move || run(rx, writer_stream));

    ui_handle.join().ok();

    Ok(())
}
