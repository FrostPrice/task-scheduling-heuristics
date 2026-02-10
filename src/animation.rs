use crate::blm::Maquina;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum AnimationEvent {
    TaskGenerated {
        machine_id: usize,
        task_value: u32,
        task_index: usize,
    },
    ComparingMachines {
        min_machine_id: usize,
        min_makespan: u32,
        total_makespan: u32,
    },
    EvaluatingMove {
        from_machine: usize,
        to_machine: usize,
        task_value: u32,
        task_index: usize,
        new_makespan_would_be: u32,
        current_makespan: u32,
        will_move: bool,
    },
    TaskMoved {
        from_machine: usize,
        to_machine: usize,
        task_value: u32,
        old_makespan: u32,
        new_makespan: u32,
    },
    MakespanUpdated {
        machine_id: usize,
        old_makespan: u32,
        new_makespan: u32,
    },
    IterationComplete {
        iteration: u32,
        total_makespan: u32,
    },
    PerturbationStart {
        iteration: u32,
        num_moves: usize,
    },
    PerturbationMove {
        from_machine: usize,
        to_machine: usize,
        task_value: u32,
    },
    LocalSearchStart {
        iteration: u32,
    },
    BestSolutionUpdated {
        iteration: u32,
        old_best: u32,
        new_best: u32,
    },
    NoImprovement {
        iteration: u32,
        stagnation_count: u32,
    },
    AlgorithmComplete {
        final_makespan: u32,
        total_moves: u32,
    },
    MachineSnapshot {
        machines: Vec<MachineState>,
    },
}

#[derive(Debug, Clone)]
pub struct MachineState {
    pub id: usize,
    pub tasks: Vec<u32>,
    pub makespan: u32,
}

impl MachineState {
    pub fn from_maquina(id: usize, maquina: &Maquina) -> Self {
        let tasks = if maquina.pos >= 0 {
            maquina.tarefas[0..=(maquina.pos as usize)].to_vec()
        } else {
            vec![]
        };
        MachineState {
            id,
            tasks,
            makespan: maquina.ms_maquina(),
        }
    }

