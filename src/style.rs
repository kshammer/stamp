use iced::{container, Background, Color};


pub struct Container; 

impl container::StyleSheet for Container {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::BLACK)),
            text_color: Some(Color::WHITE),
            ..container::Style::default()
        }
    }
}

pub struct player_card;

impl container::StyleSheet for player_card {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::from_rgb(1.0,0.0, 0.0))),
            text_color: Some(Color::WHITE),
            border_color: Color::WHITE,
            ..container::Style::default()
        }
    }
}