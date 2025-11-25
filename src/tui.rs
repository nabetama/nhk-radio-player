use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io::{self, Stdout};
use std::sync::Arc;
use tokio::sync::watch;
use unicode_width::UnicodeWidthStr;

use crate::client::NhkRadioClient;
use crate::types::{Root, StreamData};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ChannelKind {
    R1,
    R2,
    Fm,
}

impl ChannelKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            ChannelKind::R1 => "ラジオ第1",
            ChannelKind::R2 => "ラジオ第2",
            ChannelKind::Fm => "FM",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            ChannelKind::R1 => "R1",
            ChannelKind::R2 => "R2",
            ChannelKind::Fm => "FM",
        }
    }

    pub fn get_url(&self, data: &StreamData) -> String {
        match self {
            ChannelKind::R1 => data.r1hls.clone(),
            ChannelKind::R2 => data.r2hls.clone(),
            ChannelKind::Fm => data.fmhls.clone(),
        }
    }

    pub fn next(&self) -> Self {
        match self {
            ChannelKind::R1 => ChannelKind::R2,
            ChannelKind::R2 => ChannelKind::Fm,
            ChannelKind::Fm => ChannelKind::R1,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            ChannelKind::R1 => ChannelKind::Fm,
            ChannelKind::R2 => ChannelKind::R1,
            ChannelKind::Fm => ChannelKind::R2,
        }
    }
}

pub struct ProgramInfo {
    pub station_name: String,
    pub area_name: String,
    pub program_title: String,
    pub start_time: String,
    pub description: String,
}

impl ProgramInfo {
    pub fn from_program(program: &Option<Root>, kind: ChannelKind, area_name: &str) -> Self {
        let (program_title, description, start_time) = program
            .as_ref()
            .and_then(|p| {
                let channel = match kind {
                    ChannelKind::R1 => &p.r1,
                    ChannelKind::R2 => &p.r2,
                    ChannelKind::Fm => &p.r3,
                };
                channel.present.as_ref().map(|present| {
                    let title = present
                        .about
                        .as_ref()
                        .map(|a| a.name.clone())
                        .unwrap_or_else(|| present.name.clone());
                    let desc = present
                        .about
                        .as_ref()
                        .map(|a| a.description.clone())
                        .unwrap_or_default();
                    let time = format_time(&present.start_date);
                    (title, desc, time)
                })
            })
            .unwrap_or_else(|| ("番組情報を取得中...".to_string(), String::new(), String::new()));

        ProgramInfo {
            station_name: kind.display_name().to_string(),
            area_name: area_name.to_string(),
            program_title,
            start_time,
            description,
        }
    }
}

fn format_time(iso_time: &str) -> String {
    // Parse ISO format like "2025-11-25T23:00:00+09:00"
    if iso_time.len() >= 16 {
        let date_part = &iso_time[0..10];
        let time_part = &iso_time[11..16];

        // Convert to Japanese format
        let parts: Vec<&str> = date_part.split('-').collect();
        if parts.len() == 3 {
            let month = parts[1].parse::<u32>().unwrap_or(0);
            let day = parts[2].parse::<u32>().unwrap_or(0);

            // Convert 24h time to AM/PM Japanese
            let time_parts: Vec<&str> = time_part.split(':').collect();
            if time_parts.len() == 2 {
                let hour = time_parts[0].parse::<u32>().unwrap_or(0);
                let minute = time_parts[1];
                let (period, display_hour) = if hour < 12 {
                    ("午前", if hour == 0 { 12 } else { hour })
                } else {
                    ("午後", if hour == 12 { 12 } else { hour - 12 })
                };
                return format!("{}年{}月{}日 {}{:02}:{}", parts[0], month, day, period, display_hour, minute);
            }
        }
    }
    iso_time.to_string()
}

pub struct AppState {
    pub current_channel: ChannelKind,
    pub program_info: ProgramInfo,
    pub is_loading: bool,
    pub animation_frame: usize,
}


pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn restore(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub fn draw(&mut self, state: &AppState) -> Result<()> {
        self.terminal.draw(|f| {
            render_ui(f, state);
        })?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

fn truncate_str(s: &str, max_width: usize) -> String {
    let width = UnicodeWidthStr::width(s);
    if width <= max_width {
        return s.to_string();
    }

    let mut result = String::new();
    let mut current_width = 0;

    for c in s.chars() {
        let char_width = UnicodeWidthStr::width(c.to_string().as_str());
        if current_width + char_width + 3 > max_width {
            result.push_str("...");
            break;
        }
        current_width += char_width;
        result.push(c);
    }

    result
}

fn render_ui(f: &mut Frame, state: &AppState) {
    let size = f.area();

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Channel selector
            Constraint::Length(1), // Spacer
            Constraint::Min(8),    // Now playing info
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Status bar
            Constraint::Length(2), // Help
        ])
        .split(size);

    // Channel selector
    render_channel_selector(f, chunks[0], state);

    // Now playing info
    render_now_playing(f, chunks[2], state);

    // Status bar
    render_status_bar(f, chunks[4], state);

    // Help
    render_help(f, chunks[5]);
}

fn render_channel_selector(f: &mut Frame, area: Rect, state: &AppState) {
    let channels = [ChannelKind::R1, ChannelKind::R2, ChannelKind::Fm];

    let channel_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    for (i, &channel) in channels.iter().enumerate() {
        let is_selected = channel == state.current_channel;
        let key = match channel {
            ChannelKind::R1 => "1",
            ChannelKind::R2 => "2",
            ChannelKind::Fm => "3",
        };

        let label = format!("[{}] {}", key, channel.short_name());

        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            });

        let paragraph = Paragraph::new(label)
            .style(style)
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

        f.render_widget(paragraph, channel_chunks[i]);
    }
}

fn render_now_playing(f: &mut Frame, area: Rect, state: &AppState) {
    let info = &state.program_info;

    let title = format!(" 📻 NHK {} - {} ", info.station_name, info.area_name);

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    let content_width = inner.width.saturating_sub(2) as usize;

    let mut lines = vec![];

    // Program title with time
    let title_line = if info.start_time.is_empty() {
        format!("♪ {}", truncate_str(&info.program_title, content_width.saturating_sub(2)))
    } else {
        format!("♪ {}", truncate_str(&info.program_title, content_width.saturating_sub(2)))
    };
    lines.push(Line::from(Span::styled(
        title_line,
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));

    // Start time
    if !info.start_time.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("  {}", info.start_time),
            Style::default().fg(Color::Green),
        )));
    }

    // Empty line
    lines.push(Line::from(""));

    // Description (wrap if needed)
    if !info.description.is_empty() {
        let desc = truncate_str(&info.description, content_width);
        lines.push(Line::from(Span::styled(
            desc,
            Style::default().fg(Color::White).add_modifier(Modifier::DIM),
        )));
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);
}

fn render_status_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let status = if state.is_loading {
        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let frame = spinner[state.animation_frame % spinner.len()];
        format!("{} 読み込み中...", frame)
    } else {
        "▶ 再生中".to_string()
    };

    let style = if state.is_loading {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    };

    let paragraph = Paragraph::new(status)
        .style(style)
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled("[1]", Style::default().fg(Color::Cyan)),
        Span::raw(" R1  "),
        Span::styled("[2]", Style::default().fg(Color::Cyan)),
        Span::raw(" R2  "),
        Span::styled("[3]", Style::default().fg(Color::Cyan)),
        Span::raw(" FM  "),
        Span::styled("[←/→]", Style::default().fg(Color::Cyan)),
        Span::raw(" 切替  "),
        Span::styled("[q]", Style::default().fg(Color::Red)),
        Span::raw(" 終了"),
    ]);

    let paragraph = Paragraph::new(help)
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, area);
}

