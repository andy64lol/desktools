mod desktop;
mod settings;

use desktop::{COMMON_CATEGORIES, DesktopEntry, list_entries};
use iced::widget::{
    button, checkbox, column, container, pick_list, row, rule, scrollable, text, text_input,
};
use iced::{
    Alignment, Background, Border, Color, Element, Fill, Font, Subscription, Task, Theme, theme,
    window,
};
use settings::{Settings, ThemeMode};
use std::collections::HashMap;
use std::path::PathBuf;

fn main() -> iced::Result {
    iced::daemon(App::new, update, view)
        .title(title)
        .theme(|app: &App, _window| app_theme(app.colors()))
        .subscription(subscription)
        .run()
}

#[derive(Debug, Clone, Copy)]
struct Colors {
    bg: Color,
    surface: Color,
    ink: Color,
    line: Color,
    border: Color,
    cyan: Color,
    magenta: Color,
}

const LIGHT: Colors = Colors {
    bg: Color::from_rgb8(0xE8, 0xE9, 0xEA),
    surface: Color::from_rgb8(0xF3, 0xF4, 0xF4),
    ink: Color::from_rgb8(0x1E, 0x21, 0x24),
    line: Color::from_rgb8(0xC7, 0xCA, 0xCC),
    border: Color::from_rgb8(0x9E, 0xA3, 0xA6),
    cyan: Color::from_rgb8(0xDB, 0xE9, 0xE9),
    magenta: Color::from_rgb8(0xEB, 0xDF, 0xE7),
};

const DARK: Colors = Colors {
    bg: Color::from_rgb8(0x17, 0x19, 0x1B),
    surface: Color::from_rgb8(0x20, 0x23, 0x25),
    ink: Color::from_rgb8(0xE6, 0xE8, 0xE9),
    line: Color::from_rgb8(0x34, 0x38, 0x3B),
    border: Color::from_rgb8(0x4A, 0x4F, 0x52),
    cyan: Color::from_rgb8(0x2B, 0x3A, 0x3A),
    magenta: Color::from_rgb8(0x3A, 0x2F, 0x36),
};

const HAIRLINE: f32 = 1.0;
const EDGE: f32 = 1.0;
const CARD_WIDTH: f32 = 308.0;

const BOLD: Font = Font {
    weight: iced::font::Weight::Bold,
    ..Font::DEFAULT
};

const MONO: Font = Font::MONOSPACE;

fn app_theme(colors: Colors) -> Theme {
    Theme::custom(
        "desktools",
        iced::theme::Palette {
            background: colors.bg,
            text: colors.ink,
            primary: colors.ink,
            success: Color::from_rgb8(0x5C, 0x7A, 0x7A),
            warning: Color::from_rgb8(0x8C, 0x8C, 0x6E),
            danger: Color::from_rgb8(0x8C, 0x6E, 0x86),
        },
    )
}

fn dim(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

fn plain(colors: Colors, bg: Color, border: f32) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        text_color: Some(colors.ink),
        background: Some(Background::Color(bg)),
        border: Border {
            color: colors.border,
            width: border,
            radius: 0.0.into(),
        },
        shadow: iced::Shadow::default(),
        snap: true,
    }
}

fn hairline(colors: Colors) -> impl Fn(&Theme) -> rule::Style {
    move |_theme| rule::Style {
        color: colors.line,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}

fn button_style(colors: Colors) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let base = button::Style {
            background: Some(Background::Color(colors.surface)),
            text_color: colors.ink,
            border: Border {
                color: colors.border,
                width: EDGE,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            snap: true,
        };

        match status {
            button::Status::Active => base,
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(colors.cyan)),
                ..base
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(colors.magenta)),
                ..base
            },
            button::Status::Disabled => button::Style {
                text_color: dim(colors.ink, 0.35),
                border: Border {
                    color: dim(colors.border, 0.5),
                    ..base.border
                },
                ..base
            },
        }
    }
}

fn primary_button(colors: Colors) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let base = button::Style {
            background: Some(Background::Color(colors.ink)),
            text_color: colors.surface,
            border: Border {
                color: colors.ink,
                width: EDGE,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            snap: true,
        };

        match status {
            button::Status::Active => base,
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(colors.cyan)),
                text_color: colors.ink,
                ..base
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(colors.magenta)),
                text_color: colors.ink,
                ..base
            },
            button::Status::Disabled => button::Style {
                background: Some(Background::Color(dim(colors.ink, 0.3))),
                ..base
            },
        }
    }
}

fn danger_button(colors: Colors) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let base = button::Style {
            background: Some(Background::Color(colors.surface)),
            text_color: colors.ink,
            border: Border {
                color: colors.border,
                width: EDGE,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            snap: true,
        };

        match status {
            button::Status::Active => base,
            button::Status::Hovered | button::Status::Pressed => button::Style {
                background: Some(Background::Color(colors.magenta)),
                ..base
            },
            button::Status::Disabled => button::Style {
                text_color: dim(colors.ink, 0.35),
                border: Border {
                    color: dim(colors.border, 0.5),
                    ..base.border
                },
                ..base
            },
        }
    }
}

