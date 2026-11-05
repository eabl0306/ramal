//! Casos de uso del grafo: ventanas con cache y filtros, y la busqueda en todo
//! el historial.

mod history_filler;
mod load_graph_window;
mod search_history;

pub use load_graph_window::{LoadGraphWindow, WindowRequest};
pub use search_history::{MAX_SEARCH_RESULTS, SearchHistory};