    pub fn from_maquinas(maquinas: &[Maquina]) -> Vec<Self> {
        maquinas
            .iter()
            .enumerate()
            .map(|(id, m)| Self::from_maquina(id, m))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackSpeed {
    Slow,     // 2000ms per step
    Medium,   // 500ms per step
    Fast,     // 100ms per step
    VeryFast, // 20ms per step
}

impl PlaybackSpeed {
    pub fn duration(&self) -> Duration {
        match self {
            PlaybackSpeed::Slow => Duration::from_millis(2000),
            PlaybackSpeed::Medium => Duration::from_millis(500),
            PlaybackSpeed::Fast => Duration::from_millis(100),
            PlaybackSpeed::VeryFast => Duration::from_millis(20),
        }
    }

    pub fn next(&self) -> Self {
        match self {
            PlaybackSpeed::Slow => PlaybackSpeed::Medium,
            PlaybackSpeed::Medium => PlaybackSpeed::Fast,
            PlaybackSpeed::Fast => PlaybackSpeed::VeryFast,
            PlaybackSpeed::VeryFast => PlaybackSpeed::VeryFast,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            PlaybackSpeed::Slow => PlaybackSpeed::Slow,
            PlaybackSpeed::Medium => PlaybackSpeed::Slow,
            PlaybackSpeed::Fast => PlaybackSpeed::Medium,
            PlaybackSpeed::VeryFast => PlaybackSpeed::Fast,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            PlaybackSpeed::Slow => "Slow",
            PlaybackSpeed::Medium => "Medium",
            PlaybackSpeed::Fast => "Fast",
            PlaybackSpeed::VeryFast => "Very Fast",
        }
    }
}

pub struct AnimationScreen {
    pub events: Vec<AnimationEvent>,
    pub current_step: usize,
    pub playback_state: PlaybackState,
    pub speed: PlaybackSpeed,
    pub last_step_time: Instant,
    pub machine_states: Vec<MachineState>,
    pub algorithm_name: String,
}

impl AnimationScreen {
    pub fn new(events: Vec<AnimationEvent>, algorithm_name: String) -> Self {
        // Find the first MachineSnapshot to initialize the display
        let machine_states = events
            .iter()
            .find_map(|event| {
                if let AnimationEvent::MachineSnapshot { machines } = event {
                    Some(machines.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        AnimationScreen {
            events,
            current_step: 0,
            playback_state: PlaybackState::Paused,
            speed: PlaybackSpeed::Medium,
            last_step_time: Instant::now(),
            machine_states,
            algorithm_name,
        }
    }

    pub fn step_forward(&mut self) {
        if self.current_step < self.events.len() - 1 {
            self.current_step += 1;
            self.update_machine_states();
            self.last_step_time = Instant::now();
        } else {
            self.playback_state = PlaybackState::Stopped;
        }
    }

    pub fn step_backward(&mut self) {
        if self.current_step > 0 {
            self.current_step -= 1;
            self.update_machine_states();
            self.last_step_time = Instant::now();
        }
    }

    pub fn toggle_playback(&mut self) {
        self.playback_state = match self.playback_state {
            PlaybackState::Playing => PlaybackState::Paused,
            PlaybackState::Paused => PlaybackState::Playing,
            PlaybackState::Stopped => {
                self.current_step = 0;
                self.update_machine_states();
                PlaybackState::Playing
            }
        };
        self.last_step_time = Instant::now();
    }

    pub fn restart(&mut self) {
        self.current_step = 0;
        self.playback_state = PlaybackState::Paused;
        self.update_machine_states();
        self.last_step_time = Instant::now();
    }

    pub fn increase_speed(&mut self) {
        self.speed = self.speed.next();
    }

    pub fn decrease_speed(&mut self) {
        self.speed = self.speed.prev();
    }

    pub fn update(&mut self) -> bool {
        if self.playback_state == PlaybackState::Playing {
            if self.last_step_time.elapsed() >= self.speed.duration() {
                self.step_forward();
                return true;
            }
        }
        false
    }

    fn update_machine_states(&mut self) {
        // Search backwards from current step to find the most recent MachineSnapshot
        for i in (0..=self.current_step).rev() {
            if let Some(AnimationEvent::MachineSnapshot { machines }) = self.events.get(i) {
                self.machine_states = machines.clone();
                return;
            }
        }
    }

    pub fn current_event(&self) -> Option<&AnimationEvent> {
        self.events.get(self.current_step)
    }
}

pub fn render_animation(f: &mut Frame, animation: &AnimationScreen, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Machine visualization
            Constraint::Length(6), // Event details
            Constraint::Length(4), // Controls
        ])
        .split(area);

    // Title
    let title = Paragraph::new(format!("Animation: {}", animation.algorithm_name))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Machine visualization
    render_machines(f, animation, chunks[1]);

    // Event details
    render_event_details(f, animation, chunks[2]);

    // Controls
    render_controls(f, animation, chunks[3]);
}

fn render_machines(f: &mut Frame, animation: &AnimationScreen, area: Rect) {
    if animation.machine_states.is_empty() {
        let placeholder = Paragraph::new("No machine data available")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Machines"));
        f.render_widget(placeholder, area);
        return;
    }

    let block = Block::default().borders(Borders::ALL).title("Machines");
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Create layout for each machine
    let machine_count = animation.machine_states.len();
    let constraints: Vec<Constraint> =
        vec![Constraint::Ratio(1, machine_count as u32); machine_count];

    let machine_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, machine_state) in animation.machine_states.iter().enumerate() {
        if i < machine_chunks.len() {
            render_single_machine(f, machine_state, machine_chunks[i], animation);
        }
    }
}

fn render_single_machine(
    f: &mut Frame,
    machine: &MachineState,
    area: Rect,
    animation: &AnimationScreen,
) {
    let mut lines = vec![];

    // Machine header with makespan
    let header = format!(
        "Machine {} | Makespan: {} | Tasks: {}",
        machine.id,
        machine.makespan,
        machine.tasks.len()
    );
    lines.push(Line::from(Span::styled(
        header,
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));

    // Task visualization - show tasks as colored blocks
    let mut task_line = vec![];
    let current_event = animation.current_event();

    for (idx, &task) in machine.tasks.iter().enumerate() {
        let mut style = Style::default();

        // Determine color based on task value
        let color = if task >= 75 {
            Color::Red
        } else if task >= 50 {
            Color::Yellow
        } else if task >= 25 {
            Color::Green
        } else {
            Color::Cyan
        };
        style = style.fg(color);

        // Highlight tasks involved in current event
        if let Some(event) = current_event {
            match event {
                AnimationEvent::EvaluatingMove {
                    from_machine,
                    task_value,
                    ..
                } => {
                    if *from_machine == machine.id && *task_value == task {
                        style = style.add_modifier(Modifier::BOLD).bg(Color::Blue);
                    }
                }
                AnimationEvent::TaskMoved {
                    from_machine,
                    to_machine,
                    task_value,
                    ..
                } => {
                    if (*from_machine == machine.id || *to_machine == machine.id)
                        && *task_value == task
                    {
                        style = style.add_modifier(Modifier::BOLD).bg(Color::Magenta);
                    }
                }
                _ => {}
            }
        }

        task_line.push(Span::styled(format!("[{:3}]", task), style));
        if idx < machine.tasks.len() - 1 {
            task_line.push(Span::raw(" "));
        }
    }

    if !task_line.is_empty() {
        lines.push(Line::from(task_line));
    } else {
        lines.push(Line::from(Span::styled(
            "[empty]",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(lines).block(Block::default());
    f.render_widget(paragraph, area);
}

fn render_event_details(f: &mut Frame, animation: &AnimationScreen, area: Rect) {
    let mut lines = vec![];

    let progress = format!(
        "Step {}/{}",
        animation.current_step + 1,
        animation.events.len()
    );
    lines.push(Line::from(Span::styled(
        progress,
        Style::default().fg(Color::Cyan),
    )));

    if let Some(event) = animation.current_event() {
        lines.push(Line::from(""));
        match event {
            AnimationEvent::TaskGenerated {
                machine_id,
                task_value,
                task_index,
            } => {
                lines.push(Line::from(format!(
                    "Generated task with value {} at position {} on Machine {}",
                    task_value, task_index, machine_id
                )));
            }
            AnimationEvent::ComparingMachines {
                min_machine_id,
                min_makespan,
                total_makespan,
            } => {
                lines.push(Line::from(Span::styled(
                    format!("Comparing machines..."),
                    Style::default().fg(Color::Yellow),
                )));
                lines.push(Line::from(format!(
                    "Machine {} has minimum makespan: {} (Total: {})",
                    min_machine_id, min_makespan, total_makespan
                )));
            }
            AnimationEvent::EvaluatingMove {
                from_machine,
                to_machine,
                task_value,
                will_move,
                new_makespan_would_be,
                current_makespan,
                ..
            } => {
                lines.push(Line::from(Span::styled(
                    format!(
                        "Evaluating move of task {} from Machine {} to Machine {}",
                        task_value, from_machine, to_machine
                    ),
                    Style::default().fg(Color::Yellow),
                )));
                lines.push(Line::from(format!(
                    "Current makespan: {} → Would become: {} → {}",
                    current_makespan,
                    new_makespan_would_be,
                    if *will_move {
                        "✓ Moving"
                    } else {
                        "✗ Not moving"
                    }
                )));
            }
            AnimationEvent::TaskMoved {
                from_machine,
                to_machine,
                task_value,
                old_makespan,
                new_makespan,
            } => {
                lines.push(Line::from(Span::styled(
                    format!(
                        "Moved task {} from Machine {} to Machine {}",
                        task_value, from_machine, to_machine
                    ),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(format!(
                    "Makespan improved: {} → {}",
                    old_makespan, new_makespan
                )));
            }
            AnimationEvent::IterationComplete {
                iteration,
                total_makespan,
            } => {
                lines.push(Line::from(Span::styled(
                    format!(
                        "Iteration {} complete. Total makespan: {}",
                        iteration, total_makespan
                    ),
                    Style::default().fg(Color::Green),
                )));
            }
            AnimationEvent::PerturbationStart {
                iteration,
                num_moves,
            } => {
                lines.push(Line::from(Span::styled(
                    format!(
                        "Starting perturbation phase (iteration {}) with {} random moves",
                        iteration, num_moves
                    ),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )));
            }
            AnimationEvent::PerturbationMove {
                from_machine,
                to_machine,
                task_value,
            } => {
                lines.push(Line::from(Span::styled(
                    format!(
                        "Perturbation: Moving task {} from Machine {} to Machine {}",
                        task_value, from_machine, to_machine
                    ),
                    Style::default().fg(Color::Magenta),
                )));
            }
            AnimationEvent::LocalSearchStart { iteration } => {
                lines.push(Line::from(Span::styled(
                    format!("Starting local search phase (iteration {})", iteration),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));
            }
            AnimationEvent::BestSolutionUpdated {
                iteration,
                old_best,
                new_best,
            } => {
                lines.push(Line::from(Span::styled(
                    format!("New best solution found! (iteration {})", iteration),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(format!(
                    "Best makespan: {} → {}",
                    old_best, new_best
                )));
            }
            AnimationEvent::NoImprovement {
                iteration,
                stagnation_count,
            } => {
                lines.push(Line::from(Span::styled(
                    format!(
                        "No improvement in iteration {} (stagnation: {})",
                        iteration, stagnation_count
                    ),
                    Style::default().fg(Color::Yellow),
                )));
            }
            AnimationEvent::AlgorithmComplete {
                final_makespan,
                total_moves,
            } => {
                lines.push(Line::from(Span::styled(
                    format!(
                        "Algorithm complete! Final makespan: {} ({} moves)",
                        final_makespan, total_moves
                    ),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )));
            }
            _ => {}
        }
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Event Details"),
    );
    f.render_widget(paragraph, area);
}

fn render_controls(f: &mut Frame, animation: &AnimationScreen, area: Rect) {
    let state_symbol = match animation.playback_state {
        PlaybackState::Playing => "▶",
        PlaybackState::Paused => "⏸",
        PlaybackState::Stopped => "⏹",
    };

    let controls = vec![
        Line::from(vec![
            Span::styled(
                format!("{} ", state_symbol),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Speed: "),
            Span::styled(animation.speed.as_str(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Space", Style::default().fg(Color::Cyan)),
            Span::raw(": Play/Pause | "),
            Span::styled("←/→", Style::default().fg(Color::Cyan)),
            Span::raw(": Step | "),
            Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
            Span::raw(": Speed | "),
            Span::styled("R", Style::default().fg(Color::Cyan)),
            Span::raw(": Restart | "),
            Span::styled("ESC", Style::default().fg(Color::Cyan)),
            Span::raw(": Exit"),
        ]),
    ];

    let paragraph =
        Paragraph::new(controls).block(Block::default().borders(Borders::ALL).title("Controls"));
    f.render_widget(paragraph, area);
}
