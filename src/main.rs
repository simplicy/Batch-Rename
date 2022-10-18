use std::{fs::{self, DirEntry}, path::{PathBuf}, time::{Instant, Duration}};
use rfd::FileDialog;

use iced::{
    button, text_input, Button,  window,
    Column, Container, Element, Length, ProgressBar, Row, Rule, executor, time, 
    Settings, Text, TextInput, slider, Slider, Subscription, Command, Application
};


pub fn main() -> iced::Result {
    let height = 355;
    let width = 525;
    Refactoring::run(Settings {
        window: window::Settings {
            size: (width, height),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

#[derive(Default)]
enum State {
     #[default] Idle,
    Running { last_tick: Instant },
}

#[derive(Default)]
struct Refactoring {
    theme: style::Theme,
    find_value: String,
    find: text_input::State,
    slider: slider::State,
    slider_value: f32,
    start_button: button::State,
    open_button: button::State,
    msg_value: String,
    progress_value: f32,
    duration: Duration,
    file_count: f32,
    index: usize,
    files: Vec<DirEntry>,
    state: State,
}

#[derive(Debug, Clone)]
enum Message {
    FindChanged(String),
    OpenPressed,
    SliderChanged(f32),
    Tick(Instant),
    StartRunning,
}

//Implement the new Struct as an Object as "extending" Sandbox
impl Application for Refactoring {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    //What occurs on creations of this object, currently its set to default. 
    fn new(_flags: ()) -> (Refactoring, Command<Message>) {
        (
            Refactoring::default(), 
            Command::none(),
        )
    }
    //Title name of the Window Object
    fn title(&self) -> String {
        String::from("Batch Rename - Iced")
    }
    //What occurs upon a state change, all messages that can occur should be listed here
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SliderChanged(value) => self.slider_value = value,
            Message::FindChanged(value) => self.find_value = value,
            Message::OpenPressed => {
                self.progress_value = 0.00;
                self.msg_value = "".to_string();
                let open = FileDialog::new()
                    .add_filter("text", &["txt", "rs"])
                    .add_filter("rust", &["rs", "toml"])
                    .set_directory("/")
                    .pick_folder();
                let path = open.map(|s| PathBuf::from(s));
                match path {
                    Some(p) => {
                        self.find_value = p.into_os_string().into_string().unwrap();
                        self.files = fs::read_dir(&self.find_value)
                            .unwrap()
                            .map(|r| r.unwrap())
                            .collect();
                        self.file_count = fs::read_dir(&self.find_value)
                            .unwrap()
                            .count() as f32;
                    },
                    None => { self.find_value = String::from("");}
                };
                
            },
            Message::Tick(now) => {
                let file = self.files.get(self.index).unwrap();
                if let State::Running { last_tick } = &mut self.state {
                    if !file.path().is_dir() {
                        self.msg_value = file.path().file_name()
                            .unwrap()
                            .to_os_string()
                            .into_string()
                            .unwrap();
                        let len = self.msg_value.len();
                        let filename: String = self.msg_value.chars().skip(self.slider_value as usize).take(len).collect();
                        let mut new_name = format!("{}{}",self.find_value.clone(),"/");
                        new_name.push_str(&filename);
                        self.msg_value = filename;
                        let _result = fs::rename(file.path(), new_name);
                    }                        
                    self.progress_value += 100.0/self.file_count;
                    self.duration += now - *last_tick;
                    *last_tick = now;
                    self.index += 1;

                    if self.file_count == self.index as f32 {
                        self.state = State::Idle;
                    }
                }
            },
            Message::StartRunning => {
                if !self.find_value.is_empty() {
                    self.progress_value = 0.0;
                    self.index = 0;
                    self.state = State::Running {
                        last_tick: Instant::now(),
                    };
                }                
            },
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.state {
            State::Idle => Subscription::none(),
            State::Running  { .. } => {
                time::every(Duration::from_millis(10)).map(Message::Tick)
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        
        //Progress bar settings and styling

        let _progress_bar =
            ProgressBar::new(0.0..=100.0, self.progress_value).style(self.theme);
        
        let slider = Slider::new(
            &mut self.slider,
            0.0..=10.0,
            self.slider_value,
            Message::SliderChanged,
        )
        .style(self.theme);

        let text_input = TextInput::new(
            &mut self.find,
            "Path to dir...",
            &self.find_value,
            Message::FindChanged,
        )
        .padding(10)
        .size(20)
        .style(self.theme);

        let start_button = Button::new(&mut self.start_button, Text::new("Remove"))
        .padding(15)
        .on_press(Message::StartRunning)
        .style(self.theme);
        
        let msg = Text::new(&self.msg_value);

        let open_button = Button::new(&mut self.open_button, Text::new("Open"))
            .padding(10)
            .on_press(Message::OpenPressed)
            .style(self.theme);

        let mut replace_block = Row::new().spacing(20).padding(20);

        match self.state {
            State::Idle => {
                replace_block = Row::new().spacing(20)
                    .push(slider);
            }
            State::Running { .. } => {
                replace_block = Row::new().spacing(20).padding(20);
            }
        }
        
        
        //How the content is displayed on the view, can organize which children appear where by changing their occurance. 
        let content = Column::new()
            .spacing(20)
            .padding(20)
            .push(_progress_bar)
            .push(Row::new().push(msg))
            .push(Rule::horizontal(10).style(self.theme))            
            .push(Row::new().spacing(10).push(text_input).push(open_button))
            .push(Row::new().spacing(20).push(Text::new("Remove")).push(Text::new(self.slider_value.to_string())))
            .push(replace_block)
            .push(start_button);
            

        //The container that the content is within.
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .style(self.theme)
            .into()
    }
}
//Styling for the Window and Widgets. 
mod style {
    use iced::{
        button, container, progress_bar, rule,
        text_input, toggler, slider,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Theme {
        Dark
    }

    impl Theme {
        pub const ALL: [Theme; 1] = [Theme::Dark];
    }

    impl Default for Theme {
        fn default() -> Theme {
            Theme::Dark
        }
    }

    impl<'a> From<Theme> for Box<dyn container::StyleSheet + 'a> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Dark => dark::Container.into(),
            }
        }
    }

    impl<'a> From<Theme> for Box<dyn text_input::StyleSheet + 'a> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Dark => dark::TextInput.into(),
            }
        }
    }

    impl<'a> From<Theme> for Box<dyn button::StyleSheet + 'a> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Dark => dark::Button.into(),
            }
        }
    }

    impl<'a> From<Theme> for Box<dyn slider::StyleSheet + 'a> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Dark => dark::Slider.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn progress_bar::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Dark => dark::ProgressBar.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn toggler::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Dark => dark::Toggler.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn rule::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Dark => dark::Rule.into(),
            }
        }
    }

    mod dark {
        use iced::{
            button, container, progress_bar, rule,
            text_input, toggler, Color, slider,
        };

        const SURFACE: Color = Color::from_rgb(
            0x40 as f32 / 255.0,
            0x44 as f32 / 255.0,
            0x4B as f32 / 255.0,
        );

        const ACCENT: Color = Color::from_rgb(
            0x6F as f32 / 255.0,
            0xFF as f32 / 255.0,
            0xE9 as f32 / 255.0,
        );

        const ACTIVE: Color = Color::from_rgb(
            0x72 as f32 / 255.0,
            0x89 as f32 / 255.0,
            0xDA as f32 / 255.0,
        );

        const HOVERED: Color = Color::from_rgb(
            0x67 as f32 / 255.0,
            0x7B as f32 / 255.0,
            0xC4 as f32 / 255.0,
        );

        pub struct Container;

        impl container::StyleSheet for Container {
            fn style(&self) -> container::Style {
                container::Style {
                    background: Color::from_rgb8(0x36, 0x39, 0x3F).into(),
                    text_color: Color::WHITE.into(),
                    ..container::Style::default()
                }
            }
        }
        pub struct TextInput;

        impl text_input::StyleSheet for TextInput {
            fn active(&self) -> text_input::Style {
                text_input::Style {
                    background: SURFACE.into(),
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }
            }

            fn focused(&self) -> text_input::Style {
                text_input::Style {
                    border_width: 1.0,
                    border_color: ACCENT,
                    ..self.active()
                }
            }

            fn hovered(&self) -> text_input::Style {
                text_input::Style {
                    border_width: 1.0,
                    border_color: Color { a: 0.3, ..ACCENT },
                    ..self.focused()
                }
            }

            fn placeholder_color(&self) -> Color {
                Color::from_rgb(0.4, 0.4, 0.4)
            }

            fn value_color(&self) -> Color {
                Color::WHITE
            }

            fn selection_color(&self) -> Color {
                ACTIVE
            }
        }

        pub struct Slider;

        impl slider::StyleSheet for Slider {
            fn active(&self) -> slider::Style {
                slider::Style {
                    rail_colors: (ACTIVE, Color { a: 0.1, ..ACTIVE }),
                    handle: slider::Handle {
                        shape: slider::HandleShape::Circle { radius: 9.0 },
                        color: ACTIVE,
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                }
            }

            fn hovered(&self) -> slider::Style {
                let active = self.active();

                slider::Style {
                    handle: slider::Handle {
                        color: HOVERED,
                        ..active.handle
                    },
                    ..active
                }
            }

            fn dragging(&self) -> slider::Style {
                let active = self.active();

                slider::Style {
                    handle: slider::Handle {
                        color: Color::from_rgb(0.85, 0.85, 0.85),
                        ..active.handle
                    },
                    ..active
                }
            }
        }

        pub struct Button;

        impl button::StyleSheet for Button {
            fn active(&self) -> button::Style {
                button::Style {
                    background: ACTIVE.into(),
                    border_radius: 3.0,
                    text_color: Color::WHITE,
                    ..button::Style::default()
                }
            }

            fn hovered(&self) -> button::Style {
                button::Style {
                    background: HOVERED.into(),
                    text_color: Color::WHITE,
                    ..self.active()
                }
            }

            fn pressed(&self) -> button::Style {
                button::Style {
                    border_width: 1.0,
                    border_color: Color::WHITE,
                    ..self.hovered()
                }
            }
        }

        pub struct ProgressBar;

        impl progress_bar::StyleSheet for ProgressBar {
            fn style(&self) -> progress_bar::Style {
                progress_bar::Style {
                    background: SURFACE.into(),
                    bar: ACTIVE.into(),
                    border_radius: 10.0,
                }
            }
        }

        pub struct Toggler;

        impl toggler::StyleSheet for Toggler {
            fn active(&self, is_active: bool) -> toggler::Style {
                toggler::Style {
                    background: if is_active { ACTIVE } else { SURFACE },
                    background_border: None,
                    foreground: if is_active { Color::WHITE } else { ACTIVE },
                    foreground_border: None,
                }
            }

            fn hovered(&self, is_active: bool) -> toggler::Style {
                toggler::Style {
                    background: if is_active { ACTIVE } else { SURFACE },
                    background_border: None,
                    foreground: if is_active {
                        Color {
                            a: 0.5,
                            ..Color::WHITE
                        }
                    } else {
                        Color { a: 0.5, ..ACTIVE }
                    },
                    foreground_border: None,
                }
            }
        }

        pub struct Rule;

        impl rule::StyleSheet for Rule {
            fn style(&self) -> rule::Style {
                rule::Style {
                    color: SURFACE,
                    width: 2,
                    radius: 1.0,
                    fill_mode: rule::FillMode::Padded(15),
                }
            }
        }
    }
}