use crate::api::types::{AutocompleteResponse, Place, SearchResponse};

pub enum Action {
    AutocompleteResult(Result<AutocompleteResponse, String>),
    SearchResult(Result<SearchResponse, String>),
    DetailsResult(Result<Place, String>),
}
