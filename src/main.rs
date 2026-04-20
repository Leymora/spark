#![allow(unused)]
mod config;
mod custom_button;

use custom_button::CustomButton;

use serial2::COMMON_BAUD_RATES;
use serialport;
use smol;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::thread;
use std::time::Duration;

use std::cell::Cell;
use std::cell::RefCell;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;

use colored::Colorize;

use adw::prelude::*;
use adw::{
    AboutDialog, ActionRow, Application, ApplicationWindow, Banner, ButtonRow, ComboRow, Dialog,
    EntryRow, HeaderBar, StatusPage, StyleManager, SwitchRow, ToolbarStyle, ToolbarView, Window,
    gio, glib,
};
use gtk::{
    Adjustment, Box, CheckButton, CssProvider, Label, ListBox, ListView, MenuButton, Orientation,
    PolicyType, PopoverMenu, PopoverMenuFlags, ScrolledWindow, SelectionMode, SelectionModel,
    Stack, StackPage, StackSwitcher, StringList, TextBuffer, TextView, Viewport, gdk::Display,
    gio::MenuModel,
};

use crate::config::COLOR_THEME;

const APP_ID: &str = "dev.labellum.spark";

fn main() -> glib::ExitCode {
    // Disable DTR with "stty -F /dev/ttyACM0 -hupcl" to hopefully stop Arduino from resetting when a serial connection is opened
    let _process = Command::new("stty")
        .arg("-F")
        .arg("/dev/ttyACM0")
        .arg("-hupcl")
        .status();

    // Register and include resources
    gio::resources_register_include!("resources/resources.gresource")
        .expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| load_css());
    // Connect to "activate" signal of "app"
    app.connect_activate(build_ui);

    println!("\n\x1b[30;43mWelcome to Spark 󱐋\x1b[0m\n");
    app.run();

    println!("\n🗑️ Deleting temp files...\n");

    let _kill_socat = Command::new("kill").arg("socat").status();

    let _delete_tmp_folder = Command::new("rm")
        .arg("-fr")
        .arg("/tmp/dev.labellum.spark/")
        .spawn();

    glib::ExitCode::SUCCESS
}