fn card_style(colors: Colors) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let base = button::Style {
            background: Some(Background::Color(colors.surface)),
            text_color: colors.ink,
            border: Border {
                color: colors.border,
                width: EDGE,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            snap: true,
        };

        match status {
            button::Status::Active => base,
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(colors.cyan)),
                ..base
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(colors.magenta)),
                ..base
            },
            button::Status::Disabled => base,
        }
    }
}

fn input_style(colors: Colors) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_theme, status| {
        let base = text_input::Style {
            background: Background::Color(colors.surface),
            border: Border {
                color: colors.border,
                width: EDGE,
                radius: 0.0.into(),
            },
            icon: colors.ink,
            placeholder: dim(colors.ink, 0.4),
            value: colors.ink,
            selection: colors.magenta,
        };

        match status {
            text_input::Status::Active => base,
            text_input::Status::Hovered => text_input::Style {
                background: Background::Color(colors.cyan),
                ..base
            },
            text_input::Status::Focused { .. } => text_input::Style {
                background: Background::Color(colors.cyan),
                ..base
            },
            text_input::Status::Disabled => text_input::Style {
                value: dim(colors.ink, 0.35),
                ..base
            },
        }
    }
}

fn pick_list_style(colors: Colors) -> impl Fn(&Theme, pick_list::Status) -> pick_list::Style {
    move |_theme, status| {
        let base = pick_list::Style {
            text_color: colors.ink,
            placeholder_color: dim(colors.ink, 0.4),
            handle_color: colors.ink,
            background: Background::Color(colors.surface),
            border: Border {
                color: colors.border,
                width: EDGE,
                radius: 0.0.into(),
            },
        };

        match status {
            pick_list::Status::Active => base,
            pick_list::Status::Hovered => pick_list::Style {
                background: Background::Color(colors.cyan),
                ..base
            },
            pick_list::Status::Opened { .. } => pick_list::Style {
                background: Background::Color(colors.cyan),
                ..base
            },
        }
    }
}

fn checkbox_style(colors: Colors) -> impl Fn(&Theme, checkbox::Status) -> checkbox::Style {
    move |_theme, status| {
        let (is_checked, is_hovered, is_disabled) = match status {
            checkbox::Status::Active { is_checked } => (is_checked, false, false),
            checkbox::Status::Hovered { is_checked } => (is_checked, true, false),
            checkbox::Status::Disabled { is_checked } => (is_checked, false, true),
        };

        let (background, icon_color, border_color) = if is_disabled {
            (
                colors.surface,
                dim(colors.ink, 0.35),
                dim(colors.border, 0.5),
            )
        } else if is_checked && is_hovered {
            (colors.cyan, colors.ink, colors.border)
        } else if is_checked {
            (colors.ink, colors.surface, colors.ink)
        } else if is_hovered {
            (colors.cyan, colors.ink, colors.border)
        } else {
            (colors.surface, colors.ink, colors.border)
        };

        checkbox::Style {
            background: Background::Color(background),
            icon_color,
            border: Border {
                color: border_color,
                width: EDGE,
                radius: 0.0.into(),
            },
            text_color: Some(if is_disabled {
                dim(colors.ink, 0.35)
            } else {
                colors.ink
            }),
        }
    }
}

fn scroll_style(colors: Colors) -> impl Fn(&Theme, scrollable::Status) -> scrollable::Style {
    move |theme, status| {
        let mut style = scrollable::default(theme, status);

        let rail = scrollable::Rail {
            background: Some(Background::Color(colors.surface)),
            border: Border {
                color: colors.line,
                width: 1.0,
                radius: 0.0.into(),
            },
            scroller: scrollable::Scroller {
                background: Background::Color(colors.border),
                border: Border {
                    color: colors.border,
                    width: 0.0,
                    radius: 0.0.into(),
                },
            },
        };

        style.vertical_rail = rail;
        style.horizontal_rail = rail;
        style
    }
}

struct SettingsForm {
    applications_dir: String,
    desktop_dir: String,
    show_all: bool,
    theme: ThemeMode,
}

impl SettingsForm {
    fn from_settings(settings: &Settings) -> Self {
        SettingsForm {
            applications_dir: settings.applications_dir().display().to_string(),
            desktop_dir: settings.desktop_dir().display().to_string(),
            show_all: settings.show_all,
            theme: settings.theme,
        }
    }

    fn to_settings(&self) -> Settings {
        Settings {
            applications_dir: non_empty_path(&self.applications_dir),
            desktop_dir: non_empty_path(&self.desktop_dir),
            show_all: self.show_all,
            theme: self.theme,
        }
    }
}

