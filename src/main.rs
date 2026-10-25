#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app_updates;
mod audit;
mod catalog;
mod costs;
mod engine;
mod hotkey;
mod insights;
mod instance;
mod lab;
mod patch;
mod range;
mod ranking;
mod recognition;
mod reference;
mod spending;
mod storage;
mod updates;

use engine::{Goal, Profile, Slot, Track};
use iced::{
    Border, Color, Element, Fill, Font, Subscription, Task, Theme,
    widget::{
        button, checkbox, column, container, pick_list, row, rule, scrollable, slider, space, text,
        text_input,
    },
    window,
};

const BG: Color = Color::from_rgb(0.035, 0.045, 0.055);
const PANEL: Color = Color::from_rgb(0.065, 0.080, 0.093);
const LINE: Color = Color::from_rgb(0.14, 0.17, 0.19);
const ACCENT: Color = Color::from_rgb(1., 0.43, 0.19);
const MUTED: Color = Color::from_rgb(0.58, 0.65, 0.68);
const GREEN: Color = Color::from_rgb(0.48, 0.85, 0.69);

fn main() -> iced::Result {
    if let Some(path) = std::env::args().skip_while(|a| a != "--ocr-image").nth(1) {
        match recognition::read(Some(path)) {
            Ok(text) => {
                println!("{text}");
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }
    if let Some(path) = std::env::args()
        .skip_while(|a| a != "--audit-output")
        .nth(1)
    {
        if let Err(error) = audit::export(&catalog::Catalog::bundled(), std::path::Path::new(&path))
        {
            eprintln!("Audit failed: {error}");
            std::process::exit(1);
        }
        return Ok(());
    }
    let _instance = match instance::acquire() {
        Ok(instance) => instance,
        Err(error) => {
            instance::notify(&error);
            return Ok(());
        }
    };
    iced::application(App::boot, App::update, App::view)
        .title(concat!("WARDOG Companion · ", env!("CARGO_PKG_VERSION")))
        .theme(|_: &App| {
            Theme::custom(
                "Wardog",
                iced::theme::Palette {
                    background: BG,
                    text: Color::from_rgb(0.93, 0.95, 0.96),
                    primary: ACCENT,
                    success: GREEN,
                    warning: ACCENT,
                    danger: Color::from_rgb(0.95, 0.4, 0.4),
                },
            )
        })
        .style(|_: &App, _: &Theme| iced::theme::Style {
            background_color: BG,
            text_color: Color::from_rgb(0.93, 0.95, 0.96),
        })
        .subscription(App::subscription)
        .window(window::Settings {
            icon: window::icon::from_rgba(
                include_bytes!("../assets/wardog.rgba").to_vec(),
                256,
                256,
            )
            .ok(),
            size: iced::Size::new(540., 790.),
            min_size: Some(iced::Size::new(500., 660.)),
            position: window::Position::Centered,
            level: window::Level::AlwaysOnTop,
            ..Default::default()
        })
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Home,
    Settings,
    Customize,
    Weapons,
    Compare,
    Data,
    Attachments,
    Checks,
    Range,
    Costs,
    Recognition,
    Spending,
    Patch,
    AppUpdates,
}
#[derive(Debug, Clone)]
enum Message {
    Goal(Goal),
    Open(Page),
    Done,
    Level(Track, u8),
    Budget(String),
    OwnedOnly(bool),
    Pick(Slot, lab::Choice),
    ApplySwap,
    CancelSwap,
    SwapDetails,
    UpgradeDetails,
    PreferredSight(lab::Choice),
    MinimumCapacity(Capacity),
    Keep(Slot),
    Owned(String, bool),
    Reset,
    Details,
    Compact,
    CompactWindow(Option<window::Id>),
    Hide,
    Visibility(Option<window::Id>),
    Top,
    TopWindow(Option<window::Id>),
    Hotkey(hotkey::Event),
    WeaponSearch(String),
    WeaponClass(String),
    SelectWeapon(String),
    RankClass(String),
    RankMode(ranking::Mode),
    RankHealth(u32),
    RankLocked(bool),
    RankDistance(u32),
    RankBuilds(bool),
    RankAudit,
    RankParts(String),
    ClassCheck(String, usize, bool),
    ReloadData,
    AttachmentSearch(String),
    AttachmentPage(usize),
    InspectAttachment(String),
    Reach(range::Reach),
    UseRange(String),
    Unlocked(String, bool),
    CheckUpdates,
    UpdateChecked(Result<Option<updates::Candidate>, String>),
    ApplyUpdate,
    RollbackUpdate,
    Capture(bool),
    Recognized(Result<String, String>),
    RecognitionWindow(Option<window::Id>),
    UseDetected(String),
    BackgroundFailed,
    ApplySpending(usize),
    CheckAppUpdate,
    AppUpdateChecked(Result<Option<app_updates::Release>, String>),
    DownloadAppUpdate,
    AppUpdateDownloaded(Result<std::path::PathBuf, String>),
    InstallAppUpdate,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Capacity(u32);
impl std::fmt::Display for Capacity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == 0 {
            f.write_str("No minimum")
        } else {
            write!(f, "{}+ rounds", self.0)
        }
    }
}
struct App {
    catalog: catalog::Catalog,
    data_status: String,
    weapon_search: String,
    weapon_class: String,
    rank_class: String,
    rank_mode: ranking::Mode,
    rank_health: u32,
    rank_locked: bool,
    rank_distance: u32,
    rank_builds: bool,
    rank_audit: bool,
    rank_parts: Option<String>,
    rank_report: ranking::Report,
    attachment_search: String,
    attachment_page: usize,
    attachment_selected: Option<String>,
    profile: Profile,
    reference: reference::Reference,
    estimate: Result<lab::Estimate, String>,
    page: Page,
    budget: String,
    details: bool,
    compact: bool,
    pending_swap: Option<(Slot, lab::Choice)>,
    swap_details: bool,
    upgrade_details: bool,
    upgrade: Option<insights::Upgrade>,
    notice: Option<String>,
    storage_blocked: bool,
    hidden: bool,
    pinned: bool,
    shortcut_ready: bool,
    shortcut_error: Option<String>,
    reach: range::Reach,
    range_report: ranking::Report,
    update_busy: bool,
    update_candidate: Option<updates::Candidate>,
    update_status: String,
    ocr_busy: bool,
    ocr_status: String,
    ocr_matches: Vec<recognition::Match>,
    ocr_shortcut: bool,
    spending: Result<spending::Report, String>,
    patch_baseline: Option<catalog::Catalog>,
    patch_report: Option<patch::Report>,
    patch_error: Option<String>,
    app_update_busy: bool,
    app_release: Option<app_updates::Release>,
    app_installer: Option<std::path::PathBuf>,
    app_update_status: String,
}
impl App {
    fn boot() -> (Self, Task<Message>) {
        let app = Self::new();
        let task = std::env::args()
            .skip_while(|a| a != "--recognize-image")
            .nth(1)
            .map(|path| background(move || recognition::read(Some(path)), Message::Recognized))
            .unwrap_or_else(Task::none);
        (app, task)
    }
    fn new() -> Self {
        let (mut profile, mut notice, storage_blocked) = match storage::load() {
            Ok(p) => (p, None, false),
            Err(e) => (Profile::default(), Some(e), true),
        };
        let known_levels =
            profile.levels_confirmed || profile.levels.iter().any(|level| *level > 0);
        let (catalog, data_status) = if catalog::Catalog::path().exists() {
            match catalog::Catalog::external() {
                Ok(data) => (data, "External dataset loaded".into()),
                Err(error) => {
                    if notice.is_none() {
                        notice = Some(format!("Dataset rejected; using bundled data. {error}"));
                    }
                    (
                        catalog::Catalog::bundled(),
                        "Bundled fallback · external data rejected".into(),
                    )
                }
            }
        } else {
            (catalog::Catalog::bundled(), "Bundled dataset".into())
        };
        let reference = catalog
            .weapon(&profile.weapon_id)
            .or_else(|| catalog.weapon("Id.Item.AK74M"))
            .unwrap_or(&catalog.weapons[0])
            .clone();
        if !reference.weapon.id.eq_ignore_ascii_case(&profile.weapon_id) {
            profile.lab_parts.clear();
            profile.lab_pins.clear();
            profile.preferred_sight = None;
            profile.minimum_capacity = 0;
        }
        profile.weapon_id = reference.weapon.id.clone();
        if known_levels
            && profile.lab_parts.is_empty()
            && let Ok(parts) = lab::suggest(&reference, &profile)
        {
            profile.lab_parts = parts;
        }
        let estimate = lab::evaluate(&reference, &profile, &profile.lab_parts);
        let spending = spending::analyze(&reference, &profile);
        let previous_path = updates::backup_path(&catalog::Catalog::path());
        let (patch_baseline, patch_error) = if previous_path.exists() {
            match std::fs::metadata(&previous_path)
                .map_err(|e| e.to_string())
                .and_then(|m| {
                    if m.len() > 15_000_000 {
                        Err("Previous dataset exceeds size limit.".into())
                    } else {
                        std::fs::read_to_string(&previous_path).map_err(|e| e.to_string())
                    }
                })
                .and_then(|s| catalog::Catalog::parse(&s))
            {
                Ok(data) => (Some(data), None),
                Err(e) => (None, Some(format!("Previous data unavailable. {e}"))),
            }
        } else {
            (None, None)
        };
        let patch_report = patch_baseline
            .as_ref()
            .map(|old| patch::compare(old, &catalog, &profile));
        let upgrade = insights::next_upgrade(&reference, &profile);
        let rank_report = ranking::compare(
            &catalog,
            &reference.weapon.class_name,
            ranking::Mode::Fastest,
            ranking::Scenario {
                hp: 100,
                distance: 0,
                builds: false,
            },
            &profile,
            true,
        );
        let range_report = range::recommend(
            &catalog,
            &profile,
            &reference.weapon.class_name,
            range::Reach::Medium,
        );
        Self {
            rank_class: reference.weapon.class_name.clone(),
            catalog,
            data_status,
            weapon_search: String::new(),
            weapon_class: "All classes".into(),
            rank_mode: ranking::Mode::Fastest,
            rank_health: 100,
            rank_locked: true,
            rank_distance: 0,
            rank_builds: false,
            rank_audit: false,
            rank_parts: None,
            rank_report,
            attachment_search: String::new(),
            attachment_page: 0,
            attachment_selected: None,
            budget: profile.budget.to_string(),
            profile,
            reference,
            estimate,
            page: if known_levels {
                Page::Home
            } else {
                Page::Settings
            },
            details: false,
            compact: false,
            pending_swap: None,
            swap_details: false,
            upgrade_details: false,
            upgrade,
            notice,
            storage_blocked,
            hidden: false,
            pinned: true,
            shortcut_ready: false,
            shortcut_error: None,
            reach: range::Reach::Medium,
            range_report,
            update_busy: false,
            update_candidate: None,
            update_status: "Checks run only when you ask. Your profile stays on this computer."
                .into(),
            ocr_busy: false,
            ocr_status: "Open the arsenal and keep the weapon name visible.".into(),
            ocr_matches: vec![],
            ocr_shortcut: false,
            spending,
            patch_baseline,
            patch_report,
            patch_error,
            app_update_busy: false,
            app_release: None,
            app_installer: None,
            app_update_status: "Check when you want. Nothing installs automatically.".into(),
        }
    }
    fn subscription(&self) -> Subscription<Message> {
        hotkey::subscription().map(Message::Hotkey)
    }
    fn refresh(&mut self) {
        self.spending = spending::analyze(&self.reference, &self.profile);
        self.refresh_patch();
        self.estimate = lab::evaluate(&self.reference, &self.profile, &self.profile.lab_parts);
        self.upgrade = insights::next_upgrade(&self.reference, &self.profile);
        self.refresh_rank();
    }
    fn refresh_patch(&mut self) {
        self.patch_report = if let Some(candidate) = &self.update_candidate {
            catalog::Catalog::parse(&candidate.json)
                .ok()
                .map(|next| patch::compare(&self.catalog, &next, &self.profile))
        } else {
            self.patch_baseline
                .as_ref()
                .map(|old| patch::compare(old, &self.catalog, &self.profile))
        };
    }
    fn refresh_rank(&mut self) {
        self.range_report =
            range::recommend(&self.catalog, &self.profile, &self.rank_class, self.reach);
        self.rank_report = ranking::compare(
            &self.catalog,
            &self.rank_class,
            self.rank_mode,
            ranking::Scenario {
                hp: self.rank_health,
                distance: self.rank_distance,
                builds: self.rank_builds,
            },
            &self.profile,
            self.rank_locked,
        );
    }
    fn save(&mut self) {
        if self.storage_blocked {
            self.notice=Some("Your existing profile could not be read. It has been preserved; repair or back it up before saving.".into());
            return;
        }
        self.notice = storage::save(&self.profile)
            .err()
            .map(|error| format!("Could not save your changes: {error}"));
    }
    fn recommend(&mut self) {
        match lab::suggest(&self.reference, &self.profile) {
            Ok(parts) => {
                self.profile.lab_parts = parts;
                self.refresh();
                self.save();
            }
            Err(error) => {
                self.refresh();
                self.save();
                self.notice = Some(error);
            }
        }
    }
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ApplySpending(index) => {
                match spending::analyze(&self.reference, &self.profile) {
                    Ok(report) => {
                        if let Some(option) = report.options.get(index) {
                            self.profile.lab_parts = option.build.parts.clone();
                            self.refresh();
                            self.save();
                            self.page = Page::Home;
                            return iced::widget::operation::snap_to(
                                "content",
                                iced::widget::operation::RelativeOffset::START,
                            );
                        }
                    }
                    Err(e) => self.notice = Some(e),
                }
            }
            Message::BackgroundFailed => {
                self.ocr_busy = false;
                self.update_busy = false;
                self.notice = Some(
                    "Background operation stopped. Your current setup and data were preserved."
                        .into(),
                );
            }
            Message::Reach(reach) => {
                self.reach = reach;
                self.refresh_rank();
            }
            Message::UseRange(id) => {
                self.profile.goal = self.reach.goal();
                return self.update(Message::SelectWeapon(id));
            }
            Message::Unlocked(id, value) => {
                if value {
                    self.profile.unlocked.insert(id.to_lowercase());
                } else {
                    self.profile
                        .unlocked
                        .retain(|i| !i.eq_ignore_ascii_case(&id));
                }
                self.save();
            }
            Message::CheckUpdates => {
                if self.update_busy {
                    return Task::none();
                }
                self.update_busy = true;
                self.update_candidate = None;
                self.refresh_patch();
                self.update_status = "Downloading and validating source data…".into();
                let json = updates::active_json();
                return background(move || updates::check(json?), Message::UpdateChecked);
            }
            Message::CheckAppUpdate => {
                if self.app_update_busy {
                    return Task::none();
                }
                self.app_update_busy = true;
                self.app_release = None;
                self.app_installer = None;
                self.app_update_status = "Checking the official GitHub release…".into();
                return background(app_updates::check, Message::AppUpdateChecked);
            }
            Message::AppUpdateChecked(result) => {
                self.app_update_busy = false;
                match result {
                    Ok(Some(release)) => {
                        self.app_update_status =
                            format!("Version {} is available.", release.version);
                        self.app_release = Some(release);
                    }
                    Ok(None) => self.app_update_status = "You're up to date.".into(),
                    Err(error) => self.app_update_status = error,
                }
            }
            Message::DownloadAppUpdate => {
                if self.app_update_busy {
                    return Task::none();
                }
                if let Some(release) = self.app_release.clone() {
                    self.app_update_busy = true;
                    self.app_update_status = "Downloading and verifying the installer…".into();
                    return background(
                        move || app_updates::download(release),
                        Message::AppUpdateDownloaded,
                    );
                }
            }
            Message::AppUpdateDownloaded(result) => {
                self.app_update_busy = false;
                match result {
                    Ok(path) => {
                        self.app_installer = Some(path);
                        self.app_update_status = "Verified and ready. Setup will guide you through the update. Your profile stays saved.".into();
                    }
                    Err(error) => self.app_update_status = error,
                }
            }
            Message::InstallAppUpdate => {
                if let (Some(path), Some(release)) = (&self.app_installer, &self.app_release) {
                    match app_updates::install(path, release) {
                        Ok(()) => return iced::exit(),
                        Err(error) => self.app_update_status = error,
                    }
                }
            }
            Message::UpdateChecked(result) => {
                self.update_busy = false;
                match result {
                    Ok(Some(candidate)) => {
                        self.update_status = candidate.summary.clone();
                        self.update_candidate = Some(candidate);
                    }
                    Ok(None) => self.update_status = "Your data files are up to date.".into(),
                    Err(e) => self.update_status = format!("Active data preserved. {e}"),
                }
                self.refresh_patch();
            }
            Message::ApplyUpdate => {
                if let Some(candidate) = &self.update_candidate {
                    let result = (|| {
                        let next = catalog::Catalog::parse(&candidate.json)?;
                        if next.weapon(&self.profile.weapon_id).is_none() {
                            return Err(
                                "Selected weapon is missing; choose another weapon first.".into()
                            );
                        }
                        let current = updates::active_json()?;
                        updates::activate(&catalog::Catalog::path(), &candidate.json, &current)
                    })();
                    match result {
                        Ok(()) => {
                            self.update_candidate = None;
                            self.update_status =
                                "Update installed. Previous dataset is available below.".into();
                            return self.update(Message::ReloadData);
                        }
                        Err(e) => self.update_status = format!("Update not installed. {e}"),
                    }
                }
            }
            Message::RollbackUpdate => {
                let result = (|| {
                    let path = catalog::Catalog::path();
                    let old = std::fs::read_to_string(updates::backup_path(&path))
                        .map_err(|e| e.to_string())?;
                    let data = catalog::Catalog::parse(&old)?;
                    if data.weapon(&self.profile.weapon_id).is_none() {
                        return Err("Previous dataset lacks your selected weapon.".into());
                    }
                    updates::rollback(&path)
                })();
                match result {
                    Ok(()) => {
                        self.update_candidate = None;
                        self.update_status = "Previous dataset restored.".into();
                        return self.update(Message::ReloadData);
                    }
                    Err(e) => self.update_status = format!("Active data preserved. {e}"),
                }
            }
            Message::Capture(delayed) => {
                if self.ocr_busy {
                    return Task::none();
                }
                self.ocr_busy = true;
                self.ocr_matches.clear();
                self.ocr_status = "Reading the arsenal…".into();
                return background(
                    move || {
                        if delayed {
                            std::thread::sleep(std::time::Duration::from_secs(3));
                        }
                        recognition::read(None)
                    },
                    Message::Recognized,
                );
            }
            Message::Recognized(result) => {
                self.ocr_busy = false;
                match result {
                    Ok(text) => {
                        self.ocr_matches = recognition::matches(&self.catalog, &text);
                        self.ocr_status = if self.ocr_matches.is_empty() {
                            "No weapon name matched. Keep the name visible or select it manually."
                                .into()
                        } else {
                            "Confirm the weapon. Several inventory names may be visible; nothing has been changed yet.".into()
                        };
                    }
                    Err(e) => self.ocr_status = e,
                }
                self.page = Page::Recognition;
                self.hidden = false;
                self.compact = false;
                return window::oldest().map(Message::RecognitionWindow);
            }
            Message::RecognitionWindow(Some(id)) => {
                return window::set_mode(id, window::Mode::Windowed)
                    .chain(window::minimize(id, false))
                    .chain(window::set_min_size(id, Some(iced::Size::new(500., 660.))))
                    .chain(window::resize(id, iced::Size::new(540., 790.)));
            }
            Message::UseDetected(id) => {
                self.ocr_matches.clear();
                if id.eq_ignore_ascii_case(&self.profile.weapon_id) {
                    self.page = Page::Home;
                } else {
                    return self.update(Message::SelectWeapon(id));
                }
            }
            Message::WeaponSearch(value) => self.weapon_search = value,
            Message::WeaponClass(value) => self.weapon_class = value,
            Message::RankClass(value) => {
                self.rank_class = value;
                self.refresh_rank();
            }
            Message::RankMode(value) => {
                self.rank_mode = value;
                self.refresh_rank();
            }
            Message::RankHealth(value) => {
                self.rank_health = value;
                self.refresh_rank();
            }
            Message::RankLocked(value) => {
                self.rank_locked = value;
                self.refresh_rank();
            }
            Message::RankDistance(value) => {
                self.rank_distance = value;
                self.refresh_rank();
            }
            Message::RankBuilds(value) => {
                self.rank_builds = value;
                self.refresh_rank();
            }
            Message::RankAudit => self.rank_audit = !self.rank_audit,
            Message::RankParts(id) => {
                self.rank_parts = if self.rank_parts.as_ref() == Some(&id) {
                    None
                } else {
                    Some(id)
                }
            }
            Message::ClassCheck(key, index, value) => {
                let check = self.profile.class_checks.entry(key).or_default();
                match index {
                    0 => check.fits = value,
                    1 => check.requirements = value,
                    _ => check.panel = value,
                }
                self.save();
            }
            Message::AttachmentSearch(value) => {
                self.attachment_search = value;
                self.attachment_page = 0;
                self.attachment_selected = None;
            }
            Message::AttachmentPage(value) => {
                self.attachment_page = value;
                self.attachment_selected = None;
            }
            Message::InspectAttachment(value) => self.attachment_selected = Some(value),
            Message::SelectWeapon(id) => {
                if let Some(reference) = self.catalog.weapon(&id) {
                    self.reference = reference.clone();
                    self.profile.weapon_id = id;
                    self.profile.lab_parts.clear();
                    self.profile.lab_pins.clear();
                    self.profile.preferred_sight = None;
                    self.profile.minimum_capacity = 0;
                    self.pending_swap = None;
                    self.details = false;
                    self.rank_class = self.reference.weapon.class_name.clone();
                    self.recommend();
                    self.page = Page::Home;
                    return iced::widget::operation::snap_to(
                        "content",
                        iced::widget::operation::RelativeOffset::START,
                    );
                }
            }
            Message::ReloadData => match catalog::Catalog::external() {
                Ok(data) => {
                    if let Some(reference) = data.weapon(&self.profile.weapon_id) {
                        self.reference = reference.clone();
                        if data != self.catalog {
                            self.patch_baseline = Some(std::mem::replace(&mut self.catalog, data));
                            self.patch_error = None;
                        }
                        self.data_status = "External dataset validated and loaded".into();
                        self.rank_class = self.reference.weapon.class_name.clone();
                        if !self.catalog.classes().contains(&self.weapon_class) {
                            self.weapon_class = "All classes".into();
                        }
                        self.refresh();
                        self.notice = None;
                    } else {
                        self.notice = Some("New data does not contain your selected weapon. Active data was preserved; choose another weapon before reloading.".into());
                    }
                }
                Err(error) => self.notice = Some(format!("Active data preserved. {error}")),
            },
            Message::Goal(goal) => {
                self.profile.goal = goal;
                self.recommend();
            }
            Message::Open(page) => {
                self.page = page;
                self.details = false;
                if self.compact {
                    self.compact = false;
                    return window::oldest().map(Message::CompactWindow);
                }
                return iced::widget::operation::snap_to(
                    "content",
                    iced::widget::operation::RelativeOffset::START,
                );
            }
            Message::Done => {
                if self.page == Page::Settings {
                    self.profile.levels_confirmed = true;
                    self.recommend();
                } else {
                    self.refresh();
                    self.save();
                }
                self.page = Page::Home;
                return iced::widget::operation::snap_to(
                    "content",
                    iced::widget::operation::RelativeOffset::START,
                );
            }
            Message::Level(track, level) => self.profile.levels[track.index()] = level,
            Message::Budget(value) => {
                if value.len() <= 6 && value.chars().all(|c| c.is_ascii_digit()) {
                    self.profile.budget = value.parse::<u32>().unwrap_or(0).min(100_000);
                    self.budget = if value.is_empty() {
                        String::new()
                    } else {
                        self.profile.budget.to_string()
                    };
                }
            }
            Message::OwnedOnly(value) => self.profile.owned_only = value,
            Message::Pick(slot, choice) => {
                self.pending_swap = Some((slot, choice));
                self.swap_details = false;
                return iced::widget::operation::snap_to(
                    "content",
                    iced::widget::operation::RelativeOffset::START,
                );
            }
            Message::ApplySwap => {
                if let Some((slot, choice)) = &self.pending_swap {
                    match insights::compare(&self.reference, &self.profile, *slot, choice) {
                        Ok(_) => {
                            self.profile.lab_parts =
                                insights::swapped(&self.profile, *slot, choice);
                            if *slot == Slot::Optic && self.profile.preferred_sight.is_some() {
                                self.profile.preferred_sight = choice.id.clone();
                            }
                        }
                        Err(error) => {
                            self.notice = Some(error);
                            return Task::none();
                        }
                    }
                }
                self.pending_swap = None;
                self.refresh();
                self.save();
            }
            Message::CancelSwap => self.pending_swap = None,
            Message::SwapDetails => self.swap_details = !self.swap_details,
            Message::UpgradeDetails => self.upgrade_details = !self.upgrade_details,
            Message::PreferredSight(choice) => {
                self.profile.preferred_sight = choice.id;
                // An explicit new preference replaces the previous sight pin.
                self.profile.lab_pins.remove(&Slot::Optic);
                self.recommend();
            }
            Message::MinimumCapacity(capacity) => {
                self.profile.minimum_capacity = capacity.0;
                self.recommend();
            }
            Message::Keep(slot) => {
                if !self.profile.lab_pins.remove(&slot) {
                    self.profile.lab_pins.insert(slot);
                }
                self.refresh();
                self.save();
            }
            Message::Owned(id, value) => {
                if value {
                    self.profile.owned.insert(id);
                } else {
                    self.profile
                        .owned
                        .retain(|owned| !owned.eq_ignore_ascii_case(&id));
                }
                self.refresh();
                self.save();
            }
            Message::Reset => {
                self.profile.lab_pins.clear();
                self.recommend();
            }
            Message::Details => self.details = !self.details,
            Message::Compact => {
                self.compact = !self.compact;
                self.details = false;
                return window::oldest().map(Message::CompactWindow);
            }
            Message::CompactWindow(Some(id)) => {
                let (size, minimum) = if self.compact {
                    (iced::Size::new(360., 560.), iced::Size::new(340., 500.))
                } else {
                    (iced::Size::new(540., 790.), iced::Size::new(500., 660.))
                };
                return window::set_min_size(id, Some(minimum))
                    .chain(window::resize(id, size))
                    .chain(iced::widget::operation::snap_to(
                        "content",
                        iced::widget::operation::RelativeOffset::START,
                    ));
            }
            Message::Hide => return window::oldest().map(Message::Visibility),
            Message::Visibility(Some(id)) => {
                if !self.shortcut_ready {
                    return window::minimize(id, true);
                }
                self.hidden = !self.hidden;
                return window::set_mode(
                    id,
                    if self.hidden {
                        window::Mode::Hidden
                    } else {
                        window::Mode::Windowed
                    },
                );
            }
            Message::Top => {
                self.pinned = !self.pinned;
                return window::oldest().map(Message::TopWindow);
            }
            Message::TopWindow(Some(id)) => {
                return window::set_level(
                    id,
                    if self.pinned {
                        window::Level::AlwaysOnTop
                    } else {
                        window::Level::Normal
                    },
                );
            }
            Message::Hotkey(hotkey::Event::Ready) => self.shortcut_ready = true,
            Message::Hotkey(hotkey::Event::CaptureReady) => self.ocr_shortcut = true,
            Message::Hotkey(hotkey::Event::CaptureFailed(e)) => {
                self.ocr_shortcut = false;
                self.ocr_status =
                    format!("Recognition shortcut unavailable: {e}. Use the timed button.");
            }
            Message::Hotkey(hotkey::Event::Capture) => return self.update(Message::Capture(false)),
            Message::Hotkey(hotkey::Event::Failed(error)) => {
                self.shortcut_ready = false;
                self.shortcut_error = Some(error);
            }
            Message::Hotkey(hotkey::Event::Toggle) => {
                return window::oldest().map(Message::Visibility);
            }
            _ => {}
        }
        Task::none()
    }
    fn view(&self) -> Element<'_, Message> {
        if self.compact {
            return self.play_view();
        }
        let brand = row![
            container(text("W").size(23).font(Font::MONOSPACE).color(ACCENT))
                .padding([4, 8])
                .style(panel),
            column![
                text("WARDOG").size(18).font(Font::MONOSPACE),
                text(concat!("COMPANION · ", env!("CARGO_PKG_VERSION")))
                    .size(9)
                    .color(MUTED)
            ]
            .spacing(1)
        ]
        .spacing(10)
        .align_y(iced::Center);
        let controls: Element<'_, Message> = if self.page == Page::Home {
            row![
                button(if self.pinned {
                    "Pinned window"
                } else {
                    "Pin window"
                })
                .on_press(Message::Top)
                .style(button::text),
                button("Settings")
                    .on_press(Message::Open(Page::Settings))
                    .style(button::secondary)
            ]
            .spacing(6)
            .into()
        } else {
            button("Done")
                .on_press_maybe(self.pending_swap.is_none().then_some(Message::Done))
                .padding([8, 18])
                .into()
        };
        let header = row![brand, space::horizontal(), controls].align_y(iced::Center);
        let body = match self.page {
            Page::Home => self.home(),
            Page::Settings => self.settings(),
            Page::Customize => self.customize(),
            Page::Weapons => self.weapons_view(),
            Page::Compare => self.compare_view(),
            Page::Data => self.data_view(),
            Page::AppUpdates => self.app_updates_view(),
            Page::Attachments => self.attachments_view(),
            Page::Checks => self.checks_view(),
            Page::Range => self.range_view(),
            Page::Costs => self.cost_view(),
            Page::Recognition => self.recognition_view(),
            Page::Spending => self.spending_view(),
            Page::Patch => self.patch_view(),
        };
        let body = container(body).padding(iced::Padding {
            right: 14.0,
            ..Default::default()
        });
        let mut root = column![header, scrollable(body).id("content").height(Fill)].spacing(20);
        if let Some(notice) = &self.notice {
            root = root.push(text(notice.as_str()).size(12).width(Fill).color(ACCENT));
        }
        if self.page == Page::Home && self.patch_report.as_ref().is_some_and(|r| r.relevant) {
            root = root.push(
                button(if self.update_candidate.is_some() {
                    "Update may affect this setup"
                } else {
                    "Patch affects this setup"
                })
                .on_press(Message::Open(Page::Patch))
                .style(button::text),
            );
        }
        let footer = if self.page == Page::Home {
            column![
                rule::horizontal(1),
                button(
                    text(if self.shortcut_ready {
                        "Hide & play"
                    } else {
                        "Minimize"
                    })
                    .size(16)
                )
                .on_press(Message::Hide)
                .padding([12, 20])
                .width(Fill),
                text(if self.shortcut_ready {
                    "CTRL + SHIFT + SPACE to bring it back"
                } else {
                    "Shortcut unavailable · reopen from the taskbar"
                })
                .size(10)
                .color(MUTED)
            ]
            .spacing(10)
        } else {
            column![
                text("Settings stay on this computer.")
                    .size(10)
                    .color(MUTED)
            ]
        };
        root = root.push(footer);
        container(root).padding(22).width(Fill).height(Fill).into()
    }
    fn home(&self) -> Element<'_, Message> {
        let weapon = row![
            column![
                button(text(self.reference.weapon.name.as_str()).size(32))
                    .on_press(Message::Open(Page::Weapons))
                    .style(button::text),
                text(if lab::slots(&self.reference).len() > 4 {
                    "More components in Customize."
                } else {
                    "Your next setup."
                })
                .size(13)
                .color(MUTED)
            ]
            .spacing(2),
            space::horizontal(),
            text("CHANGE WEAPON").size(9).color(MUTED)
        ]
        .align_y(iced::Center);
        let mut goals = row![].spacing(7);
        for goal in Goal::ALL {
            let active = self.profile.goal == goal;
            goals = goals.push(
                button(text(goal.label()).size(14))
                    .on_press(Message::Goal(goal))
                    .padding([10, 8])
                    .width(Fill)
                    .style(move |theme, status| {
                        if active {
                            button::primary(theme, status)
                        } else {
                            button::secondary(theme, status)
                        }
                    }),
            );
        }
        let mut content = if self.compact {
            column![goals].spacing(8).width(Fill)
        } else {
            column![weapon, goals].spacing(10).width(Fill)
        };
        if ranking::eligible(&self.reference.weapon, &self.profile) == Some(false) {
            content = content.push(
                text(format!(
                    "Weapon preview · {}",
                    ranking::availability(&self.reference.weapon, &self.profile)
                ))
                .size(11)
                .color(ACCENT)
                .width(Fill),
            );
        }
        if let Err(error) = &self.estimate {
            content = content.push(card(
                column![
                    text("Let's adjust this setup.").size(18),
                    text(error.as_str()).size(12).color(ACCENT),
                    button("Review settings")
                        .on_press(Message::Open(Page::Settings))
                        .style(button::secondary)
                ]
                .spacing(9),
            ));
        }
        for slot in lab::slots(&self.reference)
            .into_iter()
            .filter(|s| lab::SLOTS.contains(s))
        {
            let selected = self
                .profile
                .lab_parts
                .get(&slot)
                .and_then(|id| lab::item(&self.reference, id));
            let name = selected.map(|item| item.name.as_str()).unwrap_or(
                if self.profile.lab_parts.contains_key(&slot) {
                    "Unavailable saved item"
                } else {
                    "Unchanged"
                },
            );
            let keep = self.profile.lab_pins.contains(&slot);
            content = content.push(
                card(
                    row![
                        column![
                            text(slot.label().to_uppercase()).size(9).color(MUTED),
                            text(name)
                                .size(if self.compact { 14 } else { 16 })
                                .width(Fill)
                        ]
                        .spacing(5)
                        .width(Fill),
                        button(if keep { "Kept" } else { "Keep" })
                            .on_press(Message::Keep(slot))
                            .style(if keep { button::primary } else { button::text })
                    ]
                    .spacing(8)
                    .align_y(iced::Center),
                )
                .padding(if self.compact { 7 } else { 10 }),
            );
        }
        if let Ok(estimate) = &self.estimate {
            if !estimate.selected.is_empty() {
                let indices = match self.profile.goal {
                    Goal::Control => [0, 1],
                    Goal::Agility => [2, 4],
                    Goal::Balanced => [1, 2],
                };
                let metric = |index: usize| {
                    let factor = estimate.factors[index];
                    let delta = (factor - 1.) * 100.;
                    let better = if index == 4 { delta > 0. } else { delta < 0. };
                    container(
                        column![
                            text(lab::ATTRIBUTES[index].1).size(11).color(MUTED),
                            text(format!("{delta:+.0}%")).size(25).color(if better {
                                GREEN
                            } else if delta.abs() < 0.001 {
                                MUTED
                            } else {
                                ACCENT
                            })
                        ]
                        .spacing(3),
                    )
                    .width(Fill)
                };
                content = content.push(row![metric(indices[0]), metric(indices[1])].spacing(10));
            } else {
                content = content.push(
                    text("No priced upgrades found at your levels and budget.")
                        .size(12)
                        .color(MUTED),
                );
            }
            content = content.push(
                row![
                    text("Selected attachments").size(12).color(MUTED),
                    space::horizontal(),
                    text(if estimate.unknown_prices.is_empty() {
                        money(estimate.cost)
                    } else {
                        "Cost incomplete".into()
                    })
                    .size(20)
                ]
                .align_y(iced::Center),
            );
            if !estimate.unknown_prices.is_empty() || estimate.cost > self.profile.budget as f64 {
                content = content.push(
                    text(estimate.budget_status(self.profile.budget))
                        .size(11)
                        .color(ACCENT)
                        .width(Fill),
                );
            }
        }
        if let Some(warning) =
            lab::preference_warning(&self.reference, &self.profile, &self.profile.lab_parts)
        {
            content = content.push(text(warning).size(11).color(ACCENT).width(Fill));
        }
        if !self.compact {
            content = content.push(
                row![
                    button("Play view")
                        .on_press(Message::Compact)
                        .style(button::primary),
                    button("Customize")
                        .on_press(Message::Open(Page::Customize))
                        .style(button::secondary),
                    button("Find my weapon")
                        .on_press(Message::Open(Page::Range))
                        .style(button::secondary),
                    space::horizontal(),
                    button(if self.details {
                        "Less detail"
                    } else {
                        "Why this setup?"
                    })
                    .on_press(Message::Details)
                    .style(button::text)
                ]
                .spacing(6)
                .align_y(iced::Center),
            );
        }
        content = content.push(
            text(if self.compact {
                "Partial attachment estimate"
            } else {
                "Estimated attachment effects · not a full loadout"
            })
            .size(10)
            .color(MUTED),
        );
        if !self.compact {
            content = content.push(
                row![
                    button("Cost to complete")
                        .on_press(Message::Open(Page::Costs))
                        .style(button::text),
                    button("Read arsenal")
                        .on_press(Message::Open(Page::Recognition))
                        .style(button::text),
                    button("Spend smarter")
                        .on_press(Message::Open(Page::Spending))
                        .style(button::text)
                ]
                .spacing(12),
            );
            if let Some(upgrade) = &self.upgrade {
                content = content.push(
                    button(
                        text(format!(
                            "Next unlock · {} more {} levels → {}",
                            upgrade
                                .level
                                .saturating_sub(self.profile.levels[upgrade.track.index()]),
                            upgrade.track.label(),
                            upgrade.name
                        ))
                        .size(11),
                    )
                    .on_press(Message::Open(Page::Customize))
                    .style(button::text),
                );
            }
        }
        if self.details {
            content = content.push(self.explanation());
        }
        content.into()
    }
    fn play_view(&self) -> Element<'_, Message> {
        let header = row![
            text(self.reference.weapon.name.as_str()).size(23),
            space::horizontal(),
            button(if self.pinned { "Pinned" } else { "Pin" })
                .on_press(Message::Top)
                .style(button::text),
            button("Edit")
                .on_press(Message::Compact)
                .style(button::secondary)
        ]
        .spacing(5)
        .align_y(iced::Center);
        let body = container(self.home()).padding(iced::Padding {
            right: 14.,
            ..Default::default()
        });
        let mut content = column![header, scrollable(body).id("content").height(Fill)].spacing(12);
        if let Some(notice) = &self.notice {
            content = content.push(text(notice.as_str()).size(11).color(ACCENT).width(Fill));
        }
        content = content.push(
            column![
                rule::horizontal(1),
                button(if self.shortcut_ready {
                    "Hide & play"
                } else {
                    "Minimize"
                })
                .on_press(Message::Hide)
                .padding([8, 12])
                .width(Fill),
                text(if self.shortcut_ready {
                    "CTRL + SHIFT + SPACE to bring it back"
                } else {
                    "Reopen from the taskbar"
                })
                .size(9)
                .color(MUTED)
            ]
            .spacing(8),
        );
        container(content)
            .padding(16)
            .width(Fill)
            .height(Fill)
            .into()
    }
    fn weapons_view(&self) -> Element<'_, Message> {
        let mut classes = vec!["All classes".to_string()];
        classes.extend(self.catalog.classes());
        let mut body = column![text("Choose your weapon.").size(28),
            text_input("Search all weapons", &self.weapon_search).on_input(Message::WeaponSearch).padding(10),
            pick_list(classes, Some(self.weapon_class.clone()), Message::WeaponClass).width(Fill),
            text("One active setup. Switching weapons clears selected parts, Keep pins and sight/capacity choices. Levels, ownership, goal and budget stay shared.").size(11).color(MUTED).width(Fill)
        ].spacing(12).width(Fill);
        let needle = self.weapon_search.to_lowercase();
        for reference in &self.catalog.weapons {
            let w = &reference.weapon;
            if (self.weapon_class != "All classes" && w.class_name != self.weapon_class)
                || !format!("{} {}", w.name, w.id)
                    .to_lowercase()
                    .contains(&needle)
            {
                continue;
            }
            body = body.push(
                button(
                    column![
                        text(w.name.as_str()).size(18),
                        text(format!(
                            "{} · {}",
                            w.class_name,
                            ranking::availability(w, &self.profile)
                        ))
                        .size(11)
                        .color(MUTED)
                    ]
                    .spacing(4),
                )
                .on_press(Message::SelectWeapon(w.id.clone()))
                .width(Fill)
                .padding(12)
                .style(list_button),
            );
        }
        body.into()
    }
    fn compare_view(&self) -> Element<'_, Message> {
        let classes = self.catalog.classes();
        let rows = &self.rank_report.entries;
        let mut body = column![text("Find your class pick.").size(27),
            pick_list(classes, Some(self.rank_class.clone()), Message::RankClass).width(Fill),
            pick_list(ranking::Mode::ALL, Some(self.rank_mode), Message::RankMode).width(Fill),
            row![text("Test HP").size(12), pick_list([100,150,200], Some(self.rank_health), Message::RankHealth),
                space::horizontal(), text("Distance (m)").size(12), pick_list([0,50,100,200,300,500],Some(self.rank_distance),Message::RankDistance)].spacing(8).align_y(iced::Center),
            checkbox(self.rank_builds).label("Compare available attachment builds").on_toggle(Message::RankBuilds),
            checkbox(self.rank_locked && !self.rank_builds).label("Include locked weapons (base mode)")
                .on_toggle_maybe((!self.rank_builds).then_some(Message::RankLocked)),
            text("Ideal raw hits · no armor/body multipliers · perfect hits. First hit at 0 s. Not a measured in-game kill time.").size(11).color(MUTED).width(Fill),
            row![button(if self.rank_audit { "Less detail" } else { "Method & audit" }).on_press(Message::RankAudit).style(button::text),
                button("Class checks").on_press(Message::Open(Page::Checks)).style(button::text)].spacing(10)
        ].spacing(12).width(Fill);
        if self.rank_builds {
            body=body.push(text(format!("{} build target · {} parts budget · your levels/ownership. Current Keep pins and sight/capacity choices are excluded. Unlock fees and weapon price are separate.",self.profile.goal.label(),money(self.profile.budget as f64))).size(11).color(MUTED).width(Fill));
        }
        if self.rank_audit {
            let mut detail=column![text("Damage follows published curves. Linear/constant segments and exact keys are supported; cubic interiors without tangents are excluded. Reloads, trigger/burst gaps, spin-up and flight time are excluded. Modeled attachments alter handling/recoil, not raw damage or RPM.").size(11).width(Fill),
                text("Recoil is mean magnitude of 10 source vectors; builds scale horizontal and vertical components separately. Balanced averages within-class TTK/recoil rank positions. Equal scores share a rank; names break display ties only.").size(11).width(Fill)].spacing(8);
            for (name, reason) in &self.rank_report.excluded {
                detail = detail.push(
                    text(format!("{name}: {reason}"))
                        .size(11)
                        .color(ACCENT)
                        .width(Fill),
                );
            }
            body = body.push(card(detail));
        }
        if rows.is_empty() {
            body = body.push(card(text("No comparable records for these conditions. Missing RPM, ambiguous ammo, explosives and test records cannot establish a fastest raw TTK. Try another mode/class or inspect the weapon's data.").size(13).width(Fill)));
        }
        for entry in rows {
            let w = &entry.weapon;
            let metrics = format!(
                "Raw TTK: {} · Raw DPS: {}",
                entry
                    .ttk
                    .map(|v| format!("{:.0} ms", v * 1000.))
                    .unwrap_or("Unknown".into()),
                number(entry.dps)
            );
            let mut entry_card = column![
                row![
                    text(format!("{}. {}", entry.position, w.name)).size(20),
                    space::horizontal(),
                    button("Use")
                        .on_press(Message::SelectWeapon(w.id.clone()))
                        .style(button::secondary)
                ],
                text(metrics).size(13).color(GREEN),
                text(format!(
                    "Damage/hit: {} · Recoil index: {}",
                    number(entry.damage),
                    number(entry.recoil)
                ))
                .size(12),
                text(ranking::availability(w, &self.profile))
                    .size(11)
                    .color(MUTED)
            ]
            .spacing(7);
            if let Some(note) = &entry.note {
                entry_card =
                    entry_card.push(text(note.as_str()).size(11).color(ACCENT).width(Fill));
            }
            if let Some(parts) = &entry.parts {
                entry_card = entry_card.push(row![
                    text(format!("Parts: {}", money(entry.build_cost.unwrap_or(0.)))).size(12),
                    space::horizontal(),
                    button("Parts")
                        .on_press(Message::RankParts(w.id.clone()))
                        .style(button::text)
                ]);
                if self.rank_parts.as_ref() == Some(&w.id)
                    && let Some(reference) = self.catalog.weapon(&w.id)
                {
                    for (slot, id) in parts {
                        if let Some(part) = lab::item(reference, id) {
                            entry_card = entry_card.push(
                                text(format!("{} · {}", slot.label(), part.name))
                                    .size(11)
                                    .color(MUTED)
                                    .width(Fill),
                            );
                        }
                    }
                    if parts.is_empty() {
                        entry_card = entry_card.push(
                            text("No priced modeled upgrade selected.")
                                .size(11)
                                .color(MUTED),
                        );
                    }
                }
            }
            body = body.push(card(entry_card));
        }
        let total = self
            .catalog
            .weapons
            .iter()
            .filter(|r| r.weapon.class_name == self.rank_class && !r.weapon.test_record)
            .count();
        body = body.push(text(format!("{} ranked / {} class records. Open Method & audit for exclusions. Build costs cover selected attachments only.", rows.len(), total)).size(11).color(MUTED).width(Fill));
        body.into()
    }
    fn spending_view(&self) -> Element<'_, Message> {
        let mut body=column![text("Worth spending more?").size(28),text(format!("{} · {} · ${} parts budget",self.reference.weapon.name,self.profile.goal.label(),self.profile.budget)).size(14),text("Your levels, ownership filter, kept slots and preferences apply. These are alternatives to choose from; opening this page changes nothing.").size(11).color(MUTED).width(Fill)].spacing(14);
        match &self.spending {
            Err(e) => body = body.push(card(text(e.as_str()).size(14))),
            Ok(report) => {
                body=body.push(card(text(if report.options.len()==1{"One distinct best-value setup fits these constraints. There is no useful extra spending tier to show.".into()}else if report.extra_cost>0.{format!("Beyond the sweet spot: +{} buys the last {:.1}% of available handling-score gain.",money(report.extra_cost),report.remaining_gain)}else{"The sweet spot already reaches the best modeled score within this budget.".into()}).size(15)));
                let mut previous: Option<&lab::Build> = None;
                for (index, option) in report.options.iter().enumerate() {
                    let mut detail = column![
                        row![
                            text(option.label).size(20),
                            space::horizontal(),
                            text(money(option.build.estimate.cost)).size(22)
                        ],
                        text(if report.has_gain {
                            format!(
                                "{:.1}% of available handling-score gain",
                                option.gain_share * 100.
                            )
                        } else {
                            "No additional modeled handling gain available.".into()
                        })
                        .size(11)
                        .color(MUTED)
                    ]
                    .spacing(9);
                    if let Some(old) = previous {
                        detail = detail.push(
                            text(format!(
                                "+{} versus the option above",
                                money(option.build.estimate.cost - old.estimate.cost)
                            ))
                            .size(12),
                        );
                        for change in insights::changes(&old.estimate, &option.build.estimate) {
                            detail = detail.push(change_text(&change));
                        }
                    }
                    for (slot, id) in &option.build.parts {
                        if let Some(part) = lab::item(&self.reference, id) {
                            detail = detail
                                .push(text(format!("{} · {}", slot.label(), part.name)).size(12));
                        }
                    }
                    if option.build.parts.is_empty() {
                        detail =
                            detail.push(text("No additional modeled parts selected.").size(12));
                    }
                    detail = detail.push(
                        button("Use this setup")
                            .on_press(Message::ApplySpending(index))
                            .padding(10),
                    );
                    body = body.push(card(detail));
                    previous = Some(&option.build);
                }
                body=body.push(text("Economy is the cheapest setup reaching 70% of available handling-score gain; Sweet spot reaches 95%; Maximum is the best score within your budget. Gain is measured from the cheapest feasible setup, under the same constraints. Identical tiers collapse. These thresholds are app preferences, not measured accuracy or kill-time improvements. Prices exclude unlock fees and the weapon; owned prices still count toward the selected-parts budget.").size(11).color(MUTED).width(Fill));
            }
        }
        body.push(
            button("Remaining purchase & unlock cost")
                .on_press(Message::Open(Page::Costs))
                .style(button::secondary),
        )
        .into()
    }
    fn patch_view(&self) -> Element<'_, Message> {
        let pending = self.update_candidate.is_some();
        let mut body = column![
            text("What changed for me?").size(28),
            text(if pending {
                "Update preview · not installed"
            } else {
                "Installed data · compared with the previous snapshot"
            })
            .size(12)
            .color(MUTED),
            text(self.reference.weapon.name.as_str()).size(22)
        ]
        .spacing(16);
        if let Some(report) = &self.patch_report {
            body = body.push(card(
                column![
                    text(report.title.as_str()).size(20),
                    text(report.versions.as_str()).size(11).color(MUTED)
                ]
                .spacing(10),
            ));
            for line in &report.lines {
                body = body.push(text(line.as_str()).size(13).width(Fill));
            }
            body = body.push(text(report.recommendation.as_str()).size(13).width(Fill));
            if !pending {
                body = body.push(
                    button("Review spending options")
                        .on_press(Message::Open(Page::Spending))
                        .style(button::secondary),
                );
            }
        } else {
            body=body.push(card(text(self.patch_error.as_deref().unwrap_or("No previous dataset is available yet. After an update, this view compares your selected weapon and parts with the saved snapshot.")).size(14)));
        }
        body.push(text("Source changes are not proof of a live game buff or nerf. Comparisons use the same current levels, goal, budget and kept slots on both datasets. They cover this weapon, selected attachments and the calculated suggestion; unmodeled game behavior remains unknown. Your saved parts are never replaced automatically.").size(11).color(MUTED).width(Fill)).push(button(if pending{"Back to update review"}else{"Data & updates"}).on_press(Message::Open(Page::Data)).style(button::secondary)).into()
    }
    fn range_view(&self) -> Element<'_, Message> {
        let mut body = column![
            text("Find your next weapon.").size(28),
            pick_list(
                self.catalog
                    .classes()
                    .into_iter()
                    .filter(|c| c != "Test / vehicle")
                    .collect::<Vec<_>>(),
                Some(self.rank_class.clone()),
                Message::RankClass
            )
            .width(Fill),
            row(range::Reach::ALL
                .into_iter()
                .map(|r| button(text(r.to_string()))
                    .on_press(Message::Reach(r))
                    .style(if r == self.reach {
                        button::primary
                    } else {
                        button::secondary
                    })
                    .width(Fill)
                    .into()))
            .spacing(8),
            text(format!(
                "{} m reference · {} handling · your levels and ${} parts budget",
                self.reach.distance(),
                self.reach.goal().label(),
                self.profile.budget
            ))
            .size(12)
            .color(MUTED)
        ]
        .spacing(16);
        let picks = &self.range_report.entries;
        if let Some(first) = picks.first() {
            let ties = picks
                .iter()
                .take_while(|e| e.position == first.position)
                .count();
            let mut pick = column![
                text(if ties > 1 {
                    "One of the tied recommendations"
                } else {
                    "Suggested for this range"
                })
                .size(12)
                .color(MUTED),
                text(first.weapon.name.clone()).size(30),
                text(format!(
                    "Ideal raw TTK: {} ms · parts {}",
                    first
                        .ttk
                        .map(|t| format!("{:.0}", t * 1000.))
                        .unwrap_or_else(|| "unknown".into()),
                    first
                        .build_cost
                        .map(money)
                        .unwrap_or_else(|| "unknown".into())
                ))
                .size(14)
            ]
            .spacing(10);
            if let Some(parts) = &first.parts {
                let reference = self.catalog.weapon(&first.weapon.id).unwrap();
                for (slot, id) in parts {
                    if let Some(p) = lab::item(reference, id) {
                        pick = pick.push(text(format!("{} · {}", slot.label(), p.name)).size(12));
                    }
                }
            }
            pick = pick.push(
                button("Use weapon & setup")
                    .on_press(Message::UseRange(first.weapon.id.clone()))
                    .padding(12),
            );
            body = body.push(card(pick));
        } else {
            body=body.push(card(text("No supported recommendation under these levels, budget and source data. Try another class or review your settings.").size(14)));
        }
        body=body.push(text("Equal weighting of ideal TTK and pattern recoil at 100 test HP. A reference distance, not a guaranteed effective range. No armor, misses or flight time. Unsupported damage curves are excluded. Your active pins and sight preferences are ignored for this new setup.").size(11).color(MUTED).width(Fill));
        body.push(
            button("Compare all class picks")
                .on_press(Message::Open(Page::Compare))
                .style(button::secondary),
        )
        .into()
    }
    fn cost_view(&self) -> Element<'_, Message> {
        let cost = costs::summary(&self.reference, &self.profile);
        let mut body=column![text("Cost to complete.").size(28),text(self.reference.weapon.name.as_str()).size(19),card(column![text(if cost.complete{money(cost.purchase+cost.fees as f64)}else{"Total incomplete".into()}).size(30),text(format!("Parts to buy: {} · unlock fees: {}",money(cost.purchase),money(cost.fees as f64))).size(13)].spacing(8)),text("Selected attachments only. Weapon, unchanged slots and ammunition are excluded. Owned parts cost $0 here. Unticked unlock fees are assumed unpaid; mark those you already paid. Level eligibility is separate. Your parts budget still excludes unlock fees.").size(11).color(MUTED).width(Fill)].spacing(16);
        for p in cost.parts {
            let mut detail = column![
                text(p.name).size(16),
                text(if p.owned {
                    "Owned · nothing left to pay".into()
                } else {
                    format!(
                        "Buy {} · unlock {}",
                        p.purchase.map(money).unwrap_or_else(|| "unknown".into()),
                        p.fee
                            .map(|n| money(n as f64))
                            .unwrap_or_else(|| "unknown".into())
                    )
                })
                .size(12)
            ]
            .spacing(8);
            if !p.owned {
                let id = p.id;
                detail = detail.push(
                    checkbox(p.unlocked)
                        .label("Already unlocked")
                        .on_toggle(move |v| Message::Unlocked(id.clone(), v)),
                );
            }
            body = body.push(card(detail));
        }
        body.push(
            button("Manage owned parts")
                .on_press(Message::Open(Page::Customize))
                .style(button::secondary),
        )
        .into()
    }
    fn recognition_view(&self) -> Element<'_, Message> {
        let mut body=column![text("Read your arsenal.").size(28),text("Experimental · local Windows OCR").size(11).color(ACCENT),text(if self.ocr_shortcut{"In the game arsenal, press CTRL + SHIFT + R."}else{"Use the timed button, then switch to the game arsenal."}).size(16),text("On demand, Windows reads text from the foreground WARDOGS window. Processing stays local. No image is uploaded; temporary capture files are deleted. Only the weapon name is matched, and you confirm before any setup changes.").size(12).color(MUTED).width(Fill),card(text(self.ocr_status.as_str()).size(14)),button(if self.ocr_busy{"Reading…"}else{"Read after 3 seconds"}).on_press_maybe((!self.ocr_busy).then_some(Message::Capture(true))).padding(12)].spacing(16);
        for found in &self.ocr_matches {
            body = body.push(card(
                row![
                    text(found.name.as_str()).size(18),
                    space::horizontal(),
                    button("Confirm").on_press(Message::UseDetected(found.id.clone()))
                ]
                .align_y(iced::Center),
            ));
        }
        body.push(button("Select manually").on_press(Message::Open(Page::Weapons)).style(button::secondary)).push(text("Keep the weapon name visible. Multiple inventory names can produce several matches. Exclusive fullscreen or unsupported OCR languages/resolutions may require manual selection. This feature does not read game memory.").size(11).color(MUTED).width(Fill)).into()
    }
    fn checks_view(&self) -> Element<'_, Message> {
        let mut body=column![text("Check a class in-game.").size(26),
            text("These are your reports, not automatic verification. Test in the arsenal, one attachment at a time. Numeric effects remain estimates when the game shows no stats.").size(12).color(MUTED).width(Fill)
        ].spacing(14).width(Fill);
        for (class, id) in [
            ("Assault rifles", "Id.Item.AK74M"),
            ("SMGs", "Id.Item.WEPN_028"),
            ("LMGs", "Id.Item.M249"),
            ("DMRs", "Id.Item.SKS"),
            ("Snipers", "Id.Item.SV98"),
            ("Pistols", "Id.Item.Glock17"),
        ] {
            let Some(reference) = self.catalog.weapon(id) else {
                continue;
            };
            let key = format!(
                "{}:{}:{}",
                self.catalog.game_build,
                self.catalog.source_commit,
                id.to_lowercase()
            );
            let check = self
                .profile
                .class_checks
                .get(&key)
                .cloned()
                .unwrap_or_default();
            let mut panel = column![row![
                text(format!("{class} · {}", reference.weapon.name)).size(16),
                space::horizontal(),
                button("Open")
                    .on_press(Message::SelectWeapon(id.into()))
                    .style(button::secondary)
            ]]
            .spacing(8);
            for (index, checked, label) in [
                (0, check.fits, "Attachment fits checked"),
                (1, check.requirements, "Levels and purchase prices checked"),
                (2, check.panel, "Panel and shortcut checked in-game"),
            ] {
                let key = key.clone();
                panel = panel.push(
                    checkbox(checked)
                        .label(label)
                        .on_toggle(move |value| Message::ClassCheck(key.clone(), index, value)),
                );
            }
            panel = panel.push(
                text(if check.fits && check.requirements && check.panel {
                    "Reported complete by you for this dataset"
                } else {
                    "Pending in-game checks"
                })
                .size(11)
                .color(MUTED),
            );
            body = body.push(card(panel));
        }
        body.push(text("Open replaces the active setup. Reports save locally and are keyed to the source build/revision. They do not certify numerical accuracy, anti-cheat acceptance or a whole weapon class.").size(11).color(MUTED).width(Fill)).into()
    }
    fn data_view(&self) -> Element<'_, Message> {
        let data = &self.catalog;
        let w = &self.reference.weapon;
        let mut body = column![text("Your data, clearly.").size(28),
            card(column![text(format!("Game build {}", data.game_build)).size(20),
                text(format!("Imported {} · {}",data.imported_on,self.data_status)).size(12),
                text(format!("{} weapon records · {} attachment records · {} checked source files",data.weapons.len(),data.all_attachments.len(),data.sources.len())).size(12),
                text(format!("{}\nRevision {}",data.source_url,data.source_commit)).size(10).color(MUTED).width(Fill),
                text("Pinned community snapshot, not live server data. Runtime reload validates structure and references; the update pipeline verifies original source checksums. Local game build was last checked as 25937608 on 2026-10-24.").size(11).color(MUTED).width(Fill)
            ].spacing(8)),
            row![button("All attachments").on_press(Message::Open(Page::Attachments)),button("Reload dataset").on_press(Message::ReloadData).style(button::secondary)].spacing(8),
            button("In-game class checks").on_press(Message::Open(Page::Checks)).style(button::secondary),
            button("What changed for me?").on_press(Message::Open(Page::Patch)).style(button::secondary),
            row![button(if self.update_busy{"Checking…"}else{"Check for updates"}).on_press_maybe((!self.update_busy).then_some(Message::CheckUpdates)),button("Restore previous data").on_press_maybe((!self.update_busy && updates::backup_path(&catalog::Catalog::path()).exists()).then_some(Message::RollbackUpdate)).style(button::secondary)].spacing(8),
            text(self.update_status.as_str()).size(12).width(Fill),
            text("Checks download data for review. Install saves the previous dataset; Restore brings it back. Reload reads Data/catalog.json beside the app (or WARDOG_DATA_PATH). Failed checks preserve active data. No background polling.").size(11).color(MUTED).width(Fill),
            text(format!("{} · {}", w.name,w.class_name)).size(22),
            text(format!("RPM: {} · ADS: {} s · Accuracy rating: {}", number(w.rpm),number(w.ads_time),number(w.accuracy))).size(12).width(Fill),
            text(format!("Fire modes: {}",w.fire_modes.join(", "))).size(12),
            text(format!("Ranking baseline: {}",w.projectile.as_deref().unwrap_or("No unambiguous single-caliber baseline"))).size(11).color(MUTED).width(Fill)
        ].spacing(14).width(Fill);
        if self.update_candidate.is_some() {
            body = body.push(
                button("Install reviewed update")
                    .on_press(Message::ApplyUpdate)
                    .padding(12),
            );
        }
        for row in &w.damage_rows {
            body = body.push(
                text(format!(
                    "{} · hit {} · raw DPS {}",
                    row.projectile,
                    number(row.damage),
                    number(row.raw_dps)
                ))
                .size(12)
                .width(Fill),
            );
        }
        for note in data
            .coverage_notes
            .iter()
            .filter(|note| note.weapon.eq_ignore_ascii_case(&w.id))
        {
            body = body.push(text(note.note.as_str()).size(11).color(ACCENT).width(Fill));
        }
        body.push(text("All source attachments are searchable, including hidden, test and conditional components. Only explicitly compatible, level-eligible, priced and supported effects can enter automatic suggestions. Missing nested mounts are not inferred from caliber.").size(11).color(MUTED).width(Fill)).into()
    }
    fn app_updates_view(&self) -> Element<'_, Message> {
        let mut body = column![
            text("Keep your companion ready.").size(28),
            card(
                column![
                    text(concat!("WARDOG Companion ", env!("CARGO_PKG_VERSION"))).size(20),
                    text("Windows · Open source · Made with Rust")
                        .size(12)
                        .color(MUTED),
                    text(&self.app_update_status).size(14).width(Fill),
                    button(if self.app_update_busy {
                        "Working…"
                    } else {
                        "Check for app updates"
                    })
                    .on_press_maybe((!self.app_update_busy).then_some(Message::CheckAppUpdate)),
                ]
                .spacing(14)
            ),
        ]
        .spacing(16)
        .width(Fill);
        if let Some(release) = &self.app_release {
            body = body.push(card(
                column![
                    text(format!("What's new in {}", release.version)).size(20),
                    text(&release.notes).size(12).width(Fill),
                ]
                .spacing(12),
            ));
            if self.app_installer.is_some() {
                body = body.push(
                    button("Install & close app")
                        .on_press(Message::InstallAppUpdate)
                        .padding(12),
                );
            } else {
                body = body.push(
                    button("Download update")
                        .on_press_maybe(
                            (!self.app_update_busy).then_some(Message::DownloadAppUpdate),
                        )
                        .padding(12),
                );
            }
        }
        body.push(text("Downloads come from cccarv82/wardog-companion on GitHub and are checked against GitHub's SHA-256 digest before Setup can run. This release is not code-signed.").size(11).color(MUTED).width(Fill))
            .push(text("App updates and game-data updates are separate. Setup keeps your profile; a portable copy stays where you extracted it.").size(11).color(MUTED).width(Fill)).into()
    }
    fn attachments_view(&self) -> Element<'_, Message> {
        let needle = self.attachment_search.to_lowercase();
        let matches: Vec<_> = self
            .catalog
            .all_attachments
            .iter()
            .filter(|part| {
                format!("{} {} {}", part.name, part.id, part.slot)
                    .to_lowercase()
                    .contains(&needle)
            })
            .collect();
        let mut body = column![
            text("Every published attachment.").size(25),
            text_input("Search name, ID or component type", &self.attachment_search)
                .on_input(Message::AttachmentSearch)
                .padding(10),
            text(format!(
                "{} matches · {} total records",
                matches.len(),
                self.catalog.all_attachments.len()
            ))
            .size(12)
            .color(MUTED)
        ]
        .spacing(12)
        .width(Fill);
        if let Some(part) = self
            .attachment_selected
            .as_ref()
            .and_then(|id| self.catalog.all_attachments.iter().find(|p| p.id == *id))
        {
            let compatible = self
                .reference
                .items
                .iter()
                .find(|p| p.id.eq_ignore_ascii_case(&part.id));
            let mut detail = column![
                text(part.name.as_str()).size(20),
                text(part.id.as_str()).size(10).color(MUTED),
                text(if compatible.is_some() {
                    "Explicit compatibility recorded for the selected weapon"
                } else {
                    "No explicit compatibility recorded for the selected weapon"
                })
                .size(12),
                text(part.price_text()).size(12).width(Fill)
            ]
            .spacing(7);
            if let Some(u) = &part.unlock {
                detail = detail.push(
                    text(format!(
                        "{} {} · unlock fee {}",
                        u.role,
                        u.level,
                        money(u.cost as f64)
                    ))
                    .size(12),
                );
            }
            if let Some(n) = part.magazine_capacity {
                detail = detail.push(text(format!("Capacity: {n} rounds")).size(12));
            }
            if part.conditional {
                detail = detail.push(
                    text("State-dependent effects; deployment is not assumed.")
                        .size(12)
                        .color(ACCENT),
                );
            }
            for m in part.modifiers.iter().chain(&part.character_modifiers) {
                detail = detail.push(text(m.description()).size(11).width(Fill));
            }
            body = body.push(card(detail));
        }
        for part in matches.iter().skip(self.attachment_page * 20).take(20) {
            body = body.push(
                button(text(format!("{} · {}", part.name, part.slot)).size(14))
                    .on_press(Message::InspectAttachment(part.id.clone()))
                    .width(Fill)
                    .padding(9)
                    .style(list_button),
            );
        }
        body.push(
            row![
                button("Previous").on_press_maybe(
                    (self.attachment_page > 0)
                        .then(|| Message::AttachmentPage(self.attachment_page - 1))
                ),
                text(format!("Page {}", self.attachment_page + 1)).size(12),
                button("Next").on_press_maybe(
                    ((self.attachment_page + 1) * 20 < matches.len())
                        .then_some(Message::AttachmentPage(self.attachment_page + 1))
                )
            ]
            .spacing(10)
            .align_y(iced::Center),
        )
        .into()
    }
    fn settings(&self) -> Element<'_, Message> {
        let mut body = column![
            text("Your levels.").size(29),
            text("Set once. Update when you level up.")
                .size(13)
                .color(MUTED)
        ]
        .spacing(16)
        .width(Fill);
        for track in Track::ALL {
            let value = self.profile.levels[track.index()];
            body = body.push(
                column![
                    row![
                        text(track.label()).size(14),
                        space::horizontal(),
                        text(value).size(18).color(ACCENT)
                    ],
                    slider(0..=100, value, move |v| Message::Level(track, v))
                ]
                .spacing(6),
            );
        }
        body = body
            .push(rule::horizontal(1))
            .push(text("Attachment budget").size(13))
            .push(
                text_input("e.g. 2000", &self.budget)
                    .on_input(Message::Budget)
                    .padding(10),
            )
            .push(
                checkbox(self.profile.owned_only)
                    .label("Only recommend items I own")
                    .on_toggle(Message::OwnedOnly),
            )
            .push(
                text("Choose Done to save and update your setup.")
                    .size(10)
                    .color(MUTED),
            );
        body = body.push(
            button("Data & updates")
                .on_press(Message::Open(Page::Data))
                .style(button::secondary),
        );
        body = body.push(
            button("App updates")
                .on_press(Message::Open(Page::AppUpdates))
                .style(button::secondary),
        );
        body.into()
    }
    fn customize(&self) -> Element<'_, Message> {
        if let Some((slot, choice)) = &self.pending_swap {
            return self.swap_preview(*slot, choice);
        }
        let mut body = column![
            text("Make it yours.").size(29),
            text("Choose a part. Keep it while the others adapt.")
                .size(13)
                .color(MUTED)
        ]
        .spacing(16)
        .width(Fill);
        let automatic = lab::Choice {
            id: None,
            label: "Automatic".into(),
        };
        let mut sights = lab::candidates(&self.reference, &self.profile, Slot::Optic);
        sights[0] = automatic.clone();
        let current_sight = self
            .profile
            .preferred_sight
            .as_ref()
            .map(|id| {
                lab::item(&self.reference, id)
                    .map(lab::Choice::item)
                    .unwrap_or_else(|| lab::Choice {
                        id: Some(id.clone()),
                        label: "Unavailable preference".into(),
                    })
            })
            .unwrap_or(automatic);
        body = body.push(card(column![
            text("Your preferences").size(15),
            text("Preferred sight").size(12).color(MUTED),
            pick_list(sights, Some(current_sight), Message::PreferredSight).width(Fill),
            text("Minimum magazine capacity").size(12).color(MUTED),
            pick_list([Capacity(0), Capacity(30), Capacity(60), Capacity(75)],
                Some(Capacity(self.profile.minimum_capacity)), Message::MinimumCapacity).width(Fill),
            text("Suggestions update and save automatically. Requirements and budget still apply.").size(10).color(MUTED).width(Fill)
        ].spacing(8)));
        if let Some(upgrade) = &self.upgrade {
            let mut next = column![
                row![
                    text("Next useful unlock").size(12).color(MUTED),
                    space::horizontal(),
                    button(if self.upgrade_details {
                        "Less"
                    } else {
                        "Details"
                    })
                    .on_press(Message::UpgradeDetails)
                    .style(button::text)
                ],
                text(format!(
                    "{} {} · {}",
                    upgrade.track.label(),
                    upgrade.level,
                    upgrade.name
                ))
                .size(16)
            ]
            .spacing(6);
            if self.upgrade_details {
                next = next.push(
                    text(format!(
                        "Suggested parts: {} · unlock fee: {} (separate)",
                        money(upgrade.cost),
                        money(upgrade.unlock_fee as f64)
                    ))
                    .size(11)
                    .color(MUTED)
                    .width(Fill),
                );
                for change in &upgrade.changes {
                    next = next.push(change_text(change));
                }
                next = next.push(text("Compared with the best current suggestion for your objective, budget and kept slots. Only this track is raised; other levels stay the same. Level requirements do not mean the item is purchased.").size(11).color(MUTED).width(Fill));
            }
            body = body.push(card(next));
        }
        let mut browse = self.profile.clone();
        browse.owned_only = false;
        for slot in lab::slots(&self.reference) {
            let selected = self
                .profile
                .lab_parts
                .get(&slot)
                .and_then(|id| lab::item(&self.reference, id));
            let current = selected.map(lab::Choice::item).unwrap_or_else(|| {
                self.profile
                    .lab_parts
                    .get(&slot)
                    .map(|id| lab::Choice {
                        id: Some(id.clone()),
                        label: "Unavailable saved item".into(),
                    })
                    .unwrap_or_else(lab::Choice::unchanged)
            });
            let kept = self.profile.lab_pins.contains(&slot);
            let mut part = column![
                row![
                    text(slot.label()).size(14),
                    space::horizontal(),
                    button(if kept { "Kept" } else { "Keep" })
                        .on_press(Message::Keep(slot))
                        .style(if kept { button::primary } else { button::text })
                ],
                pick_list(
                    lab::candidates(&self.reference, &browse, slot),
                    Some(current),
                    move |choice| Message::Pick(slot, choice)
                )
                .width(Fill)
            ]
            .spacing(10);
            if let Some(item) = selected {
                let id = item.id.clone();
                let check = checkbox(lab::owned(item, &self.profile)).label("I own this part");
                part = part.push(if item.is_default {
                    check
                } else {
                    check.on_toggle(move |value| Message::Owned(id.clone(), value))
                });
            }
            body = body.push(card(part));
        }
        body = body
            .push(
                button("Reset to recommendation")
                    .on_press(Message::Reset)
                    .style(button::secondary),
            )
            .push(
                text("Choose a part to preview the trade-off. Only Apply swap changes your setup.")
                    .size(11)
                    .color(MUTED)
                    .width(Fill),
            );
        if let Err(error) = &self.estimate {
            body = body.push(text(error.as_str()).size(12).color(ACCENT));
        }
        body.into()
    }
    fn swap_preview(&self, slot: Slot, choice: &lab::Choice) -> Element<'_, Message> {
        let current = self
            .profile
            .lab_parts
            .get(&slot)
            .and_then(|id| lab::item(&self.reference, id))
            .map(|part| part.name.as_str())
            .unwrap_or("Unchanged");
        let mut body = column![
            text("Before you swap.").size(29),
            text(slot.label()).size(12).color(MUTED),
            text(format!("{current} → {}", choice.label))
                .size(19)
                .width(Fill)
        ]
        .spacing(14)
        .width(Fill);
        match insights::compare(&self.reference, &self.profile, slot, choice) {
            Ok(preview) => {
                let mut effects = column![].spacing(9);
                if preview.before.is_none() {
                    effects = effects.push(text("Your previous setup is invalid, so a relative comparison is unavailable.").size(12).color(MUTED).width(Fill));
                } else if preview.changes.is_empty() {
                    effects = effects.push(text("No change in the modeled effects.").size(14));
                } else {
                    for beneficial in [true, false] {
                        let strongest = preview
                            .changes
                            .iter()
                            .filter(|c| c.beneficial() == beneficial)
                            .max_by(|a, b| a.percent.abs().total_cmp(&b.percent.abs()));
                        if let Some(change) = strongest {
                            effects = effects
                                .push(
                                    text(if beneficial {
                                        "Main gain"
                                    } else {
                                        "Main trade-off"
                                    })
                                    .size(11)
                                    .color(MUTED),
                                )
                                .push(change_text(change));
                        }
                    }
                    effects = effects.push(
                        button(if self.swap_details {
                            "Less detail"
                        } else {
                            "All effects"
                        })
                        .on_press(Message::SwapDetails)
                        .style(button::text),
                    );
                    if self.swap_details {
                        for change in &preview.changes {
                            effects = effects.push(change_text(change));
                        }
                    }
                }
                let cost = if let Some(before) = &preview.before {
                    if before.unknown_prices.is_empty() && preview.after.unknown_prices.is_empty() {
                        format!(
                            "Parts cost: {} → {} ({:+.2})",
                            money(before.cost),
                            money(preview.after.cost),
                            preview.after.cost - before.cost
                        )
                    } else {
                        "Cost difference unknown: a purchase price is missing.".into()
                    }
                } else {
                    preview.after.budget_status(self.profile.budget)
                };
                effects = effects.push(text(cost).size(13).width(Fill));
                if let Some(part) = choice
                    .id
                    .as_ref()
                    .and_then(|id| lab::item(&self.reference, id))
                {
                    if let Some(capacity) = part.magazine_capacity {
                        effects = effects
                            .push(text(format!("Magazine capacity: {capacity} rounds")).size(13));
                    }
                    if let Some(weight) = part.weight {
                        effects = effects.push(
                            text(format!("Part weight: {weight:.2} kg"))
                                .size(12)
                                .color(MUTED),
                        );
                    }
                }
                if !preview.after.unknown_prices.is_empty()
                    || preview.after.cost > self.profile.budget as f64
                {
                    effects = effects.push(
                        text(preview.after.budget_status(self.profile.budget))
                            .size(12)
                            .color(ACCENT)
                            .width(Fill),
                    );
                }
                if slot == Slot::Optic && self.profile.preferred_sight.is_some() {
                    effects = effects.push(
                        text("Your preferred sight will follow this change.")
                            .size(11)
                            .color(MUTED),
                    );
                } else if let Some(warning) = lab::preference_warning(
                    &self.reference,
                    &self.profile,
                    &insights::swapped(&self.profile, slot, choice),
                ) {
                    effects = effects.push(text(warning).size(11).color(ACCENT).width(Fill));
                }
                body = body.push(card(effects)).push(
                    row![
                        button("Apply swap")
                            .on_press(Message::ApplySwap)
                            .padding([10, 16]),
                        button("Cancel")
                            .on_press(Message::CancelSwap)
                            .style(button::secondary)
                            .padding([10, 16])
                    ]
                    .spacing(10),
                );
            }
            Err(error) => {
                body = body.push(card(text(error).size(13).color(ACCENT).width(Fill)));
                if let Some(part) = choice
                    .id
                    .as_ref()
                    .and_then(|id| lab::item(&self.reference, id))
                    && self.profile.owned_only
                    && !lab::owned(part, &self.profile)
                {
                    let id = part.id.clone();
                    body = body.push(
                        checkbox(false)
                            .label("I own this part")
                            .on_toggle(move |value| Message::Owned(id.clone(), value)),
                    );
                }
                body = body.push(button("Back to parts").on_press(Message::CancelSwap));
            }
        }
        body.push(text("Estimated changes relative to your saved selection. Unchanged slots are not detected; unmodeled effects may affect your choice. Nothing changes until you apply.").size(11).color(MUTED).width(Fill)).into()
    }
    fn explanation(&self) -> Element<'_, Message> {
        let mut details=column![text(self.profile.goal.description()).size(14),text("Based on this weapon's explicitly linked attachment records. Multiple recorded compound modifiers are multiplied, not added.").size(12).color(MUTED).width(Fill),
            button("Data version & coverage").on_press(Message::Open(Page::Data)).style(button::text)].spacing(10);
        if let Ok(estimate) = &self.estimate {
            for (index, (_, label)) in lab::ATTRIBUTES.iter().enumerate() {
                details = details.push(row![
                    text(*label).size(12),
                    space::horizontal(),
                    text(format!("{:+.2}%", (estimate.factors[index] - 1.) * 100.)).size(12)
                ]);
            }
            details = details.push(
                text(estimate.budget_status(self.profile.budget))
                    .size(11)
                    .color(MUTED)
                    .width(Fill),
            );
            details=details.push(text(format!("{} other effects are outside these metrics. Weight tiers, ammo, zoom, suppression and magazine capacity are not optimized.",estimate.ignored_effects)).size(11).color(MUTED).width(Fill));
        }
        details=details.push(text("Unchanged slots are not detected or modeled. Missing requirements/prices are excluded from suggestions. Keep a slot to stop the recommender changing it. Equal preference scores favor lower cost.").size(11).color(MUTED).width(Fill))
            .push(text(format!("{} · {} · imported {}",self.reference.weapon.name,self.reference.weapon.id,self.reference.imported_on)).size(10).color(MUTED).width(Fill))
            .push(text(format!("Source build {} / local build checked {}. Live server overrides and in-game overlay compatibility are not verified.",self.reference.game_build,self.reference.local_build_at_import)).size(10).color(MUTED).width(Fill))
            .push(text(format!("{}\nRevision {}",self.reference.source_url,self.reference.source_commit)).size(10).color(MUTED).width(Fill));
        for id in self.profile.lab_parts.values() {
            if let Some(item) = lab::item(&self.reference, id) {
                details = details
                    .push(rule::horizontal(1))
                    .push(text(format!("{} · {}", item.slot_label(), item.name)).size(12))
                    .push(text(item.price_text()).size(10).color(MUTED).width(Fill));
                if let Some(unlock) = &item.unlock {
                    details = details.push(
                        text(format!(
                            "Unlock: {} level {} · ${} fee (not included in the attachment budget)",
                            unlock.role, unlock.level, unlock.cost
                        ))
                        .size(10)
                        .color(MUTED)
                        .width(Fill),
                    );
                }
                if let Some(weight) = item.weight {
                    details = details.push(
                        text(format!("Recorded part weight: {weight:.2} kg"))
                            .size(10)
                            .color(MUTED),
                    );
                }
                if let Some(capacity) = item.magazine_capacity {
                    details = details.push(
                        text(format!("Magazine capacity: {capacity} rounds"))
                            .size(11)
                            .color(MUTED),
                    );
                }
                if let Some(source) = &item.compatibility_source {
                    details = details.push(
                        text(format!(
                            "Compatibility: {source} · checked 2026-10-24 (community source)"
                        ))
                        .size(10)
                        .color(MUTED)
                        .width(Fill),
                    );
                }
                for modifier in item.modifiers.iter().chain(&item.character_modifiers) {
                    details = details.push(
                        text(modifier.description())
                            .size(10)
                            .color(MUTED)
                            .width(Fill),
                    );
                }
            }
        }
        if let Some(error) = &self.shortcut_error {
            details = details.push(
                text(format!("Shortcut: {error}"))
                    .size(11)
                    .color(ACCENT)
                    .width(Fill),
            );
        }
        card(details).into()
    }
}
fn background<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
    message: impl Fn(T) -> Message + Send + 'static,
) -> Task<Message> {
    let (sender, receiver) = iced::futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let _ = sender.send(work());
    });
    Task::perform(receiver, move |result| match result {
        Ok(value) => message(value),
        Err(_) => Message::BackgroundFailed,
    })
}
fn change_text(change: &insights::Change) -> Element<'static, Message> {
    text(format!(
        "{} {:+.1}%",
        lab::ATTRIBUTES[change.index].1,
        change.percent
    ))
    .size(14)
    .color(if change.beneficial() { GREEN } else { ACCENT })
    .into()
}
fn money(value: f64) -> String {
    if value.fract().abs() < 0.000001 {
        format!("${value:.0}")
    } else {
        format!("${value:.2}")
    }
}
fn number(value: Option<f64>) -> String {
    value
        .map(|n| format!("{n:.2}"))
        .unwrap_or_else(|| "Not published".into())
}
fn card<'a>(content: impl Into<Element<'a, Message>>) -> iced::widget::Container<'a, Message> {
    container(content).padding(16).width(Fill).style(panel)
}
fn panel(_: &Theme) -> container::Style {
    container::Style {
        background: Some(PANEL.into()),
        border: Border {
            color: LINE,
            width: 1.,
            radius: 12.into(),
        },
        ..Default::default()
    }
}
fn list_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::secondary(theme, status);
    style.background = Some(
        if matches!(status, button::Status::Hovered | button::Status::Pressed) {
            Color::from_rgb(0.10, 0.13, 0.15).into()
        } else {
            PANEL.into()
        },
    );
    style.text_color = Color::from_rgb(0.93, 0.95, 0.96);
    style.border = Border {
        color: LINE,
        width: 1.,
        radius: 8.into(),
    };
    style
}
