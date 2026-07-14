use std::process::{Child, Command};

use image::imageops::FilterType;
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
use tray_icon::{
    Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem},
};

const ICON_SIZE: u32 = 36;
const ICON_CROP_X: u32 = 180;
const ICON_CROP_Y: u32 = 104;
const ICON_CROP_SIZE: u32 = 976;

enum UserEvent {
    Tray(TrayIconEvent),
    Menu(MenuEvent),
}

struct App {
    tray_icon: Option<TrayIcon>,
    caffeinate: Option<Child>,
    inactive_icon: Icon,
    active_icon: Icon,
    quit_menu_item: MenuItem,
}

impl App {
    fn new() -> Self {
        Self {
            tray_icon: None,
            caffeinate: None,
            inactive_icon: load_icon(include_bytes!(
                "../resources/icons/cup-inactive-template.png"
            )),
            active_icon: load_icon(include_bytes!("../resources/icons/cup-active-template.png")),
            quit_menu_item: MenuItem::new("Quit Steamy", true, None),
        }
    }

    fn initialize_tray(&mut self) {
        let menu =
            Menu::with_items(&[&self.quit_menu_item]).expect("tray menu could not be created");

        self.tray_icon = Some(
            TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_menu_on_left_click(false)
                .with_menu_on_right_click(true)
                .with_icon(self.inactive_icon.clone())
                .with_icon_as_template(true)
                .with_tooltip("Steamy")
                .build()
                .expect("tray icon could not be created"),
        );
    }

    fn toggle_keep_awake(&mut self) {
        if let Some(mut child) = self.caffeinate.take() {
            match stop_keep_awake(&mut child) {
                Ok(()) => {
                    println!("caffeinate stopped");
                    self.set_icon(&self.inactive_icon);
                }
                Err(error) => {
                    eprintln!("could not stop caffeinate: {error}");
                    self.caffeinate = Some(child);
                }
            }
        } else {
            match start_keep_awake() {
                Ok(child) => {
                    println!("caffeinate started with PID {}", child.id());
                    self.caffeinate = Some(child);
                    self.set_icon(&self.active_icon);
                }
                Err(error) => {
                    eprintln!("could not start caffeinate: {error}");
                }
            }
        }
    }

    fn is_quit_event(&self, event: &MenuEvent) -> bool {
        event.id() == self.quit_menu_item.id()
    }

    fn shutdown(&mut self) {
        let Some(mut child) = self.caffeinate.take() else {
            return;
        };

        if let Err(error) = stop_keep_awake(&mut child) {
            eprintln!("could not stop caffeinate during shutdown: {error}");
        }
    }

    fn set_icon(&self, icon: &Icon) {
        let Some(tray_icon) = &self.tray_icon else {
            return;
        };

        if let Err(error) = tray_icon.set_icon_with_as_template(Some(icon.clone()), true) {
            eprintln!("could not update tray icon: {error}");
        }
    }
}

fn main() {
    let mut app = App::new();

    let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    event_loop.set_activation_policy(ActivationPolicy::Accessory);

    let event_loop_proxy = event_loop.create_proxy();

    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = event_loop_proxy.send_event(UserEvent::Tray(event));
    }));

    let event_loop_proxy = event_loop.create_proxy();

    MenuEvent::set_event_handler(Some(move |event| {
        let _ = event_loop_proxy.send_event(UserEvent::Menu(event));
    }));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                app.initialize_tray();
            }
            Event::UserEvent(UserEvent::Tray(TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            })) => {
                app.toggle_keep_awake();
            }
            Event::UserEvent(UserEvent::Menu(event)) if app.is_quit_event(&event) => {
                app.shutdown();
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

fn load_icon(bytes: &[u8]) -> Icon {
    // Resize once at startup; pre-size the source art if startup profiling warrants it.
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .expect("icon could not be decoded")
        // Use the same crop for both states so the cup does not jump when toggled.
        .crop_imm(ICON_CROP_X, ICON_CROP_Y, ICON_CROP_SIZE, ICON_CROP_SIZE)
        .resize_exact(ICON_SIZE, ICON_SIZE, FilterType::Lanczos3)
        .into_rgba8();

    let (width, height) = image.dimensions();

    Icon::from_rgba(image.into_raw(), width, height).expect("tray icon could not be created")
}

fn start_keep_awake() -> std::io::Result<Child> {
    let app_pid = std::process::id().to_string();

    Command::new("/usr/bin/caffeinate")
        .args(["-d", "-i", "-w", app_pid.as_str()])
        .spawn()
}

fn stop_keep_awake(child: &mut Child) -> std::io::Result<()> {
    child.kill()?;
    child.wait()?;

    Ok(())
}
