// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer COSMIC - Send Page

use crate::engine::EngineBridge;
use cosmic::iced::Length;
use cosmic::widget::{self, button, container, text, text_input};
use cosmic::{theme, Element, Task};
use gosh_lan_transfer::Favorite;
use gosh_transfer_core::FileFavoritesStore;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum SendMessage {
    AddressChanged(String),
    PortChanged(String),
    ResolveAddress,
    AddressResolved(gosh_lan_transfer::ResolveResult),
    BrowseFiles,
    #[allow(dead_code)] // Will be used when file picker is implemented
    FilesSelected(Vec<PathBuf>),
    RemoveFile(usize),
    ClearFiles,
    StartTransfer,
    TransferStarted,
    TransferError(String),
    // Favorites
    FavoritesLoaded(Vec<Favorite>),
    SelectFavorite(usize),
    SaveFavorite,
    FavoriteSaved,
    FavoriteNameChanged(String),
    ToggleSaveFavoriteDialog,
}

pub struct SendPage {
    address: String,
    port: String,
    selected_files: Vec<PathBuf>,
    is_resolving: bool,
    is_sending: bool,
    error_message: Option<String>,
    resolve_result: Option<gosh_lan_transfer::ResolveResult>,
    // Favorites
    favorites: Vec<Favorite>,
    favorites_store: Option<Arc<FileFavoritesStore>>,
    selected_favorite_idx: Option<usize>,
    show_save_dialog: bool,
    new_favorite_name: String,
}

impl SendPage {
    pub fn new() -> Self {
        // Try to load favorites store
        let favorites_store = FileFavoritesStore::new().ok().map(Arc::new);
        let favorites = favorites_store
            .as_ref()
            .and_then(|store| {
                use gosh_lan_transfer::FavoritesPersistence;
                store.list().ok()
            })
            .unwrap_or_default();

        Self {
            address: String::new(),
            port: "53317".to_string(),
            selected_files: Vec::new(),
            is_resolving: false,
            is_sending: false,
            error_message: None,
            resolve_result: None,
            favorites,
            favorites_store,
            selected_favorite_idx: None,
            show_save_dialog: false,
            new_favorite_name: String::new(),
        }
    }

    pub fn view(&self) -> Element<'_, SendMessage> {
        let spacing = theme::active().cosmic().spacing;

        // Header
        let header = text::title3("Send Files");

        // Favorites section
        let favorites_content: Element<SendMessage> = if self.favorites.is_empty() {
            container(text::caption("No favorites saved"))
                .padding(spacing.space_s)
                .into()
        } else {
            let mut favorites_column = widget::column().spacing(spacing.space_xs);
            for (idx, fav) in self.favorites.iter().enumerate() {
                let is_selected = self.selected_favorite_idx == Some(idx);
                let fav_button = if is_selected {
                    button::suggested(format!("{} ({})", fav.name, fav.address))
                        .on_press(SendMessage::SelectFavorite(idx))
                } else {
                    button::text(format!("{} ({})", fav.name, fav.address))
                        .on_press(SendMessage::SelectFavorite(idx))
                };
                favorites_column = favorites_column.push(fav_button);
            }
            favorites_column.into()
        };

        let favorites_card = container(
            widget::column()
                .push(text::title4("Favorites"))
                .push(text::caption("Quick access to saved destinations"))
                .push(favorites_content)
                .spacing(spacing.space_s),
        )
        .padding(spacing.space_m)
        .class(theme::Container::Card);

        // Address input
        let address_input = text_input("Hostname or IP address", &self.address)
            .on_input(SendMessage::AddressChanged)
            .width(Length::Fill);

        let port_input = text_input("Port", &self.port)
            .on_input(SendMessage::PortChanged)
            .width(Length::Fixed(100.0));

        let resolve_button = if self.is_resolving {
            button::text("Resolving...")
        } else {
            button::text("Resolve").on_press(SendMessage::ResolveAddress)
        };

        let address_row = widget::row()
            .push(address_input)
            .push(port_input)
            .push(resolve_button)
            .spacing(spacing.space_s);

        // Save to favorites button
        let save_favorite_button = if !self.address.is_empty() {
            button::icon(widget::icon::from_name("starred-symbolic"))
                .on_press(SendMessage::ToggleSaveFavoriteDialog)
        } else {
            button::icon(widget::icon::from_name("non-starred-symbolic"))
        };

        let save_favorite_row = widget::row()
            .push(text::body("Save to Favorites"))
            .push(widget::horizontal_space())
            .push(save_favorite_button)
            .align_y(cosmic::iced::Alignment::Center)
            .spacing(spacing.space_s);

        // Save favorite dialog (inline)
        let save_dialog_content: Element<SendMessage> = if self.show_save_dialog {
            container(
                widget::column()
                    .push(text::body("Enter a name for this favorite:"))
                    .push(
                        text_input("Name", &self.new_favorite_name)
                            .on_input(SendMessage::FavoriteNameChanged)
                            .width(Length::Fill),
                    )
                    .push(
                        widget::row()
                            .push(
                                button::text("Cancel")
                                    .on_press(SendMessage::ToggleSaveFavoriteDialog),
                            )
                            .push(button::suggested("Save").on_press(SendMessage::SaveFavorite))
                            .spacing(spacing.space_s),
                    )
                    .spacing(spacing.space_s),
            )
            .padding(spacing.space_m)
            .class(theme::Container::Card)
            .into()
        } else {
            widget::Space::new(0, 0).into()
        };

        // Resolve result
        let result_text: Element<SendMessage> = if let Some(ref result) = self.resolve_result {
            if result.success {
                text::body(format!("Resolved: {}", result.ips.join(", "))).into()
            } else {
                text::body(result.error.clone().unwrap_or_default()).into()
            }
        } else {
            text::body("").into()
        };

