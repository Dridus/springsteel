use futures::stream::StreamExt as _;
use futures::stream_select;
use gdk::Display;
use gtk::prelude::{
    ApplicationExt as _, ApplicationExtManual as _, BoxExt as _, ButtonExt as _, GtkWindowExt as _,
};
use gtk::{
    Align, Application, ApplicationWindow, Box, Button, CssProvider, Label, Orientation,
    StyleContext,
};
use std::future::ready;
use springsteel::{ImpulseStream, glib_run_future};

const APP_ID: &str = "com.dridus.springsteel-workbench";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_startup(|_| load_css());
    app.connect_activate(build_ui);
    app.run();
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(
        b"
        label#display {
            font-weight: bold;
            font-size: 7em;
        }

        button {
            font-weight: bold;
            font-size: 2em;
        }
    ",
    );

    StyleContext::add_provider_for_display(
        &Display::default().expect("Display::default"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &Application) {
    let display = Label::builder()
        .label(&"0")
        .name("display")
        .halign(Align::End)
        .build();

    let increments = ImpulseStream::new();
    let increment = Button::builder().label("+").build();
    increment.connect_clicked(increments.triggerer());

    let decrements = ImpulseStream::new();
    let decrement = Button::builder().label("-").build();
    decrement.connect_clicked(decrements.triggerer());

    let deltas = stream_select!(increments.map(|()| 1), decrements.map(|()| -1));
    let count = deltas.scan(0, |s, d| {
        *s += d;
        ready(Some(*s))
    });

    let display_for_future = display.clone();
    let display_count = count.for_each(move |c| {
        display_for_future.set_text(&*format!("{}", c));
        ready(())
    });

    glib_run_future(display_count);

    let controls = Box::builder()
        .orientation(Orientation::Vertical)
        .homogeneous(true)
        .spacing(8)
        .build();
    controls.append(&increment);
    controls.append(&decrement);

    let content = Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .spacing(12)
        .build();
    content.append(&controls);
    content.append(&display);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("hi")
        .child(&content)
        .build();

    window.present();
}