fn non_empty_path(value: &str) -> Option<PathBuf> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

struct Home {
    directory: PathBuf,
    entries: Vec<DesktopEntry>,
    status: String,
    status_is_error: bool,
    show_settings: bool,
    settings_form: SettingsForm,
}

impl Home {
    fn refresh(&mut self, settings: &Settings) {
        self.entries = list_entries(&self.directory, settings.show_all);
    }

    fn info(&mut self, message: String) {
        self.status = message;
        self.status_is_error = false;
    }

    fn error(&mut self, message: String) {
        self.status = message;
        self.status_is_error = true;
    }
}

struct Editor {
    form: DesktopEntry,
    status: String,
    status_is_error: bool,
}

impl Editor {
    fn new(form: DesktopEntry) -> Self {
        let status = if form.path.is_some() {
            "Loaded. Edit the fields and hit Save.".to_string()
        } else {
            "New launcher. Fill the form and hit Save.".to_string()
        };

        Editor {
            form,
            status,
            status_is_error: false,
        }
    }

    fn info(&mut self, message: String) {
        self.status = message;
        self.status_is_error = false;
    }

    fn error(&mut self, message: String) {
        self.status = message;
        self.status_is_error = true;
    }

    fn validate(&self) -> Result<(), String> {
        if self.form.name.trim().is_empty() {
            return Err("A name is required.".to_string());
        }

        if self.form.exec.trim().is_empty() {
            return Err("A command to run is required.".to_string());
        }

        Ok(())
    }
}

struct App {
    settings: Settings,
    system_mode: theme::Mode,
    home_window: Option<window::Id>,
    home: Home,
    editors: HashMap<window::Id, Editor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Name,
    GenericName,
    Comment,
    Exec,
    Icon,
    WorkingDir,
    Categories,
    Keywords,
    StartupWmClass,
}

#[derive(Debug, Clone)]
enum Message {
    CloseRequested,
    CloseWindow,
    SystemMode(theme::Mode),
    Home(HomeMessage),
    Editor(window::Id, EditorMessage),
}

#[derive(Debug, Clone)]
enum HomeMessage {
    Open(usize),
    New,
    Reload,
    OpenSettings,
    CloseSettings,
    SettingsAppsDir(String),
    SettingsDesktopDir(String),
    SettingsShowAll(bool),
    SettingsTheme(ThemeMode),
    SaveSettings,
}

#[derive(Debug, Clone)]
enum EditorMessage {
    Field(Field, String),
    Terminal(bool),
    NoDisplay(bool),
    StartupNotify(bool),
    AddCategory(String),
    Save,
    PinToDesktop,
    Discard,
    Delete,
    OpenFolder,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let settings = Settings::load();
        let directory = settings.applications_dir();
        let entries = list_entries(&directory, settings.show_all);

        let (home_window, open_home) = window::open(window::Settings {
            size: iced::Size::new(1040.0, 720.0),
            min_size: Some(iced::Size::new(620.0, 440.0)),
            ..Default::default()
        });

        let app = App {
            home: Home {
                settings_form: SettingsForm::from_settings(&settings),
                directory,
                entries,
                status: "Ready. Hit New launcher to create one.".to_string(),
                status_is_error: false,
                show_settings: false,
            },
            home_window: Some(home_window),
            editors: HashMap::new(),
            settings,
            system_mode: theme::Mode::default(),
        };

        let system_theme = iced::system::theme().map(Message::SystemMode);

        (app, Task::batch([open_home.discard(), system_theme]))
    }

    fn colors(&self) -> Colors {
        match self.settings.theme {
            ThemeMode::Light => LIGHT,
            ThemeMode::Dark => DARK,
            ThemeMode::System => match self.system_mode {
                theme::Mode::Dark => DARK,
                _ => LIGHT,
            },
        }
    }
}

fn title(app: &App, id: window::Id) -> String {
    if let Some(editor) = app.editors.get(&id) {
        let name = if editor.form.name.trim().is_empty() {
            "New launcher".to_string()
        } else {
            editor.form.name.clone()
        };

        return format!("desktools — {name}");
    }

    "desktools".to_string()
}

fn subscription(_app: &App) -> Subscription<Message> {
    Subscription::batch([
        window::close_requests().map(|_| Message::CloseRequested),
        iced::system::theme_changes().map(Message::SystemMode),
    ])
}

fn open_editor(app: &mut App, form: DesktopEntry) -> Task<Message> {
    let (id, open) = window::open(window::Settings {
        size: iced::Size::new(1180.0, 800.0),
        min_size: Some(iced::Size::new(760.0, 540.0)),
        ..Default::default()
    });

    app.editors.insert(id, Editor::new(form));

    open.discard()
}

fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::CloseRequested | Message::CloseWindow => {
            return iced::exit();
        }
        Message::SystemMode(mode) => app.system_mode = mode,
        Message::Home(message) => return update_home(app, message),
        Message::Editor(id, message) => return update_editor(app, id, message),
    }

    Task::none()
}

fn update_home(app: &mut App, message: HomeMessage) -> Task<Message> {
    match message {
        HomeMessage::Open(index) => {
            let Some(entry) = app.home.entries.get(index).cloned() else {
                return Task::none();
            };

            if let Some(path) = entry.path.clone() {
                let already_open = app
                    .editors
                    .values()
                    .any(|editor| editor.form.path.as_ref() == Some(&path));

                if already_open {
                    app.home.info(format!(
                        "{} is already open in another window.",
                        entry.file_name()
                    ));
                    return Task::none();
                }
            }

            return open_editor(app, entry);
        }
        HomeMessage::New => {
            return open_editor(app, DesktopEntry::default());
        }
        HomeMessage::Reload => {
            app.home.refresh(&app.settings);
            app.home
                .info(format!("Found {} launcher(s).", app.home.entries.len()));
        }
        HomeMessage::OpenSettings => {
            app.home.settings_form = SettingsForm::from_settings(&app.settings);
            app.home.show_settings = true;
            app.home.info("Settings.".to_string());
        }
        HomeMessage::CloseSettings => {
            app.home.show_settings = false;
            app.home.info("Settings closed.".to_string());
        }
        HomeMessage::SettingsAppsDir(value) => app.home.settings_form.applications_dir = value,
        HomeMessage::SettingsDesktopDir(value) => app.home.settings_form.desktop_dir = value,
        HomeMessage::SettingsShowAll(value) => app.home.settings_form.show_all = value,
        HomeMessage::SettingsTheme(value) => app.home.settings_form.theme = value,
        HomeMessage::SaveSettings => {
            let settings = app.home.settings_form.to_settings();

            match settings.save() {
                Ok(()) => {
                    app.settings = settings;
                    app.home.settings_form = SettingsForm::from_settings(&app.settings);
                    app.home.directory = app.settings.applications_dir();
                    app.home.show_settings = false;
                    app.home.refresh(&app.settings);
                    app.home.info(format!(
                        "Settings saved. Using {}.",
                        app.home.directory.display()
                    ));
                }
                Err(error) => app.home.error(format!("Could not save settings: {error}")),
            }
        }
    }

    Task::none()
}

fn update_editor(app: &mut App, id: window::Id, message: EditorMessage) -> Task<Message> {
    let directory = app.settings.applications_dir();
    let desktop_dir = app.settings.desktop_dir();

    let Some(editor) = app.editors.get_mut(&id) else {
        return Task::none();
    };

    let mut refresh_home = false;

    match message {
        EditorMessage::Field(field, value) => match field {
            Field::Name => editor.form.name = value,
            Field::GenericName => editor.form.generic_name = value,
            Field::Comment => editor.form.comment = value,
            Field::Exec => editor.form.exec = value,
            Field::Icon => editor.form.icon = value,
            Field::WorkingDir => editor.form.working_dir = value,
            Field::Categories => editor.form.categories = value,
            Field::Keywords => editor.form.keywords = value,
            Field::StartupWmClass => editor.form.startup_wm_class = value,
        },
        EditorMessage::Terminal(value) => editor.form.terminal = value,
        EditorMessage::NoDisplay(value) => editor.form.no_display = value,
        EditorMessage::StartupNotify(value) => editor.form.startup_notify = value,
        EditorMessage::AddCategory(category) => {
            let current = editor
                .form
                .categories
                .trim()
                .trim_end_matches(';')
                .to_string();

            editor.form.categories = if current.is_empty() {
                format!("{category};")
            } else {
                format!("{current};{category};")
            };
        }
        EditorMessage::Save => {
            if let Err(error) = editor.validate() {
                editor.error(error);
            } else {
                match editor.form.save(&directory) {
                    Ok(path) => {
                        editor.info(format!("Saved to {}.", path.display()));
                        refresh_home = true;
                    }
                    Err(error) => editor.error(format!("Could not save: {error}")),
                }
            }
        }
        EditorMessage::Delete => {
            let Some(path) = editor.form.path.clone() else {
                editor.error("Nothing to delete, this launcher is not saved yet.".to_string());
                return Task::none();
            };

            match std::fs::remove_file(&path) {
                Ok(()) => {
                    editor.form = DesktopEntry::default();
                    editor.info(format!("Deleted {}.", path.display()));
                    refresh_home = true;
                }
                Err(error) => editor.error(format!("Could not delete: {error}")),
            }
        }
        EditorMessage::PinToDesktop => {
            if let Err(error) = editor.validate() {
                editor.error(error);
            } else {
                match editor.form.save(&directory) {
                    Ok(source) => {
                        refresh_home = true;

                        let target = desktop_dir.join(
                            source
                                .file_name()
                                .map(|name| name.to_os_string())
                                .unwrap_or_else(|| editor.form.file_name().into()),
                        );

                        let result = std::fs::create_dir_all(&desktop_dir)
                            .and_then(|()| std::fs::copy(&source, &target).map(|_| ()))
                            .and_then(|()| make_executable(&target));

                        match result {
                            Ok(()) => editor.info(format!("Pinned to {}.", target.display())),
                            Err(error) => {
                                editor.error(format!("Could not pin to desktop: {error}"))
                            }
                        }
                    }
                    Err(error) => editor.error(format!("Could not save: {error}")),
                }
            }
        }
        EditorMessage::Discard => match editor.form.path.clone() {
            Some(path) => match DesktopEntry::parse(&path) {
                Ok(entry) => {
                    editor.form = entry;
                    editor.info("Changes discarded.".to_string());
                }
                Err(error) => editor.error(format!("Could not reload the file: {error}")),
            },
            None => {
                editor.form = DesktopEntry::default();
                editor.info("Form cleared.".to_string());
            }
        },
        EditorMessage::OpenFolder => {
            if let Err(error) = std::fs::create_dir_all(&directory).and_then(|()| {
                std::process::Command::new("xdg-open")
                    .arg(&directory)
                    .spawn()
                    .map(|_| ())
            }) {
                editor.error(format!("Could not open the folder: {error}"));
            }
        }
    }

    if refresh_home {
        app.home.refresh(&app.settings);
    }

    Task::none()
}