pub async fn run_interactive_player(
    area: String,
    initial_kind: ChannelKind,
) -> Result<()> {
    let client = Arc::new(NhkRadioClient::new());
    let config = client.fetch_config().await?;

    let stream_data = config
        .stream_url
        .data
        .iter()
        .find(|d| d.area == area)
        .ok_or_else(|| anyhow::anyhow!("Area not found: {}", area))?
        .clone();

    let program_url = config
        .url_program_noa
        .replace("//", "https://")
        .replace("{area}", &stream_data.areakey);

    let program = client.fetch_program(&program_url).await.ok();

    let initial_info = ProgramInfo::from_program(&program, initial_kind, &stream_data.areajp);

    let mut state = AppState {
        current_channel: initial_kind,
        program_info: initial_info,
        is_loading: true,
        animation_frame: 0,
    };

    let (channel_tx, channel_rx) = watch::channel(initial_kind);
    let (audio_tx, audio_rx) = std::sync::mpsc::channel::<Vec<i16>>();

    // Audio playback thread (must be on main thread for rodio)
    let audio_handle = std::thread::spawn(move || {
        run_audio_thread(audio_rx, channel_rx)
    });

    // Start streaming in background
    let player_client = client.clone();
    let player_stream_data = stream_data.clone();
    let player_channel_rx = channel_tx.subscribe();
    let player_handle = tokio::spawn(async move {
        run_stream_loop(player_client, player_stream_data, player_channel_rx, audio_tx).await
    });

    let mut tui = Tui::new()?;

    state.is_loading = false;

    loop {
        tui.draw(&state)?;

        // Handle input with timeout for animation
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            break;
                        }
                        KeyCode::Char('1') => {
                            if state.current_channel != ChannelKind::R1 {
                                state.current_channel = ChannelKind::R1;
                                state.is_loading = true;
                                state.program_info = ProgramInfo::from_program(
                                    &program,
                                    ChannelKind::R1,
                                    &stream_data.areajp,
                                );
                                let _ = channel_tx.send(ChannelKind::R1);
                                state.is_loading = false;
                            }
                        }
                        KeyCode::Char('2') => {
                            if state.current_channel != ChannelKind::R2 {
                                state.current_channel = ChannelKind::R2;
                                state.is_loading = true;
                                state.program_info = ProgramInfo::from_program(
                                    &program,
                                    ChannelKind::R2,
                                    &stream_data.areajp,
                                );
                                let _ = channel_tx.send(ChannelKind::R2);
                                state.is_loading = false;
                            }
                        }
                        KeyCode::Char('3') => {
                            if state.current_channel != ChannelKind::Fm {
                                state.current_channel = ChannelKind::Fm;
                                state.is_loading = true;
                                state.program_info = ProgramInfo::from_program(
                                    &program,
                                    ChannelKind::Fm,
                                    &stream_data.areajp,
                                );
                                let _ = channel_tx.send(ChannelKind::Fm);
                                state.is_loading = false;
                            }
                        }
                        KeyCode::Left => {
                            let new_channel = state.current_channel.prev();
                            if state.current_channel != new_channel {
                                state.current_channel = new_channel;
                                state.is_loading = true;
                                state.program_info = ProgramInfo::from_program(
                                    &program,
                                    new_channel,
                                    &stream_data.areajp,
                                );
                                let _ = channel_tx.send(new_channel);
                                state.is_loading = false;
                            }
                        }
                        KeyCode::Right => {
                            let new_channel = state.current_channel.next();
                            if state.current_channel != new_channel {
                                state.current_channel = new_channel;
                                state.is_loading = true;
                                state.program_info = ProgramInfo::from_program(
                                    &program,
                                    new_channel,
                                    &stream_data.areajp,
                                );
                                let _ = channel_tx.send(new_channel);
                                state.is_loading = false;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        state.animation_frame = state.animation_frame.wrapping_add(1);
    }

    drop(tui);
    player_handle.abort();
    drop(audio_handle);

    Ok(())
}

fn run_audio_thread(
    rx: std::sync::mpsc::Receiver<Vec<i16>>,
    _channel_rx: watch::Receiver<ChannelKind>,
) -> Result<()> {
    use rodio::buffer::SamplesBuffer;

    log::info!("Audio thread starting...");

    // Try to get audio output, retry if it fails
    let (stream, sink) = loop {
        match rodio::OutputStream::try_default() {
            Ok((_stream, stream_handle)) => {
                match rodio::Sink::try_new(&stream_handle) {
                    Ok(sink) => {
                        log::info!("Audio output initialized successfully");
                        break (_stream, sink);
                    }
                    Err(e) => {
                        log::error!("Failed to create audio sink: {}", e);
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to get audio output: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    };

    // Keep _stream alive
    let _stream = stream;

    let mut sample_count = 0u64;

    loop {
        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(samples) => {
                if samples.is_empty() {
                    log::info!("Audio: Received clear signal, clearing sink");
                    sink.clear();
                    sink.play(); // Ensure sink is in playing state after clear
                } else {
                    sample_count += 1;
                    if sample_count % 10 == 1 {
                        log::debug!("Audio: Received samples batch #{}, {} samples, sink empty: {}",
                            sample_count, samples.len(), sink.empty());
                    }
                    let buffer = SamplesBuffer::new(2, 48000, samples);
                    sink.append(buffer);
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                continue;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                log::info!("Audio thread: channel disconnected, exiting");
                break;
            }
        }
    }

    Ok(())
}

async fn run_stream_loop(
    client: Arc<NhkRadioClient>,
    stream_data: StreamData,
    mut channel_rx: watch::Receiver<ChannelKind>,
    audio_tx: std::sync::mpsc::Sender<Vec<i16>>,
) -> Result<()> {
    use crate::crypto::decrypt_segment;
    use crate::decoder::decode_aac_to_pcm;
    use crate::m3u8::parse_m3u8;
    use std::collections::{HashMap, HashSet};

    let mut current_channel = *channel_rx.borrow_and_update();
    let mut seen_segments: HashSet<String> = HashSet::new();
    let mut resolved_urls: HashMap<ChannelKind, String> = HashMap::new();

    loop {
        // Check for channel change
        if channel_rx.has_changed().unwrap_or(false) {
            let new_channel = *channel_rx.borrow_and_update();
            if new_channel != current_channel {
                log::info!("Channel changed from {:?} to {:?}", current_channel, new_channel);
                current_channel = new_channel;
                seen_segments.clear();
                // Send empty vec to signal clear
                let _ = audio_tx.send(vec![]);
            }
        }

        let m3u8_url = current_channel.get_url(&stream_data);
        log::debug!("Fetching playlist for channel {:?}: {}", current_channel, m3u8_url);

        // Resolve master playlist if needed (cache the result)
        let actual_url = if let Some(url) = resolved_urls.get(&current_channel) {
            url.clone()
        } else {
            match resolve_master_playlist(&client, &m3u8_url).await {
                Ok(url) => {
                    log::info!("Resolved playlist URL for {:?}: {}", current_channel, url);
                    resolved_urls.insert(current_channel, url.clone());
                    url
                }
                Err(e) => {
                    log::error!("Failed to resolve master playlist: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    continue;
                }
            }
        };

        let playlist_content = match client.fetch_m3u8(&actual_url).await {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to fetch playlist: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                continue;
            }
        };

        let segments = match parse_m3u8(&playlist_content, &actual_url) {
            Ok(segs) => segs,
            Err(e) => {
                log::error!("Failed to parse playlist: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                continue;
            }
        };

        let key = if let Some(ref seg) = segments.first() {
            if let Some(ref key_url) = seg.key_url {
                Some(client.fetch_key(key_url).await?)
            } else {
                None
            }
        } else {
            None
        };

        let mut channel_changed = false;
        for segment in segments {
            // Check for channel change during segment processing
            if channel_rx.has_changed().unwrap_or(false) {
                channel_changed = true;
                break;
            }

            if seen_segments.contains(&segment.url) {
                continue;
            }
            seen_segments.insert(segment.url.clone());

            let mut data = match client.fetch_segment(&segment.url).await {
                Ok(d) => d,
                Err(e) => {
                    log::error!("Failed to fetch segment: {}", e);
                    continue;
                }
            };

            if let Some(ref k) = key {
                data = match decrypt_segment(&data, k, segment.iv.as_deref(), segment.seq_no) {
                    Ok(d) => d,
                    Err(e) => {
                        log::error!("Failed to decrypt: {}", e);
                        continue;
                    }
                };
            }

            match decode_aac_to_pcm(&data) {
                Ok(pcm_samples) if !pcm_samples.is_empty() => {
                    let _ = audio_tx.send(pcm_samples);
                }
                Ok(_) => {}
                Err(e) => {
                    log::debug!("Failed to decode AAC: {}", e);
                }
            }
        }

        // If channel changed, loop immediately without sleeping
        if channel_changed {
            continue;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn resolve_master_playlist(client: &NhkRadioClient, m3u8_url: &str) -> Result<String> {
    use crate::m3u8::parse_m3u8;

    let playlist_content = client.fetch_m3u8(m3u8_url).await?;

    match parse_m3u8(&playlist_content, m3u8_url) {
        Ok(_) => Ok(m3u8_url.to_string()),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Master playlist detected") {
                if let Some(variant_url) = error_msg.split("Variant URL: ").nth(1) {
                    log::info!("Detected master playlist, using variant: {}", variant_url);
                    Ok(variant_url.to_string())
                } else {
                    Err(e)
                }
            } else {
                Err(e)
            }
        }
    }
}
