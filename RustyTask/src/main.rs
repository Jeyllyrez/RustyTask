
use iced::widget::{button, column, text};
use iced::{Center, Subscription, window};
use sysinfo::System;
use iced::time;
use iced_aw::Menu;
use iced_aw::menu_bar;
use iced::border::Radius;
use iced::widget::{container, horizontal_space, row, slider,
    toggler
};
use iced::{Border, Length, Color};
use iced_aw::menu_items;
use iced::widget::vertical_space;
use iced::{alignment, theme, Element, Size, Theme};
use iced_aw::menu::Item;
use iced_aw::{quad, widgets::InnerBounds};
mod system_info;


pub fn main() -> iced::Result {
    let window_settings = window::Settings {
        size: Size::new(950.0, 700.0),                    // Initial size (width, height)
        min_size: Some(Size::new(400.0, 300.0)),           // Minimum size (width, height)
        max_size: Some(Size::new(1500.0, 1400.0)),          // Maximum size (width, height)
        ..window::Settings::default()
    };
    iced::application("RustyTask", DataValue::update, DataValue::view)
    .font(iced_fonts::REQUIRED_FONT_BYTES)
    .window(window_settings)
    .theme(DataValue::theme)
    .subscription(DataValue::subscription)
    .run()
}

struct DataValue {
    cpu_value: u64,
    mem_value: f64,
    disk_value: String,
    network_value: Vec<f64>,
    gpu_value: Vec<u64>,
    max_mem: f64,
    speed_interval: u64,
    title: String,
    theme: iced::Theme,
    dark_mode: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Debug(String),
    ColorChange(Color),
    ThemeChange(bool),
    Speed(u64),
    None,
    Tick,
}
impl Default for DataValue {
    fn default() -> Self {
        let theme = iced::Theme::custom(
            "Custom Theme".into(),
            theme::Palette {
                primary: Color::from([0.15, 0.25, 0.57]),
                ..iced::Theme::Light.palette()
            },
        );

        Self {
            speed_interval: 5,
            cpu_value: 0,
            mem_value: 0.0,
            disk_value: String::from(""),
            network_value: Vec::new(),
            gpu_value: Vec::new(),
            max_mem: 0.0,
            title: "RustyTask".to_string(),
            theme,
            dark_mode: false,
        }
        
    }
}

