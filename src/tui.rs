use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
	backend::{Backend, CrosstermBackend},
	layout::{Constraint, Direction, Layout},
	style::{Color, Modifier, Style},
	widgets::{Block, Borders, Cell, Row, Table, TableState},
	Frame, Terminal,
};
use std::{error::Error, io};

struct App<'a> {
	state: TableState,
	items: &'a Vec<corrator::container::Status>,
}

impl<'a> App<'a> {
	fn new(items: &'a Vec<corrator::container::Status>) -> App<'a> {
		App {
			state: TableState::default(),
			items,
		}
	}
}

pub fn render_tui(config: crate::Config) -> Result<(), Box<dyn Error>> {
	let data = corrator::run(config).expect("Could not process docker commands.");

	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	// create app and run it
	let app = App::new(&data);
	let res = run_app(&mut terminal, app);

	// restore terminal
	disable_raw_mode()?;
	execute!(
		terminal.backend_mut(),
		LeaveAlternateScreen,
		DisableMouseCapture
	)?;
	terminal.show_cursor()?;

	if let Err(err) = res {
		println!("{:?}", err)
	}

	Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
	loop {
		terminal.draw(|f| ui(f, &mut app))?;

		if let Event::Key(key) = event::read()? {
			match key.code {
				KeyCode::Char('q') => return Ok(()),
				_ => {}
			}
		}
	}
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
	let rects = Layout::default()
		.constraints([Constraint::Percentage(100)].as_ref())
		.direction(Direction::Vertical)
		.margin(5)
		.split(f.size());

	let selected_style = Style::default().add_modifier(Modifier::REVERSED);
	let normal_style = Style::default().fg(Color::Black).bg(Color::LightBlue);
	let header_cells = ["Container", "App", "Version", "EOL"]
		.iter()
		.map(|h| Cell::from(*h));

	let header = Row::new(header_cells).style(normal_style);

	let mut rows = vec![];

	for container_status in app.items.iter() {
		rows.push(Row::new([
			Cell::from(container_status.name.clone()),
			Cell::from(""),
			Cell::from(""),
			Cell::from(""),
		]));

		let apps: Vec<Vec<String>> = container_status
			.apps
			.iter()
			.map(|x| {
				let mut data = x.as_vec();
				data.insert(0, String::from(""));

				data
			})
			.collect();

		for app_vec in apps.iter() {
			let cells = app_vec.iter().map(|x| Cell::from(x.clone()));
			rows.push(Row::new(cells));
		}
	}

	let t = Table::new(rows)
		.header(header.clone())
		.block(Block::default().borders(Borders::ALL).title("Containers"))
		.highlight_style(selected_style)
		.highlight_symbol(">> ")
		.widths(&[
			Constraint::Min(20),
			Constraint::Min(20),
			Constraint::Min(20),
			Constraint::Min(20),
		]);
	f.render_stateful_widget(t, rects[0], &mut app.state);
}
