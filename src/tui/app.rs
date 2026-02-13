use std::sync::Arc;
use std::time::Instant;

use ratatui::widgets::ListState;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use tracing::info;
use tui_input::Input;

use crate::api::client::Client;
use crate::api::types::{
    price_level_to_api, AutocompleteRequest, Circle, DetailsRequest, LatLng, Place, SearchRequest,
    Suggestion,
};
use crate::config::Config;
use crate::tui::event::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    SearchInput,
    AutocompleteList,
    ResultsList,
    FilterPanel,
    FilterEditing, // editing a text field inside the filter panel
}

/// Which filter row is selected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterField {
    Type,      // included_type (text)
    Radius,    // cycle: 500, 1000, 2000, 5000, 10000, 25000, 50000
    MinRating, // cycle: None, 3.0, 3.5, 4.0, 4.5
    Price,     // toggle individual price levels 0-4
    OpenNow,   // toggle bool
}

impl FilterField {
    pub const ALL: [FilterField; 5] = [
        FilterField::Type,
        FilterField::Radius,
        FilterField::MinRating,
        FilterField::Price,
        FilterField::OpenNow,
    ];

    pub fn from_index(i: usize) -> Self {
        FilterField::ALL[i % FilterField::ALL.len()]
    }
}

pub struct App {
    pub should_quit: bool,
    pub focus: Focus,
    pub loading: bool,
    pub status: Option<(String, bool)>, // (message, is_error)
    pub last_ctrl_c: Option<Instant>,

    // Search input
    pub input: Input,
    pub autocomplete: Vec<Suggestion>,
    pub ac_selected: usize,
    pub ac_task: Option<JoinHandle<()>>,

    // Filters
    pub filter_selected: usize, // index into FilterField::ALL
    pub filter_type_input: Input,
    pub filter_type_matches: Vec<&'static str>,
    pub filter_type_match_idx: usize,
    pub filter_radius: f64,            // meters
    pub filter_min_rating: Option<f64>,
    pub filter_price_levels: [bool; 5], // indices 0-4 (Free, $, $$, $$$, $$$$)
    pub filter_open_now: bool,

    // Results
    pub results: Vec<Place>,
    pub results_state: ListState,

    // Details (right pane)
    pub detail: Option<Place>,
    pub detail_scroll: u16,

    // Shared
    pub client: Arc<Client>,
    pub config: Config,
    pub session_token: String,
    pub action_tx: UnboundedSender<Action>,
}

impl App {
    pub fn new(client: Arc<Client>, config: Config, action_tx: UnboundedSender<Action>) -> Self {
        Self {
            should_quit: false,
            focus: Focus::SearchInput,
            loading: false,
            status: None,
            last_ctrl_c: None,

            input: Input::default(),
            autocomplete: Vec::new(),
            ac_selected: 0,
            ac_task: None,

            filter_selected: 0,
            filter_type_input: Input::default(),
            filter_type_matches: Vec::new(),
            filter_type_match_idx: 0,
            filter_radius: config.default_radius(),
            filter_min_rating: None,
            filter_price_levels: [false; 5],
            filter_open_now: false,

            results: Vec::new(),
            results_state: ListState::default(),

            detail: None,
            detail_scroll: 0,

            client,
            config,
            session_token: uuid::Uuid::new_v4().to_string(),
            action_tx,
        }
    }

    pub fn update_type_matches(&mut self) {
        use crate::tui::place_types::filter_types;
        self.filter_type_matches = filter_types(self.filter_type_input.value(), 6);
        self.filter_type_match_idx = 0;
    }

    const RADIUS_OPTIONS: [f64; 7] = [500.0, 1000.0, 2000.0, 5000.0, 10000.0, 25000.0, 50000.0];

    pub fn cycle_radius(&mut self) {
        let current = self.filter_radius;
        // Find the next value after the current one
        let next = Self::RADIUS_OPTIONS
            .iter()
            .find(|&&r| r > current)
            .copied()
            .unwrap_or(Self::RADIUS_OPTIONS[0]);
        self.filter_radius = next;
    }

    pub fn cycle_min_rating(&mut self) {
        self.filter_min_rating = match self.filter_min_rating {
            None => Some(3.0),
            Some(r) if r < 3.5 => Some(3.5),
            Some(r) if r < 4.0 => Some(4.0),
            Some(r) if r < 4.5 => Some(4.5),
            _ => None,
        };
    }