impl DataValue {
    //use the new value that are pull from sysinfo etc. to update
    fn update(&mut self, message: Message) {
        let sys = System::new_all();
        self.cpu_value = system_info::cpu_usage();
        self.mem_value = (sys.used_memory() as f64) / 1000000000.0;
        let mut s = String::from("");
        s.push_str(&format!("Total Usage: {:.2}% \n Total Used: {:.2}Gb \n Total Available: {:.2}Gb \n", system_info::disk_info().0, system_info::disk_info().1, system_info::disk_info().2));
        self.disk_value = s;
        let network: Vec<f64> = system_info::network_usage().into_iter()
        .flat_map(|(x, y)| vec![x as f64 / 1_000.0, y as f64 / 1_000.0])
        .collect();
        system_info::log_file();
        self.network_value = network;
        self.gpu_value = system_info::gpu_usage().unwrap();
        self.max_mem = (sys.total_memory() as f64) / 1000000000.0;
        match message {
            Message::Speed(second) => {
                self.speed_interval = second;
            }
            Message::ColorChange(c) => {
                self.theme = iced::Theme::custom(
                    "Color Change".into(),
                    theme::Palette {
                        primary: c,
                        ..self.theme.palette()
                    },
                );
                self.title = format!("[{:.2}, {:.2}, {:.2}]", c.r, c.g, c.b);
            }
            Message::ThemeChange(b) => {
                self.dark_mode = b;
                let primary = self.theme.palette().primary;
                if b {
                    self.theme = iced::Theme::custom(
                        "Dark".into(),
                        theme::Palette {
                            primary,
                            ..iced::Theme::Dark.palette()
                        },
                    )
                } else {
                    self.theme = iced::Theme::custom(
                        "Light".into(),
                        theme::Palette {
                            primary,
                            ..iced::Theme::Light.palette()
                        },
                    )
                }
            }
            _=> {}
        }
        
    }
    fn subscription(&self) -> Subscription<Message> {
        time::every(std::time::Duration::from_secs(self.speed_interval)).map(|_| Message::Tick)
    }
    fn view(&self) -> iced::Element<'_, Message> {
        let menu_tpl_1 = |items| Menu::new(items).max_width(180.0).offset(0.0).spacing(0.0);
        let menu_tpl_2 = |items| Menu::new(items).max_width(180.0).offset(0.0).spacing(0.0);
        #[rustfmt::skip]
        let mb = menu_bar!(
            (debug_button_s("Settings"), {
                let speed_control = menu_tpl_2(menu_items!(
                    (debug_button("Fast").on_press(Message::Speed(2)))
                    (debug_button("Normal").on_press(Message::Speed(4)))
                    (debug_button("Slow").on_press(Message::Speed(7)))
                    (debug_button("diabollically slow").on_press(Message::Speed(10)))
                ));
                let color_control = menu_tpl_2(menu_items!(
                    (row![toggler(
                        self.dark_mode,
                    ).label("Dark Mode".to_string()).on_toggle(Message::ThemeChange)].padding([0, 8])
                )
                (color_button([0.45, 0.25, 0.57]))
                (color_button([0.15, 0.59, 0.64]))
                (color_button([0.76, 0.82, 0.20]))
                (color_button([0.17, 0.27, 0.33]))
                (labeled_button("Primary", Message::None)
                    .width(Length::Fill),
                    {
                        let [r, g, b, _] = self.theme.palette().primary.into_rgba8();

                        menu_tpl_2(menu_items!(
                            (slider(0..=255, r, move |x| {
                                Message::ColorChange(Color::from_rgb8(x, g, b))
                            }))
                            (slider(0..=255, g, move |x| {
                                Message::ColorChange(Color::from_rgb8(r, x, b))
                            }))
                            (slider(0..=255, b, move |x| {
                                Message::ColorChange(Color::from_rgb8(r, g, x))
                            }))
                        ))
                    }
                )
                ));
                menu_tpl_1(menu_items!(
                    (debug_button("Speed Control"), speed_control)
                    (debug_button("Color Control"), color_control)
                ))
                
        }));
        let cpu_section = bordered_container(
            text(format!("CPU Usage: {:.2}%", self.cpu_value)),
            &self.theme);
        let mem_section = bordered_container(
            text(format!("Memory Usage: {:.2}GB/{:.2}GB ", self.mem_value, self.max_mem)), 
            &self.theme);
        let gpu_section = bordered_container(
            text(format!("Gpu Usage: {:.2}%", self.gpu_value.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" "))),
            &self.theme);
        let disk_section = bordered_container(
            text(format!("{}", self.disk_value)), 
        &self.theme);
        let network_section = bordered_container(
            text(format!(
                "Network Transmit: {:.2}kbps \n Network Receive: {:.2}kbps ",
                self.network_value.get(0).unwrap_or(&0.0),
                self.network_value.get(1).unwrap_or(&0.0)
            )), 
            &self.theme);
        let c1 = column![
            cpu_section,
            vertical_space().height(180),
            mem_section,
        ]
        .padding(20)
        .align_x(Center);
        let c2 = column![
            gpu_section,
            vertical_space().height(180),
            disk_section
        ]
        .padding(20)
        .align_x(Center);
        let r2 = row![
            c1,
            horizontal_space().width(150),
            c2,
        ];
        let r3 = row![
            network_section,
        ];
        let c = column![
            mb,
            vertical_space().height(20),
            r2,
            vertical_space().height(90),
            r3,
        ]
        .padding(20)
        .align_x(Center);

        fn back_style(theme: &iced::Theme) -> container::Style {
            container::Style {
                background: Some(theme.extended_palette().primary.base.color.into()),
                ..Default::default()
            }
        }
        let back = container(c)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(back_style);

        back.into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
fn debug_button(label: &str) -> button::Button<Message, iced::Theme, iced::Renderer> {
    labeled_button(label, Message::Debug(label.into())).width(Length::Fill)
}
fn debug_button_s(label: &str) -> button::Button<Message, iced::Theme, iced::Renderer> {
    labeled_button(label, Message::Debug(label.into())).width(Length::Shrink)
}

fn labeled_button(
    label: &str,
    msg: Message,
) -> button::Button<Message, iced::Theme, iced::Renderer> {
    base_button(text(label).align_y(alignment::Vertical::Center), msg)
}
fn base_button<'a>(
    content: impl Into<Element<'a, Message>>,
    msg: Message,
) -> button::Button<'a, Message> {
    button(content)
        .padding([4, 8])
        .style(iced::widget::button::primary)
        .on_press(msg)
}
fn color_button<'a>(
    color: impl Into<Color>,
) -> button::Button<'a, Message, iced::Theme, iced::Renderer> {
    let color = color.into();
    base_button(circle(color), Message::ColorChange(color))
}
fn circle(color: Color) -> quad::Quad {
    let radius = 10.0;

    quad::Quad {
        quad_color: color.into(),
        inner_bounds: InnerBounds::Square(radius * 2.0),
        quad_border: Border {
            radius: Radius::new(radius),
            ..Default::default()
        },
        height: Length::Fixed(20.0),
        ..Default::default()
    }
}
fn bordered_container<'a>(
    content: impl Into<Element<'a, Message>>,
    theme: &iced::Theme,
) -> iced::widget::container::Container<'a, Message> {
    let background_color = theme.extended_palette().background.base.color;
    iced::widget::container(content)
        .width(Length::Fill)
        .height(Length::Shrink)
        .padding(10)
        .style(move |_| iced::widget::container::Style {
            background: Some(background_color.into()),
            border: (iced::Border {
                width: 2.0,
                color: Color::BLACK,
                radius: 5.0.into(),
            }),
            shadow: Default::default(),
            text_color: None,
        })
}