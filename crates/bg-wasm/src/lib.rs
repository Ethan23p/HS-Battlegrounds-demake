//! The WebAssembly seam: JSON in, a resolved Action Phase out.
//!
//! This crate exists so that `bg-sim` never has to know a browser exists. It is
//! the same shape as `bg-cli` -- a shallow frontend adapter over a deep engine --
//! and it holds no rules of its own. Anything here that looks like a game
//! decision is a bug.
//!
//! The interface is deliberately one function. A frontend hands over a Seed and
//! two Parties; it gets back the Board as it stood before anyone acted, and the
//! log of everything that happened. That is enough to draw every frame: the log
//! is an ordered list of mutations, so replaying the first *k* of them onto the
//! initial Board yields the Board at step *k*, which is what makes a scrubber
//! possible without the engine storing a frame history.
//!
//! ## The ABI
//!
//! No `wasm-bindgen`, and deliberately so: the surface is one string in and one
//! string out, which the C ABI expresses in four exports and about twenty lines
//! of JavaScript. A codegen dependency would cost the project a build tool for
//! a seam this small.
//!
//! Buffers cross as `[u32 length, little-endian][UTF-8 bytes]`, so a caller
//! reads the length from the pointer it was handed and never has to be told a
//! size out of band.

use std::alloc::{Layout, alloc, dealloc};

use bg_sim::action_phase::{Resolution, board_of, resolve};
use bg_sim::party::{Board, Unit};
use bg_sim::rng::{Domain, Seed};
use bg_sim::units::{DefId, Keyword, Tribe, UnitDef};
use serde::{Deserialize, Serialize};

/// A Unit as a frontend describes it: stats and keywords, no abilities yet.
///
/// This mirrors the subset of [`UnitDef`] that a Party built by hand needs.
/// When Unit Definitions load from data files, this shrinks to a `DefId`.
#[derive(Debug, Clone, Deserialize)]
struct UnitSpec {
    name: String,
    attack: i32,
    health: i32,
    #[serde(default)]
    keywords: Vec<Keyword>,
    #[serde(default)]
    tribes: Vec<Tribe>,
}

impl UnitSpec {
    fn build(&self) -> Unit {
        Unit::new(&UnitDef {
            id: DefId::new(self.name.to_lowercase().replace(' ', "_")),
            name: self.name.clone(),
            tier: 1,
            attack: self.attack,
            health: self.health,
            tribes: self.tribes.clone(),
            all_tribes: false,
            keywords: self.keywords.clone(),
            abilities: Vec::new(),
            token: false,
            text: String::new(),
        })
    }
}

/// The request: a Seed and two Parties.
#[derive(Debug, Clone, Deserialize)]
struct Request {
    seed: u64,
    player: Vec<UnitSpec>,
    opposing: Vec<UnitSpec>,
}

/// The reply: where the Board started, and everything that happened to it.
#[derive(Debug, Clone, Serialize)]
struct Reply {
    /// The Board before Beat 0 of Pass 1. The log replays onto this.
    initial: Board,
    #[serde(flatten)]
    resolution: Resolution,
    /// The engine's own prose rendering, so a frontend can show the log as text
    /// without reimplementing the phrasing.
    narration: Vec<String>,
}

/// Resolve one Action Phase, reporting a bad request as an error rather than a
/// panic -- a panic across the wasm boundary aborts the module for good.
fn run(request: &str) -> Result<String, String> {
    let request: Request = serde_json::from_str(request).map_err(|e| e.to_string())?;
    if request.player.len() > bg_sim::party::SLOTS || request.opposing.len() > bg_sim::party::SLOTS
    {
        return Err(format!(
            "a Party holds at most {} Units",
            bg_sim::party::SLOTS
        ));
    }

    let board: Board = board_of(
        request.player.iter().map(UnitSpec::build).collect(),
        request.opposing.iter().map(UnitSpec::build).collect(),
    );
    let initial = board.clone();

    let mut rng = Seed(request.seed).stream(Domain::Combat, 0);
    let resolution = resolve(board, &mut rng);
    let narration = resolution.log.iter().map(|e| e.to_string()).collect();

    serde_json::to_string(&Reply {
        initial,
        resolution,
        narration,
    })
    .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// The C ABI. Everything below is plumbing; no rules live here.
// ---------------------------------------------------------------------------

/// Hand the caller a buffer of `len` bytes to write a request into.
///
/// # Safety
/// The caller must pass the returned pointer, with the same `len`, to exactly
/// one of [`bg_free`] or [`bg_resolve`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bg_alloc(len: usize) -> *mut u8 {
    if len == 0 {
        return std::ptr::null_mut();
    }
    let Ok(layout) = Layout::from_size_align(len, 1) else {
        return std::ptr::null_mut();
    };
    unsafe { alloc(layout) }
}

