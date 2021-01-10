use iced::Command;
use iced::Application;
use iced::Subscription;
use calculator::parse;
use iced::{executor, text_input, Align, Column, TextInput, Element, Settings, Text};

fn main() {
    std::env::set_var("FONTCONFIG_FILE", "/etc/fonts");
    Counter::run(Settings{
        window: iced::window::Settings{
            size: (550, 150),
            ..Default::default()
        },
        ..Default::default()
    }).unwrap();
}

#[derive(Default)]
struct Counter {
    // The input value
    input_value: String,
    // The calculation_result
    calculation_result: String,

    input: text_input::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateTask,
    EventOccurred,
    InputChanged(String),
}


impl Application for Counter {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();


    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("QuickCalc")
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::InputChanged(value) => {
                self.input_value = value.to_string();
                let res = parse(&value);
                if let Ok(mut res) = res {
                    let res = res.calculate();
                    if res.floor() == res {
                        self.calculation_result = format!("{:.0}", res) ;
                    }else{
                        self.calculation_result = format!("{:.2}", res) ;
                    }
                }
            }
            Message::EventOccurred => {
            }
            Message::CreateTask => {
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        // iced_native::subscription::events().map(Message::EventOccurred)

        Subscription::none()
    }

    fn view(&mut self) -> Element<Message> {
        // let hehehe = unsafe{ std::mem::transmute::<&mut Counter, &'static Counter>(self) };
        self.input.focus();
        let input = TextInput::new(
                    &mut self.input,
                    "What needs to be calculated, sir?",
                    &self.input_value,
                    Message::InputChanged,
                )
                .padding(15)
                .size(30)
                .on_submit(Message::CreateTask);


        // use std::thread;
        // thread::spawn(move || {
        //     loop {
        //         use std::{time};
        //         let ten_millis = time::Duration::from_millis(1000);
        //         thread::sleep(ten_millis);   
        //         dbg!((*hehehe).input.is_focused());
        //     }
        // });

        Column::new()
            .padding(20)
            .align_items(Align::Center)
            .push(input)
            .push(Text::new(self.calculation_result.to_string()).size(50))
            .into()
    }
}


