use futures::stream::StreamExt as _;
use futures::stream_select;
use gdk::Display;
use gio::prelude::{ApplicationExt as _, ApplicationExtManual as _};
use gtk::prelude::{ButtonExt as _, GtkWindowExt as _, WidgetExt as _};
use gtk::{
    Align, Application, ApplicationWindow, Button, ConstraintGuide, CssProvider, Label,
    StyleContext,
};
use springsteel::{add_constraint, glib_run_future, ConstraintView, ImpulseStream};
use std::future::ready;

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
    let increment = Button::with_label("+");
    increment.connect_clicked(increments.triggerer());

    let decrements = ImpulseStream::new();
    let decrement = Button::with_label("-");
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

    let content = ConstraintView::new();
    content.set_widget_name("content");
    let content_layout = content.layout();

    display.set_parent(&content);
    increment.set_parent(&content);
    decrement.set_parent(&content);

    let content_body = ConstraintGuide::new();
    let controls_display_spacer = ConstraintGuide::new();
    content_layout.add_guide(&content_body);
    content_layout.add_guide(&controls_display_spacer);

    add_constraint!(content_layout, content_body.top == top + 20.0);
    add_constraint!(content_layout, content_body.left == left + 20.0);
    add_constraint!(content_layout, right == content_body.right + 20.0);
    add_constraint!(content_layout, bottom == content_body.bottom + 20.0);

    add_constraint!(content_layout, increment.top == content_body.top);
    add_constraint!(content_layout, increment.left == content_body.left);
    add_constraint!(
        content_layout,
        increment.right == controls_display_spacer.left
    );

    add_constraint!(content_layout, increment.width == increment.height);

    add_constraint!(content_layout, decrement.top == increment.bottom + 10.0);

    add_constraint!(content_layout, decrement.bottom == content_body.bottom);
    add_constraint!(content_layout, decrement.left == content_body.left);
    add_constraint!(
        content_layout,
        decrement.right == controls_display_spacer.left
    );

    add_constraint!(content_layout, increment.height == decrement.height);

    add_constraint!(content_layout, controls_display_spacer.width == 10.0);

    add_constraint!(content_layout, display.top == content_body.top);
    add_constraint!(
        content_layout,
        display.left == controls_display_spacer.right
    );
    add_constraint!(content_layout, display.right == content_body.end);
    add_constraint!(content_layout, display.bottom == content_body.bottom);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("hi")
        .child(&content)
        .build();

    window.present();
}