/// Release a buffer obtained from [`bg_alloc`].
///
/// # Safety
/// `ptr` must have come from [`bg_alloc`] with the same `len`, and must not
/// have been freed already.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bg_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    if let Ok(layout) = Layout::from_size_align(len, 1) {
        unsafe { dealloc(ptr, layout) };
    }
}

/// Length-prefix a string into a freshly allocated buffer the caller owns.
fn into_prefixed(body: &str) -> *mut u8 {
    let bytes = body.as_bytes();
    let total = 4 + bytes.len();
    let layout = Layout::from_size_align(total, 1).expect("a byte layout is always valid");
    // SAFETY: `total` is non-zero, so the layout has a non-zero size.
    let out = unsafe { alloc(layout) };
    if out.is_null() {
        return out;
    }
    // SAFETY: `out` owns `total` bytes, and the two writes cover exactly those.
    unsafe {
        std::ptr::copy_nonoverlapping((bytes.len() as u32).to_le_bytes().as_ptr(), out, 4);
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out.add(4), bytes.len());
    }
    out
}

/// Resolve an Action Phase described by `len` UTF-8 bytes at `ptr`.
///
/// Consumes the request buffer. Returns a pointer to `[u32 length][UTF-8 JSON]`
/// which the caller must release with [`bg_free`], passing `4 + length`. The
/// reply is either the resolved fight or `{"error": "..."}`; it is never a
/// panic, because a panic here would poison the module.
///
/// # Safety
/// `ptr` must be a [`bg_alloc`] buffer of exactly `len` bytes holding UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bg_resolve(ptr: *mut u8, len: usize) -> *mut u8 {
    let request = if ptr.is_null() || len == 0 {
        String::new()
    } else {
        // SAFETY: the caller guarantees `len` readable bytes at `ptr`.
        let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
        let owned = String::from_utf8_lossy(bytes).into_owned();
        unsafe { bg_free(ptr, len) };
        owned
    };

    match run(&request) {
        Ok(reply) => into_prefixed(&reply),
        Err(why) => into_prefixed(
            &serde_json::to_string(&serde_json::json!({ "error": why }))
                .unwrap_or_else(|_| r#"{"error":"unreportable"}"#.to_owned()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_described_fight_resolves_to_json() {
        let reply = run(r#"{"seed": 1,
                "player": [{"name": "Bait", "attack": 1, "health": 1}],
                "opposing": [{"name": "Brute", "attack": 3, "health": 4}]}"#)
        .expect("a well-formed request resolves");
        let value: serde_json::Value = serde_json::from_str(&reply).expect("valid JSON");
        assert_eq!(value["outcome"], "OpposingWins");
        assert!(
            value["initial"]["player"]["slots"][0]["health"] == 1,
            "the initial Board is reported before anything hit it"
        );
        assert!(!value["log"].as_array().expect("a log").is_empty());
    }

    #[test]
    fn keywords_and_tribes_survive_the_crossing() {
        let reply = run(r#"{"seed": 7,
                "player": [{"name": "Gusty", "attack": 2, "health": 9,
                            "keywords": ["Windfury"], "tribes": ["Beast"]}],
                "opposing": [{"name": "Wall", "attack": 0, "health": 40}]}"#)
        .expect("resolves");
        let value: serde_json::Value = serde_json::from_str(&reply).expect("valid JSON");
        let unit = &value["initial"]["player"]["slots"][0];
        assert_eq!(unit["keywords"][0], "Windfury");
        assert_eq!(unit["tribes"][0], "Beast");
        // Windfury means two Struck events in the Beat, not one.
        let strikes = value["log"]
            .as_array()
            .expect("a log")
            .iter()
            .filter(|e| e["kind"] == "Struck")
            .count();
        assert!(strikes >= 2, "Windfury strikes twice in its Beat");
    }

    #[test]
    fn a_malformed_request_is_an_error_not_a_panic() {
        assert!(run("not json").is_err());
        assert!(run(r#"{"seed": 1, "player": [], "opposing": []}"#).is_ok());
    }

    #[test]
    fn an_oversized_party_is_refused() {
        let many: Vec<String> = (0..9)
            .map(|i| format!(r#"{{"name":"u{i}","attack":1,"health":1}}"#))
            .collect();
        let request = format!(
            r#"{{"seed":1,"player":[{}],"opposing":[]}}"#,
            many.join(",")
        );
        assert!(run(&request).is_err(), "nine Units do not fit eight Slots");
    }
}