fn load_css() {
    // Load the CSS file and add it to the provider
    let provider = CssProvider::new();
    provider.load_from_resource("/dev/labellum/spark/style.css");

    // Add the provider to the default screen
    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_startup(app: &Application) {
    let status_page = StatusPage::builder()
        .title("Choose Serial Port")
        .description("If the list is empty, check your connections")
        .build();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Spark")
        .content(&status_page)
        .build();

    window.present();
}

fn build_ui(app: &Application) {
    // let res = gio::Resource::load("resources/resources.gresource").unwrap();
    // gio::resources_register(&res);

    let current_dir = env::current_dir().unwrap();
    let dir_display: String = current_dir.display().to_string();
    println!("{}", dir_display);

    // Disable DTR with "stty -F /dev/ttyACM0 -hupcl"
    let _process = Command::new("stty")
        .arg("-F")
        .arg("/dev/ttyACM0")
        .arg("-hupcl")
        .status()
        .expect("shidd");

    println!("\n📦 Creating temp files...\n");

    let _create_tmp_dir = Command::new("mkdir").arg("/tmp/dev.labellum.spark").spawn();

    let load_tmp_s1 = String::from_utf8(
        Command::new("mktemp")
            .arg("/tmp/dev.labellum.spark/tmp.XXX")
            .output()
            .unwrap()
            .stdout,
    );
    let load_tmp_s2 = String::from_utf8(
        Command::new("mktemp")
            .arg("/tmp/dev.labellum.spark/tmp.XXX")
            .output()
            .unwrap()
            .stdout,
    );

    let tmp_s1 = load_tmp_s1.clone();
    let tmp_s2 = load_tmp_s2.clone();

    // Create a dummy serialport (/dev/pts/1) to connect to so the program doesn't crash on startup
    let _dummy_serial_port = Command::new("socat")
        .arg("-dd")
        .arg("-t0")
        .arg("pty,raw,echo=0,link=".to_owned() + tmp_s1.unwrap().as_str().trim())
        .arg("pty,raw,echo=0,link=".to_owned() + tmp_s2.unwrap().as_str().trim())
        .spawn()
        .expect("Failed to create temporary serial port! Is socat installed?");

    std::thread::sleep(Duration::from_millis(500));

    //let port = Rc::new(RefCell::new(serialport::new("/dev/ttyACM0", 9600).open().unwrap_or_else(serialport::Error{kind: serialport::ErrorKind::NoDevice, description: String::new()})));

    let ser_port = load_tmp_s1.clone();
    let baud = Rc::new(Cell::new(COMMON_BAUD_RATES[0]));

    let port = Rc::new(RefCell::new(
        serialport::new(ser_port.unwrap().as_str().trim(), baud.get())
            .open()
            .expect("Failed to open port"),
    ));

    let _icn_dev = gio::resources_lookup_data(
        "/dev/labellum/spark/data/icons/logo.png",
        gio::ResourceLookupFlags::NONE,
    )
    .expect("BRUH!!");

    let about_diag = AboutDialog::builder()
        .application_name("Spark")
        .developer_name("Vilhelm Hansson")
        .can_focus(true)
        .application_icon("/dev/labellum/spark/data/icons/logo.png")
        .version(config::VERSION)
        .build();

    let banner_no_path = Banner::builder()
        .title("⛔ Path not found. Could not connect.")
        .button_label("Close")
        .build();

    banner_no_path.connect_button_clicked(|banner_no_path| {
        banner_no_path.set_revealed(false);
    });

    // Make these checkboxes
    let theme_chooser_system = CustomButton::new();
    theme_chooser_system.add_css_class("circular");

    let theme_chooser_light = CustomButton::new();
    theme_chooser_light.add_css_class("circular");

    let theme_chooser_dark = CustomButton::new();
    theme_chooser_dark.add_css_class("circular");

    let row = Box::new(Orientation::Horizontal, 4);
    row.append(&theme_chooser_system);
    row.append(&theme_chooser_light);
    row.append(&theme_chooser_dark);

    let btn_settings = Label::builder().label("Settings").build();

    let btn_about = CustomButton::with_label("About");
    btn_about.add_css_class("flat");

    let list_row = ListBox::new();
    list_row.append(&row);
    list_row.append(&btn_settings);
    list_row.append(&btn_about);
    list_row.set_selection_mode(SelectionMode::None);
    //list_row.add_css_class("navigation-sidebar");

    let stack_mainmenu = Stack::new();
    stack_mainmenu.add_named(&list_row, Some("main"));

    let viewport_menu = Viewport::builder().child(&stack_mainmenu).build();

    let sw_menu = ScrolledWindow::new();
    sw_menu.set_child(Some(&viewport_menu));
    sw_menu.set_hscrollbar_policy(PolicyType::Never);
    sw_menu.set_vscrollbar_policy(PolicyType::Never);

    let menu_main = PopoverMenu::builder().child(&sw_menu).build();

    let btn_menu = MenuButton::new();
    //btn_menu.add_css_class("accent");
    btn_menu.set_icon_name("open-menu-symbolic");
    btn_menu.set_popover(Some(&menu_main));
    btn_menu.connect_activate(|menu_main| {
        menu_main.display();
    });

    // Create headerbar for the window
    let header_bar = HeaderBar::builder()
        .decoration_layout("icon:menu,minimize,close")
        .build();
    header_bar.pack_end(&btn_menu);

    let srow = SwitchRow::builder()
        .activatable(true)
        .title("Car Enabled")
        .subtitle("Enable or Disable your car")
        .build();

    let entry_port = EntryRow::builder()
        .width_request(480)
        .title("Serial Port")
        .valign(gtk::Align::BaselineFill)
        .hexpand(true)
        .input_purpose(gtk::InputPurpose::Terminal)
        .text("/dev/ttyACM0")
        .build();
    entry_port.add_css_class("monospace");

    let connect_button = CustomButton::with_label("Connect");
    connect_button.set_margin_top(12);
    connect_button.set_margin_bottom(12);
    connect_button.set_margin_start(12);
    connect_button.set_margin_end(12);
    if COLOR_THEME == config::ColorTheme::Accent {
        connect_button.add_css_class("suggested-action");
    } else if COLOR_THEME == config::ColorTheme::Sparking {
        connect_button.add_css_class("yel");
    }
    connect_button.set_halign(gtk::Align::End);
    // Connect to "clicked" signal of "button"
    let c_baud = baud.clone();
    let c_entry_port = entry_port.clone();
    let c_port = port.clone();
    let c_tmp_port = load_tmp_s2.clone();
    let connected = Cell::new(false);
    let c_banner = banner_no_path.clone();
    connect_button.connect_clicked(move |connect_button| {
        if connected.get() == false {
            let test_path = &c_entry_port.text().to_string();
            let check_test_path = Path::new(test_path).try_exists();

            if check_test_path.is_ok_and(|x| x == true) {
                connect_button.set_css_classes(&["destructive-action"]);
                connect_button.set_label("Disconnect");
                c_port.replace(
                    serialport::new(c_entry_port.text().as_str(), c_baud.get())
                        .open()
                        .expect("Failed to open port"),
                );
                connected.set(true);
            } else {
                c_banner.set_revealed(true);
            }
        } else {
            if COLOR_THEME == config::ColorTheme::Accent {
                connect_button.set_css_classes(&["suggested-action"]);
            } else if COLOR_THEME == config::ColorTheme::Sparking {
                connect_button.set_css_classes(&["yel"]);
            }
            connect_button.set_label("Connect");
            c_port.replace(
                serialport::new(c_tmp_port.as_ref().unwrap().as_str().trim(), c_baud.get())
                    .open()
                    .expect("Failed to open port"),
            );
            connected.set(false);
        }
    });

    let connect_box = Box::new(Orientation::Horizontal, 16);
    connect_box.set_focusable(false);
    //send_box.add_css_class("linked");
    connect_box.append(&entry_port);
    connect_box.append(&connect_button);
    //send_box.set_halign(gtk::Align::End);

    let strings = [
        "4800", "9600", "19200", "38400", "57600", "115200", "230400", "460800", "500000",
        "576000", "921600", "1000000", "1500000", "2000000",
    ];

    let baud_box_strings = StringList::new(&strings);

    let baud_box = ComboRow::builder()
        .title("Baudrate")
        .subtitle("Select which baudrate to use")
        .build();
    baud_box.set_model(Some(&baud_box_strings));
    baud_box.set_use_subtitle(false);

    if COLOR_THEME == config::ColorTheme::Accent {
        baud_box.add_css_class("acc");
    } else if COLOR_THEME == config::ColorTheme::Sparking {
        baud_box.add_css_class("pur");
    }

    //baud_box.set_halign(gtk::Align::Start);
    baud_box.connect_selected_notify(glib::clone!(
        #[weak]
        baud,
        move |baud_box| {
            baud.set(baud_box.selected());
            baud.set(COMMON_BAUD_RATES[baud.get() as usize]);
        }
    ));

    let ser_buffer = TextBuffer::new(None);

    let text_box = TextView::builder()
        .editable(false)
        .accepts_tab(false)
        .can_focus(false)
        .can_target(false)
        .justification(gtk::Justification::Left)
        .indent(4)
        .build();
    //text_box.add_css_class("monospace");
    text_box.set_width_request(640);
    text_box.set_height_request(480);
    text_box.set_buffer(Some(&ser_buffer));

    let entry_textbox = EntryRow::builder()
        .width_request(480)
        .title("Command")
        .valign(gtk::Align::BaselineFill)
        .hexpand(true)
        .input_purpose(gtk::InputPurpose::Terminal)
        .build();

    let button = CustomButton::with_label("Send");
    button.set_margin_top(12);
    button.set_margin_bottom(12);
    button.set_margin_start(12);
    button.set_margin_end(12);
    if COLOR_THEME == config::ColorTheme::Accent {
        button.add_css_class("suggested-action");
    } else if COLOR_THEME == config::ColorTheme::Sparking {
        button.add_css_class("yel");
    }
    button.set_halign(gtk::Align::End);
    // Connect to "clicked" signal of "button"
    let xd = entry_textbox.clone();
    button.connect_clicked(move |_| {
        send_serial_data(
            port.borrow_mut().by_ref(),
            baud.get(),
            xd.text().as_str(),
            &ser_buffer,
        )
    });

    let send_box = Box::new(Orientation::Horizontal, 16);
    send_box.set_focusable(false);
    //send_box.add_css_class("linked");
    send_box.append(&entry_textbox);
    send_box.append(&button);
    //send_box.set_halign(gtk::Align::End);

    let list = ListBox::builder()
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .selection_mode(SelectionMode::None)
        .css_classes(vec![String::from("boxed-list-separate")])
        .build();
    list.append(&connect_box);
    list.append(&text_box);
    list.append(&baud_box);
    list.append(&send_box);

    let content = Box::new(Orientation::Vertical, 0);
    content.append(&list);

    let viewport_main = Viewport::builder().build();
    viewport_main.set_child(Some(&content));
    viewport_main.set_overflow(gtk::Overflow::Visible);

    let tool_bar = ToolbarView::builder()
        .bottom_bar_style(ToolbarStyle::Flat)
        .build();

    tool_bar.add_top_bar(&header_bar);
    tool_bar.add_top_bar(&banner_no_path); //Works but causes white bar on top of header_bar
    tool_bar.set_content(Some(&viewport_main));

    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Spark")
        .content(&tool_bar)
        .build();

    if config::DEV_MODE == true {
        header_bar.add_css_class("dev");
        window.add_css_class("dev");
        println!("\n🚧 Running in Dev mode 🚧\n")
    }

    // Present window
    window.present();

    btn_about.connect_clicked(move |_| {
        about_diag.present(Some(&window));
    });

    fn send_serial_data(
        portt: &mut std::boxed::Box<dyn serialport::SerialPort>,
        baudd: u32,
        cmd: &str,
        txtbuffer: &TextBuffer,
    ) {
        portt
            .set_baud_rate(baudd)
            .expect("Failed to set a baudrate of {baudd}");
        let output = cmd.as_bytes();
        portt
            .write(output)
            .expect("Failed to write to serial port ");
        println!(
            "📨 Sent command \x1b[32;3m{:?}\x1b[0m with a baudrate of \x1b[33;1m{baudd}\x1b[0m to \x1b[30;44;1m{}\x1b[0m",
            str::from_utf8(output).unwrap(),
            portt.name().unwrap()
        );

        // Give the connected serial port some time to proccess the command before reading the answer
        std::thread::sleep(Duration::from_millis(1200));

        let mut reader = BufReader::new(portt);
        let mut my_str = String::new();
        reader.read_line(&mut my_str).unwrap_or_default();
        txtbuffer.insert_at_cursor(&my_str);
        println!("{}", my_str);
    }
}