fn make_executable(path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
    }

    Ok(())
}

fn view(app: &App, id: window::Id) -> Element<'_, Message> {
    if Some(id) == app.home_window {
        return home_view(app);
    }

    if let Some(editor) = app.editors.get(&id) {
        return editor_view(app, id, editor);
    }

    let colors = app.colors();

    container(column![])
        .width(Fill)
        .height(Fill)
        .style(plain(colors, colors.bg, 0.0))
        .into()
}

fn home_view(app: &App) -> Element<'_, Message> {
    let colors = app.colors();

    let body = if app.home.show_settings {
        settings_page(app, colors)
    } else {
        gallery(app, colors)
    };

    column![
        home_header(app, colors),
        body,
        status_bar(
            colors,
            &app.home.status,
            app.home.status_is_error,
            format!("{} launcher(s)", app.home.entries.len()),
        ),
    ]
    .width(Fill)
    .height(Fill)
    .into()
}

fn home_header(app: &App, colors: Colors) -> Element<'_, Message> {
    let (label, message) = if app.home.show_settings {
        ("Launchers", HomeMessage::CloseSettings)
    } else {
        ("Settings", HomeMessage::OpenSettings)
    };

    column![
        container(
            row![
                text("desktools").size(17).font(BOLD),
                text(format!("{} launcher(s)", app.home.entries.len()))
                    .size(12)
                    .color(dim(colors.ink, 0.55)),
                row![].width(Fill),
                button(text("Reload").size(12))
                    .on_press(Message::Home(HomeMessage::Reload))
                    .style(button_style(colors))
                    .padding([7, 12]),
                button(text(label).size(12))
                    .on_press(Message::Home(message))
                    .style(button_style(colors))
                    .padding([7, 12]),
                button(text("New launcher").size(12))
                    .on_press(Message::Home(HomeMessage::New))
                    .style(primary_button(colors))
                    .padding([7, 14]),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding([14, 18])
        .width(Fill)
        .style(plain(colors, colors.surface, 0.0)),
        rule::horizontal(HAIRLINE as u32).style(hairline(colors)),
    ]
    .spacing(0)
    .into()
}

fn gallery(app: &App, colors: Colors) -> Element<'_, Message> {
    if app.home.entries.is_empty() {
        return empty_gallery(app, colors);
    }

    let mut grid = column![].spacing(12).width(Fill);

    for (row_index, chunk) in app.home.entries.chunks(3).enumerate() {
        let mut line = row![].spacing(12).width(Fill);

        for (offset, entry) in chunk.iter().enumerate() {
            line = line.push(card(colors, row_index * 3 + offset, entry));
        }

        if chunk.len() < 3 {
            line = line.push(container(row![]).width(Fill));
        }

        grid = grid.push(line);
    }

    container(
        scrollable(container(grid).padding(20).width(Fill))
            .height(Fill)
            .style(scroll_style(colors)),
    )
    .width(Fill)
    .height(Fill)
    .style(plain(colors, colors.bg, 0.0))
    .into()
}

fn empty_gallery(app: &App, colors: Colors) -> Element<'_, Message> {
    let hint = if app.settings.show_all {
        "No .desktop files in this folder yet."
    } else {
        "Only launchers made with desktools appear here."
    };

    let message = container(
        column![
            text("Nothing here yet").size(16).font(BOLD),
            text("Hit New launcher to create one.")
                .size(12)
                .color(dim(colors.ink, 0.6)),
            text(hint).size(11).color(dim(colors.ink, 0.45)),
        ]
        .spacing(6)
        .align_x(Alignment::Center),
    )
    .padding([32, 44])
    .style(plain(colors, colors.surface, EDGE));

    container(message)
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .center_y(Fill)
        .style(plain(colors, colors.bg, 0.0))
        .into()
}

