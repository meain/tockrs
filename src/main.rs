#[allow(dead_code)]
mod util;
extern crate reqwest;

use std::collections::HashMap;
use std::io;

use serde::{Deserialize, Serialize};
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Widget};
use tui::Terminal;

use crate::util::event::{Event, Events};

struct App {
    data: Vec<(f64, f64)>,
    min: f64,
    max: f64,
    x_max: usize,
    labels: Vec<String>,
    window: [usize; 2],
}

impl App {
    fn new(vals: HashMap<String, Point>) -> App {
        let mut data: Vec<(f64, f64)> = Vec::new();
        let mut i = 0;
        let mut min = 500.0;
        let mut max = 0.0;
        let mut labels: Vec<String> = Vec::new();
        for (key, value) in vals {
            let close_value = value.close.parse().unwrap();
            if close_value > max {
                max = close_value;
            }
            if close_value < min {
                min = close_value;
            }
            labels.push(key);
            data.push((i as f64, value.close.parse().unwrap()));
            i = i + 1;
            // println!("{} / {}", key, value.close);
        }
        // let data: Vec<(f64, f64)> = vec![(1.0, 1.9), (2.0, 2.9), (3.0, 4.9)];
        App {
            data,
            min,
            max,
            labels,
            x_max: i,
            window: [0, 100],
        }
    }

    // fn update(&mut self) {
    //     for _ in 0..5 {
    //         self.data1.remove(0);
    //     }
    //     self.data1.extend(self.signal1.by_ref().take(5));
    //     for _ in 0..10 {
    //         self.data2.remove(0);
    //     }
    //     self.data2.extend(self.signal2.by_ref().take(10));
    //     self.window[0] += 1.0;
    //     self.window[1] += 1.0;
    // }
}

// fn get_data() -> Result<(), failure::Error>{
//  let resp: HashMap<String, String> = reqwest::get("https://httpbin.org/ip")?
//         .json()?;
//     println!("{:#?}", resp);
//     Ok(())
// }

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    #[serde(rename = "1. open")]
    open: String,
    #[serde(rename = "2. high")]
    high: String,
    #[serde(rename = "3. low")]
    low: String,
    #[serde(rename = "4. close")]
    close: String,
    #[serde(rename = "5. volume")]
    volume: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    #[serde(rename = "Meta Data")] // to comply with Rust coding standards
    meta_data: HashMap<String, String>,
    #[serde(rename = "Time Series (Daily)")]
    time_series: HashMap<String, Point>,
}

fn main() -> Result<(), failure::Error> {
    let resp: Data =
        reqwest::get("https://www.alphavantage.co/query?function=TIME_SERIES_DAILY&symbol=MSFT&outputsize=full&apikey=demo")?.json()?;
    // println!("{:#?}", resp);
    // Ok(())

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();

    // App
    let mut app = App::new(resp.time_series);
    let mut zoom = 0.0;

    loop {
        terminal.draw(|mut f| {
            let size = f.size();
            Chart::default()
                .block(
                    Block::default()
                        .title("Chart")
                        .title_style(Style::default().fg(Color::Cyan).modifier(Modifier::BOLD))
                        .borders(Borders::ALL),
                )
                .x_axis(
                    Axis::default()
                        .title("Day")
                        .style(Style::default().fg(Color::Gray))
                        .labels_style(Style::default().modifier(Modifier::ITALIC))
                        .bounds([app.window[0] as f64, app.window[1] as f64])
                        .labels(&[
                            &format!("{}", app.labels[0]), // TODO: not accurately show in chart
                            &format!("{}", app.labels[app.labels.len() - 1]),
                        ]),
                )
                .y_axis(
                    Axis::default()
                        .title("Price")
                        .style(Style::default().fg(Color::Gray))
                        .labels_style(Style::default().modifier(Modifier::ITALIC))
                        .bounds([app.min - zoom, app.max + zoom])
                        .labels(&[(app.min - zoom).to_string(), (app.max + zoom).to_string()]),
                )
                .datasets(&[Dataset::default()
                    .name("MSFT")
                    .marker(Marker::Braille)
                    .style(Style::default().fg(Color::Cyan))
                    .data(&app.data)])
                .render(&mut f, size);
        })?;

        match events.next()? {
            Event::Input(input) => {
                if input == Key::Char('q') {
                    break;
                } else if input == Key::Char('l') {
                    if app.window[1] < app.x_max {
                        app.window = [app.window[0] + 100, app.window[1] + 100]
                    }
                } else if input == Key::Char('h') {
                    if app.window[0] > 0 {
                        app.window = [app.window[0] - 100, app.window[1] - 100]
                    }
                } else if input == Key::Char('j') {
                    zoom = zoom + 10.0;
                } else if input == Key::Char('k') {
                    if zoom > 10.0 {
                        zoom = zoom - 10.0;
                    }
                }
            }
            Event::Tick => {
                // app.update();
            }
        }
    }

    Ok(())
}
