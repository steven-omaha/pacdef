use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Frame, Terminal,
};

#[derive(Debug)]
struct Ui<'a> {
    pub backend_names: Vec<&'a str>,
    pub index: usize,
}

impl<'a> Ui<'a> {
    fn new(backend_names: Vec<&'a str>) -> Self {
        let index = 0;
        Self {
            backend_names,
            index,
        }
    }

    fn run(&self) -> Result<()> {
        enable_raw_mode()?;

        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut app = Ui::new(vec!["abc, def"]);
        app.do_your_stuff(&mut terminal)?;

        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    // TODO rename
    fn do_your_stuff<B>(&mut self, terminal: &mut Terminal<B>) -> Result<()>
    where
        B: Backend,
    {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let shift = key.modifiers.contains(KeyModifiers::SHIFT);

                    match key.code {
                        KeyCode::Char('q') => return Ok(()), // TODO "are you sure you want to quit?"
                        KeyCode::Char('E') => return Ok(()), // execute, TODO "are you sure"?

                        KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab if !shift => {
                            self.next_backend();
                        }
                        KeyCode::Left | KeyCode::Char('h') | KeyCode::Tab if shift => {
                            self.previous_backend();
                        }
                        KeyCode::Down | KeyCode::Char('j') => self.next_package(),
                        KeyCode::Up | KeyCode::Char('k') => self.previous_package(),

                        // KeyCode::Char(char) => match char {
                        //     'a' if self.supports_as_dependency => Ok(ReviewIntention::AsDependency),
                        //     'd' => Ok(ReviewIntention::Delete),
                        //     'g' => Ok(ReviewIntention::AssignGroup),
                        //     'i' => Ok(ReviewIntention::Info),
                        //     'q' => Ok(ReviewIntention::Quit),
                        //     's' => Ok(ReviewIntention::Skip),
                        //     _ => Ok(ReviewIntention::Invalid),
                        // },
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw<B>(&self, frame: &mut Frame<B>)
    where
        B: Backend,
    {
        // help at bottom
        // each backend a tab
        // info as popup
        todo!()
    }

    fn next_package(&mut self) {
        todo!()
    }

    fn previous_package(&mut self) {
        todo!()
    }

    fn next_backend(&mut self) {
        todo!()
    }

    fn previous_backend(&mut self) {
        todo!()
    }
}