    pub fn trigger_autocomplete(&mut self) {
        // Cancel previous autocomplete task
        if let Some(handle) = self.ac_task.take() {
            handle.abort();
        }

        let query = self.input.value().to_string();
        if query.is_empty() {
            self.autocomplete.clear();
            self.ac_selected = 0;
            return;
        }

        let client = Arc::clone(&self.client);
        let tx = self.action_tx.clone();
        let session_token = self.session_token.clone();
        let location = self.location_bias();

        let handle = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            info!(input = %query, "Autocomplete request");

            let req = AutocompleteRequest {
                input: query,
                session_token: Some(session_token),
                location,
                limit: Some(5),
                language: None,
                region: None,
            };

            let result = client.autocomplete(&req).await;
            let _ = tx.send(Action::AutocompleteResult(
                result.map_err(|e| e.to_string()),
            ));
        });

        self.ac_task = Some(handle);
    }

    pub fn execute_search(&mut self, query: String) {
        if query.is_empty() {
            return;
        }

        // Cancel any pending autocomplete
        if let Some(handle) = self.ac_task.take() {
            handle.abort();
        }
        self.autocomplete.clear();
        self.ac_selected = 0;

        self.loading = true;
        self.status = Some(("Searching...".to_string(), false));

        // New session token after search (per Google billing best practice)
        self.session_token = uuid::Uuid::new_v4().to_string();

        let client = Arc::clone(&self.client);
        let tx = self.action_tx.clone();
        let location = self.location_bias();

        // Build filter values for the spawned task
        let included_type = {
            let v = self.filter_type_input.value().to_string();
            if v.is_empty() { None } else { Some(v) }
        };
        let min_rating = self.filter_min_rating;
        let price_levels: Vec<String> = self
            .filter_price_levels
            .iter()
            .enumerate()
            .filter(|(_, &v)| v)
            .filter_map(|(i, _)| price_level_to_api(i as u8).map(String::from))
            .collect();
        let open_now = self.filter_open_now;

        tokio::spawn(async move {
            info!(
                query = %query,
                included_type = ?included_type,
                min_rating = ?min_rating,
                price_levels = ?price_levels,
                open_now = open_now,
                "Search request"
            );

            let req = SearchRequest {
                query,
                included_type,
                min_rating,
                price_levels,
                open_now,
                location,
                limit: Some(10),
                language: None,
                region: None,
            };

            let result = client.search(&req).await;
            let _ = tx.send(Action::SearchResult(result.map_err(|e| e.to_string())));
        });
    }

    pub fn fetch_details(&mut self) {
        let place_id = match self.selected_place() {
            Some(p) if !p.id.is_empty() => p.id.clone(),
            _ => return,
        };

        self.loading = true;
        self.status = Some(("Loading details...".to_string(), false));

        let client = Arc::clone(&self.client);
        let tx = self.action_tx.clone();

        tokio::spawn(async move {
            info!(place_id = %place_id, "Details request");

            let req = DetailsRequest {
                place_id,
                include_reviews: true,
                include_photos: false,
                language: None,
                region: None,
            };

            let result = client.details(&req).await;
            let _ = tx.send(Action::DetailsResult(result.map_err(|e| e.to_string())));
        });
    }

    pub fn selected_place(&self) -> Option<&Place> {
        self.results_state
            .selected()
            .and_then(|i| self.results.get(i))
    }

    pub fn select_next_result(&mut self) {
        let len = self.results.len();
        if len == 0 {
            return;
        }
        let i = self.results_state.selected().map_or(0, |i| {
            if i + 1 >= len { i } else { i + 1 }
        });
        self.results_state.select(Some(i));
        self.update_detail_from_selection();
    }

    pub fn select_prev_result(&mut self) {
        let len = self.results.len();
        if len == 0 {
            return;
        }
        let i = self
            .results_state
            .selected()
            .map_or(0, |i| i.saturating_sub(1));
        self.results_state.select(Some(i));
        self.update_detail_from_selection();
    }

    fn update_detail_from_selection(&mut self) {
        self.detail = self.selected_place().cloned();
        self.detail_scroll = 0;
    }

    fn location_bias(&self) -> Option<Circle> {
        self.config.default_location().map(|(lat, lng)| Circle {
            center: LatLng {
                latitude: lat,
                longitude: lng,
            },
            radius: self.filter_radius,
        })
    }
}