fn card(colors: Colors, index: usize, entry: &DesktopEntry) -> Element<'_, Message> {
    let name = if entry.name.trim().is_empty() {
        "(unnamed)".to_string()
    } else {
        entry.name.clone()
    };

    let description = if !entry.comment.trim().is_empty() {
        entry.comment.clone()
    } else if !entry.generic_name.trim().is_empty() {
        entry.generic_name.clone()
    } else if !entry.exec.trim().is_empty() {
        entry.exec.clone()
    } else {
        "No command".to_string()
    };

    let body = column![
        text(name).size(14).font(BOLD),
        text(description).size(12).color(dim(colors.ink, 0.62)),
        row![].height(Fill),
        text(entry.file_name())
            .size(10)
            .font(MONO)
            .color(dim(colors.ink, 0.45)),
    ]
    .spacing(4)
    .height(Fill);

    button(container(body).padding(12).height(Fill).clip(true))
        .on_press(Message::Home(HomeMessage::Open(index)))
        .style(card_style(colors))
        .width(CARD_WIDTH)
        .height(118)
        .padding(0)
        .into()
}

fn settings_page(app: &App, colors: Colors) -> Element<'_, Message> {
    let form = column![
        column![
            text("Settings").size(19).font(BOLD),
            text(format!("Stored in {}", settings::config_path().display()))
                .size(12)
                .color(dim(colors.ink, 0.55)),
        ]
        .spacing(2),
        section(
            colors,
            "Appearance",
            column![
                labeled(
                    colors,
                    "Theme",
                    pick_list(
                        [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark],
                        Some(app.home.settings_form.theme),
                        |mode| Message::Home(HomeMessage::SettingsTheme(mode)),
                    )
                    .width(Fill)
                    .padding([6, 9])
                    .text_size(13)
                    .style(pick_list_style(colors)),
                ),
                text("System follows your desktop's light or dark preference.")
                    .size(11)
                    .color(dim(colors.ink, 0.55)),
            ]
            .spacing(8),
        ),
        section(
            colors,
            "Storage",
            column![
                labeled(
                    colors,
                    "Applications directory",
                    text_input(
                        "/home/me/.local/share/applications",
                        &app.home.settings_form.applications_dir
                    )
                    .on_input(|value| Message::Home(HomeMessage::SettingsAppsDir(value)))
                    .padding([7, 9])
                    .size(13)
                    .style(input_style(colors)),
                ),
                labeled(
                    colors,
                    "Desktop directory",
                    text_input("/home/me/Desktop", &app.home.settings_form.desktop_dir)
                        .on_input(|value| Message::Home(HomeMessage::SettingsDesktopDir(value)))
                        .padding([7, 9])
                        .size(13)
                        .style(input_style(colors)),
                ),
                text("Leave empty to use the system defaults.")
                    .size(11)
                    .color(dim(colors.ink, 0.55)),
            ]
            .spacing(14),
        ),
        section(
            colors,
            "Listing",
            column![
                checkbox(app.home.settings_form.show_all)
                    .label("Show all .desktop files, including pre-existing apps")
                    .on_toggle(|value| Message::Home(HomeMessage::SettingsShowAll(value)))
                    .style(checkbox_style(colors))
                    .text_size(13),
                text("When off, only launchers created with desktools are listed.")
                    .size(11)
                    .color(dim(colors.ink, 0.55)),
            ]
            .spacing(8),
        ),
        row![
            button(text("Save settings").size(12))
                .on_press(Message::Home(HomeMessage::SaveSettings))
                .style(primary_button(colors))
                .padding([8, 16]),
            button(text("Cancel").size(12))
                .on_press(Message::Home(HomeMessage::CloseSettings))
                .style(button_style(colors))
                .padding([8, 14]),
        ]
        .spacing(8),
    ]
    .spacing(24)
    .max_width(680);

    container(scrollable(container(form).padding([24, 28]).center_x(Fill)).height(Fill))
        .width(Fill)
        .height(Fill)
        .style(plain(colors, colors.bg, 0.0))
        .into()
}

fn editor_view<'a>(app: &'a App, id: window::Id, editor: &'a Editor) -> Element<'a, Message> {
    let colors = app.colors();

    let trailing = match &editor.form.path {
        Some(path) => path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default(),
        None => "Not saved yet".to_string(),
    };

    column![
        editor_header(editor, colors),
        row![
            editor_form(app, id, editor),
            rule::vertical(HAIRLINE as u32).style(hairline(colors)),
            preview(editor, colors),
        ]
        .height(Fill),
        status_bar(colors, &editor.status, editor.status_is_error, trailing),
    ]
    .width(Fill)
    .height(Fill)
    .into()
}