        // Files section
        let files_header = text::title4("Files");

        let files_content: Element<SendMessage> = if self.selected_files.is_empty() {
            container(
                widget::column()
                    .push(text::body("No files selected"))
                    .push(button::text("Browse Files").on_press(SendMessage::BrowseFiles))
                    .spacing(spacing.space_s)
                    .align_x(cosmic::iced::Alignment::Center),
            )
            .width(Length::Fill)
            .padding(spacing.space_m)
            .into()
        } else {
            let mut file_list = widget::column().spacing(spacing.space_xs);

            for (idx, path) in self.selected_files.iter().enumerate() {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                let row = widget::row()
                    .push(text::body(name))
                    .push(widget::horizontal_space())
                    .push(
                        button::icon(widget::icon::from_name("window-close-symbolic"))
                            .on_press(SendMessage::RemoveFile(idx)),
                    )
                    .spacing(spacing.space_s);

                file_list = file_list.push(row);
            }

            widget::column()
                .push(file_list)
                .push(
                    widget::row()
                        .push(button::text("Add More").on_press(SendMessage::BrowseFiles))
                        .push(button::text("Clear All").on_press(SendMessage::ClearFiles))
                        .spacing(spacing.space_s),
                )
                .spacing(spacing.space_m)
                .into()
        };

        // Send button
        let can_send = !self.address.is_empty()
            && !self.selected_files.is_empty()
            && !self.is_sending
            && self.resolve_result.as_ref().map(|r| r.success).unwrap_or(false);

        let send_button = if can_send {
            button::suggested("Send Files").on_press(SendMessage::StartTransfer)
        } else {
            button::suggested("Send Files")
        };

        // Error message
        let error_text: Element<SendMessage> = if let Some(ref err) = self.error_message {
            text::body(err).into()
        } else {
            text::body("").into()
        };

        widget::column()
            .push(header)
            .push(favorites_card)
            .push(address_row)
            .push(save_favorite_row)
            .push(save_dialog_content)
            .push(result_text)
            .push(files_header)
            .push(files_content)
            .push(error_text)
            .push(send_button)
            .spacing(spacing.space_m)
            .padding(spacing.space_m)
            .into()
    }

    pub fn update(&mut self, message: SendMessage, engine: &Arc<EngineBridge>) -> Task<SendMessage> {
        match message {
            SendMessage::AddressChanged(addr) => {
                self.address = addr;
                self.resolve_result = None;
                Task::none()
            }
            SendMessage::PortChanged(port) => {
                self.port = port;
                Task::none()
            }
            SendMessage::ResolveAddress => {
                self.is_resolving = true;
                let address = self.address.clone();
                let engine = engine.clone();

                cosmic::task::future(async move {
                    let result = engine.resolve_address(&address).await;
                    SendMessage::AddressResolved(result)
                })
            }
            SendMessage::AddressResolved(result) => {
                self.is_resolving = false;
                self.resolve_result = Some(result);
                Task::none()
            }
            SendMessage::BrowseFiles => {
                // TODO: Implement file picker
                Task::none()
            }
            SendMessage::FilesSelected(paths) => {
                self.selected_files.extend(paths);
                Task::none()
            }
            SendMessage::RemoveFile(idx) => {
                if idx < self.selected_files.len() {
                    self.selected_files.remove(idx);
                }
                Task::none()
            }
            SendMessage::ClearFiles => {
                self.selected_files.clear();
                Task::none()
            }
            SendMessage::StartTransfer => {
                self.is_sending = true;
                self.error_message = None;

                let engine = engine.clone();
                let address = self.address.clone();
                let port: u16 = self.port.parse().unwrap_or(53317);
                let paths = self.selected_files.clone();

                cosmic::task::future(async move {
                    match engine.send_files(&address, port, paths).await {
                        Ok(_) => SendMessage::TransferStarted,
                        Err(e) => SendMessage::TransferError(e.to_string()),
                    }
                })
            }
            SendMessage::TransferStarted => {
                self.is_sending = false;
                self.selected_files.clear();
                Task::none()
            }
            SendMessage::TransferError(err) => {
                self.is_sending = false;
                self.error_message = Some(err);
                Task::none()
            }
            // Favorites handlers
            SendMessage::FavoritesLoaded(favorites) => {
                self.favorites = favorites;
                Task::none()
            }
            SendMessage::SelectFavorite(idx) => {
                if let Some(fav) = self.favorites.get(idx) {
                    self.address = fav.address.clone();
                    self.selected_favorite_idx = Some(idx);
                    self.resolve_result = None;
                }
                Task::none()
            }
            SendMessage::ToggleSaveFavoriteDialog => {
                self.show_save_dialog = !self.show_save_dialog;
                if self.show_save_dialog {
                    self.new_favorite_name = self.address.clone();
                }
                Task::none()
            }
            SendMessage::FavoriteNameChanged(name) => {
                self.new_favorite_name = name;
                Task::none()
            }
            SendMessage::SaveFavorite => {
                if let Some(ref store) = self.favorites_store {
                    use gosh_lan_transfer::FavoritesPersistence;
                    if let Ok(fav) = store.add(self.new_favorite_name.clone(), self.address.clone())
                    {
                        self.favorites.push(fav);
                    }
                }
                self.show_save_dialog = false;
                self.new_favorite_name.clear();
                Task::none()
            }
            SendMessage::FavoriteSaved => {
                self.show_save_dialog = false;
                self.new_favorite_name.clear();
                Task::none()
            }
        }
    }
}