fn editor_header(editor: &Editor, colors: Colors) -> Element<'_, Message> {
    let file_name = editor
        .form
        .path
        .as_ref()
        .and_then(|path| path.file_name())
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "untitled.desktop".to_string());

    let mode = if editor.form.path.is_some() {
        "Edit"
    } else {
        "New"
    };

    column![
        container(
            row![
                text("desktools").size(15).font(BOLD),
                text(file_name).size(13).color(dim(colors.ink, 0.55)),
                container(text(mode).size(10).font(BOLD))
                    .padding([2, 7])
                    .style(plain(colors, colors.cyan, EDGE)),
                row![].width(Fill),
                button(text("Close").size(11))
                    .on_press(Message::CloseWindow)
                    .style(button_style(colors))
                    .padding([5, 10]),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .padding([12, 16])
        .width(Fill)
        .style(plain(colors, colors.surface, 0.0)),
        rule::horizontal(HAIRLINE as u32).style(hairline(colors)),
    ]
    .spacing(0)
    .into()
}

fn editor_form<'a>(app: &'a App, id: window::Id, editor: &'a Editor) -> Element<'a, Message> {
    let colors = app.colors();

    let is_new = editor.form.path.is_none();
    let title = if is_new {
        "New launcher"
    } else {
        "Edit launcher"
    };

    let subtitle = match &editor.form.path {
        Some(path) => path.display().to_string(),
        None => format!("Saves to {}", app.settings.applications_dir().display()),
    };

    let form = column![
        column![
            text(title).size(19).font(BOLD),
            text(subtitle).size(12).color(dim(colors.ink, 0.55)),
        ]
        .spacing(2),
        section(
            colors,
            "Application",
            column![
                row![
                    labeled(
                        colors,
                        "Name",
                        text_input("My App", &editor.form.name)
                            .on_input(move |value| {
                                Message::Editor(id, EditorMessage::Field(Field::Name, value))
                            })
                            .padding([7, 9])
                            .size(13)
                            .style(input_style(colors)),
                    ),
                    labeled(
                        colors,
                        "Command",
                        text_input("my-app --flag %u", &editor.form.exec)
                            .on_input(move |value| {
                                Message::Editor(id, EditorMessage::Field(Field::Exec, value))
                            })
                            .padding([7, 9])
                            .size(13)
                            .style(input_style(colors)),
                    ),
                ]
                .spacing(14),
                row![
                    labeled(
                        colors,
                        "Icon",
                        text_input("icon name or /path/icon.png", &editor.form.icon)
                            .on_input(move |value| {
                                Message::Editor(id, EditorMessage::Field(Field::Icon, value))
                            })
                            .padding([7, 9])
                            .size(13)
                            .style(input_style(colors)),
                    ),
                    labeled(
                        colors,
                        "Working directory",
                        text_input("/home/me/projects", &editor.form.working_dir)
                            .on_input(move |value| {
                                Message::Editor(id, EditorMessage::Field(Field::WorkingDir, value))
                            })
                            .padding([7, 9])
                            .size(13)
                            .style(input_style(colors)),
                    ),
                ]
                .spacing(14),
            ]
            .spacing(14),
        ),
        section(
            colors,
            "Menu entry",
            column![
                labeled(
                    colors,
                    "Comment",
                    text_input("What does this app do?", &editor.form.comment)
                        .on_input(move |value| {
                            Message::Editor(id, EditorMessage::Field(Field::Comment, value))
                        })
                        .padding([7, 9])
                        .size(13)
                        .style(input_style(colors)),
                ),
                row![
                    labeled(
                        colors,
                        "Generic name",
                        text_input("Web browser, text editor...", &editor.form.generic_name)
                            .on_input(move |value| {
                                Message::Editor(id, EditorMessage::Field(Field::GenericName, value))
                            })
                            .padding([7, 9])
                            .size(13)
                            .style(input_style(colors)),
                    ),
                    labeled(
                        colors,
                        "Startup WM class",
                        text_input("my-app", &editor.form.startup_wm_class)
                            .on_input(move |value| {
                                Message::Editor(
                                    id,
                                    EditorMessage::Field(Field::StartupWmClass, value),
                                )
                            })
                            .padding([7, 9])
                            .size(13)
                            .style(input_style(colors)),
                    ),
                ]
                .spacing(14),
                labeled(
                    colors,
                    "Categories",
                    column![
                        text_input("Utility;Development;", &editor.form.categories)
                            .on_input(move |value| {
                                Message::Editor(id, EditorMessage::Field(Field::Categories, value))
                            })
                            .padding([7, 9])
                            .size(13)
                            .style(input_style(colors)),
                        pick_list(COMMON_CATEGORIES, None::<&str>, move |category| {
                            Message::Editor(id, EditorMessage::AddCategory(category.to_string()))
                        })
                        .placeholder("+ add common category")
                        .width(Fill)
                        .padding([6, 9])
                        .text_size(12)
                        .style(pick_list_style(colors)),
                    ]
                    .spacing(6),
                ),
                labeled(
                    colors,
                    "Keywords",
                    text_input("search;terms;", &editor.form.keywords)
                        .on_input(move |value| {
                            Message::Editor(id, EditorMessage::Field(Field::Keywords, value))
                        })
                        .padding([7, 9])
                        .size(13)
                        .style(input_style(colors)),
                ),
            ]
            .spacing(14),
        ),
        section(
            colors,
            "Behavior",
            row![
                checkbox(editor.form.terminal)
                    .label("Launch in a terminal")
                    .on_toggle(move |value| Message::Editor(id, EditorMessage::Terminal(value)))
                    .style(checkbox_style(colors))
                    .text_size(13),
                checkbox(editor.form.no_display)
                    .label("Hide from menus")
                    .on_toggle(move |value| Message::Editor(id, EditorMessage::NoDisplay(value)))
                    .style(checkbox_style(colors))
                    .text_size(13),
                checkbox(editor.form.startup_notify)
                    .label("Startup notification")
                    .on_toggle(move |value| {
                        Message::Editor(id, EditorMessage::StartupNotify(value))
                    })
                    .style(checkbox_style(colors))
                    .text_size(13),
            ]
            .spacing(24),
        ),
        row![
            button(text("Save").size(12))
                .on_press(Message::Editor(id, EditorMessage::Save))
                .style(primary_button(colors))
                .padding([8, 16]),
            button(text("Pin to desktop").size(12))
                .on_press(Message::Editor(id, EditorMessage::PinToDesktop))
                .style(button_style(colors))
                .padding([8, 14]),
            button(text("Discard").size(12))
                .on_press(Message::Editor(id, EditorMessage::Discard))
                .style(button_style(colors))
                .padding([8, 14]),
            button(text("Delete").size(12))
                .on_press_maybe(
                    editor
                        .form
                        .path
                        .is_some()
                        .then_some(Message::Editor(id, EditorMessage::Delete)),
                )
                .style(danger_button(colors))
                .padding([8, 14]),
            row![].width(Fill),
            button(text("Open folder").size(12))
                .on_press(Message::Editor(id, EditorMessage::OpenFolder))
                .style(button_style(colors))
                .padding([8, 14]),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(24)
    .max_width(760);

    container(scrollable(container(form).padding([24, 28]).center_x(Fill)).height(Fill))
        .width(Fill)
        .height(Fill)
        .style(plain(colors, colors.bg, 0.0))
        .into()
}

fn preview(editor: &Editor, colors: Colors) -> Element<'_, Message> {
    container(
        column![
            row![
                text("Preview")
                    .size(11)
                    .font(BOLD)
                    .color(dim(colors.ink, 0.55)),
                row![].width(Fill),
                text(editor.form.file_name())
                    .size(11)
                    .color(dim(colors.ink, 0.55)),
            ]
            .align_y(Alignment::Center),
            rule::horizontal(HAIRLINE as u32).style(hairline(colors)),
            scrollable(
                container(text(editor.form.render()).size(12).font(MONO).width(Fill)).padding(14)
            )
            .height(Fill)
            .style(scroll_style(colors)),
        ]
        .padding(16)
        .width(340)
        .height(Fill)
        .spacing(12),
    )
    .width(340)
    .height(Fill)
    .style(plain(colors, colors.surface, 0.0))
    .into()
}

fn status_bar(
    colors: Colors,
    status: &str,
    is_error: bool,
    trailing: String,
) -> Element<'_, Message> {
    column![
        rule::horizontal(HAIRLINE as u32).style(hairline(colors)),
        container(
            row![
                text(status).size(12),
                row![].width(Fill),
                text(trailing).size(12),
            ]
            .align_y(Alignment::Center),
        )
        .padding([8, 16])
        .width(Fill)
        .style(plain(
            colors,
            if is_error {
                colors.magenta
            } else {
                colors.surface
            },
            0.0
        )),
    ]
    .spacing(0)
    .into()
}

fn section<'a>(
    colors: Colors,
    title: &'a str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        text(title).size(11).font(BOLD).color(dim(colors.ink, 0.55)),
        rule::horizontal(HAIRLINE as u32).style(hairline(colors)),
        content.into(),
    ]
    .spacing(10)
    .width(Fill)
    .into()
}

fn labeled<'a>(
    colors: Colors,
    label: &'a str,
    input: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let input: Element<'a, Message> = input.into();

    column![text(label).size(12).color(dim(colors.ink, 0.6)), input,]
        .spacing(5)
        .width(Fill)
        .into()
}
