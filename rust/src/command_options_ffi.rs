//! Caller-owned UTF-8 buffers and opaque session identities, no native handles.
use crate::{command_option_schemas, command_options::*};
use std::{
    collections::BTreeMap,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Om9OptionValue {
    pub kind: u32,
    pub choice: u32,
    pub number: f64,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Om9OptionItem {
    pub start: u32,
    pub length: u32,
    pub enabled: u32,
    pub selected: u32,
}
struct Entry {
    session: OptionSession,
    candidate: Option<(u64, Candidate)>,
    message: String,
    action: String,
    sealed: bool,
    curve_generation: Option<u64>,
    curve_revision: Option<u64>,
    solid_generation: Option<u64>,
    solid_revision: Option<u64>,
}
static REGISTRY: OnceLock<Mutex<BTreeMap<u64, Entry>>> = OnceLock::new();
static NEXT: AtomicU64 = AtomicU64::new(1);
fn registry() -> &'static Mutex<BTreeMap<u64, Entry>> {
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}
fn identity() -> u64 {
    NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .unwrap_or(0)
}
unsafe fn text(ptr: *const u8, len: usize) -> Result<String, String> {
    if len > 1024 || (len > 0 && ptr.is_null()) {
        return Err("Invalid input buffer".into());
    }
    let bytes = if len == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len) }
    };
    let s = std::str::from_utf8(bytes).map_err(|_| "Invalid UTF-8")?;
    if s.contains(['\0', '\n', '\r']) {
        return Err("Input must be one line".into());
    }
    Ok(s.into())
}
unsafe fn copy(s: &str, out: *mut u8, cap: usize) -> usize {
    if cap > 0 && !out.is_null() {
        let n = s.len().min(cap - 1);
        unsafe {
            std::ptr::copy_nonoverlapping(s.as_ptr(), out, n);
            *out.add(n) = 0;
        }
    }
    s.len() + 1
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_options_open(family: u32, mm_per_unit: f64) -> u64 {
    open_named(
        family,
        mm_per_unit,
        command_option_schemas::feature_id(family),
    )
}
/// # Safety
/// id is readable for len bytes; the feature identity is copied into ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_open_named(
    family: u32,
    scale: f64,
    id: *const u8,
    len: usize,
) -> u64 {
    let Ok(id) = (unsafe { text(id, len) }) else {
        return 0;
    };
    if id.is_empty() || id.len() > 128 {
        return 0;
    }
    open_named(family, scale, &id)
}
fn open_named(family: u32, mm_per_unit: f64, id: &str) -> u64 {
    let Some((schema, values)) = command_option_schemas::schema(family) else {
        return 0;
    };
    let h = identity();
    if h == 0 {
        return 0;
    }
    let Ok(mut session) = OptionSession::new(
        schema,
        values,
        OptionContext::new(h, mm_per_unit).feature(id),
    ) else {
        return 0;
    };
    for rule in command_option_schemas::rules(family) {
        if session.add_rule(rule).is_err() {
            return 0;
        }
    }
    let Ok(mut r) = registry().lock() else {
        return 0;
    };
    if r.len() >= 16 {
        return 0;
    }
    r.insert(
        h,
        Entry {
            session,
            candidate: None,
            message: String::new(),
            action: String::new(),
            sealed: true,
            curve_generation: None,
            curve_revision: None,
            solid_generation: None,
            solid_revision: None,
        },
    );
    h
}
/// # Safety
/// id is readable for len bytes. All schema text is copied into Rust ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_begin(id: *const u8, len: usize, scale: f64) -> u64 {
    let Ok(id) = (unsafe { text(id, len) }) else {
        return 0;
    };
    if id.is_empty() || id.len() > 128 {
        return 0;
    }
    let h = identity();
    if h == 0 {
        return 0;
    }
    let Ok(session) = OptionSession::new(
        Vec::new(),
        BTreeMap::new(),
        OptionContext::new(h, scale).feature(&id),
    ) else {
        return 0;
    };
    let Ok(mut r) = registry().lock() else {
        return 0;
    };
    if r.len() >= 16 {
        return 0;
    }
    r.insert(
        h,
        Entry {
            session,
            candidate: None,
            message: String::new(),
            action: String::new(),
            sealed: false,
            curve_generation: None,
            curve_revision: None,
            solid_generation: None,
            solid_revision: None,
        },
    );
    h
}
/// # Safety
/// Each string pointer is readable for its length. No pointers are retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_define(
    handle: u64,
    key: *const u8,
    len: usize,
    kind: u32,
    min: f64,
    max: f64,
    integer: bool,
    unit: *const u8,
    unit_len: usize,
    parent: *const u8,
    parent_len: usize,
    scope: u32,
) -> bool {
    let Ok(key) = (unsafe { text(key, len) }) else {
        return false;
    };
    let Ok(unit) = (unsafe { text(unit, unit_len) }) else {
        return false;
    };
    let Ok(parent) = (unsafe { text(parent, parent_len) }) else {
        return false;
    };
    if key.is_empty() || key.len() > 128 || !(1..=3).contains(&scope) {
        return false;
    }
    let mut spec = match kind {
        1 => OptionSpec::toggle(&key),
        2 => OptionSpec::choice(&key, Vec::new()),
        3 => OptionSpec::number(&key, min, max, &unit, integer),
        4 => OptionSpec::action(&key),
        5 => OptionSpec::native_input(&key),
        _ => return false,
    };
    if !parent.is_empty() {
        spec = spec.parent(&parent);
    }
    spec.scope = scope;
    let Ok(mut r) = registry().lock() else {
        return false;
    };
    let Some(e) = r.get_mut(&handle) else {
        return false;
    };
    if e.sealed
        || e.session.schema.len() >= 128
        || e.session
            .schema
            .iter()
            .any(|s| s.key.eq_ignore_ascii_case(&key))
    {
        return false;
    }
    e.session.schema.push(spec);
    true
}
/// # Safety
/// key and value are readable for their supplied lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_initial(
    handle: u64,
    key: *const u8,
    len: usize,
    kind: u32,
    number: f64,
    value: *const u8,
    value_len: usize,
) -> bool {
    let Ok(key) = (unsafe { text(key, len) }) else {
        return false;
    };
    let Ok(value) = (unsafe { text(value, value_len) }) else {
        return false;
    };
    let value = match kind {
        1 if number == 0. || number == 1. => OptionValue::Toggle(number == 1.),
        2 => OptionValue::Choice(value),
        3 => OptionValue::Number(number),
        _ => return false,
    };
    let Ok(mut r) = registry().lock() else {
        return false;
    };
    let Some(e) = r.get_mut(&handle) else {
        return false;
    };
    if e.sealed {
        return false;
    }
    e.session.initial_value(key, value);
    true
}
/// # Safety
/// The string inputs are readable for their lengths, bounded and copied.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_choice(
    handle: u64,
    key: *const u8,
    len: usize,
    value: *const u8,
    value_len: usize,
    child: *const u8,
    child_len: usize,
    enabled: bool,
    reason: *const u8,
    reason_len: usize,
) -> bool {
    let Ok(key) = (unsafe { text(key, len) }) else {
        return false;
    };
    let Ok(value) = (unsafe { text(value, value_len) }) else {
        return false;
    };
    let Ok(child) = (unsafe { text(child, child_len) }) else {
        return false;
    };
    let Ok(reason) = (unsafe { text(reason, reason_len) }) else {
        return false;
    };
    let Ok(mut r) = registry().lock() else {
        return false;
    };
    let Some(e) = r.get_mut(&handle) else {
        return false;
    };
    if e.sealed {
        return false;
    }
    let Some(spec) = e
        .session
        .schema
        .iter_mut()
        .find(|s| s.key == key && s.kind == OptionKind::Choice)
    else {
        return false;
    };
    if spec.choices.len() >= 128 {
        return false;
    }
    let mut c = Choice::new(&value);
    if !child.is_empty() {
        c = c.child(&child);
    }
    if !enabled {
        c = c.disabled(&reason);
    }
    spec.choices.push(c);
    true
}
/// # Safety
/// key and alias buffers are readable for their lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_alias(
    handle: u64,
    key: *const u8,
    len: usize,
    alias: *const u8,
    alias_len: usize,
) -> bool {
    let Ok(key) = (unsafe { text(key, len) }) else {
        return false;
    };
    let Ok(alias) = (unsafe { text(alias, alias_len) }) else {
        return false;
    };
    let Ok(mut r) = registry().lock() else {
        return false;
    };
    let Some(e) = r.get_mut(&handle) else {
        return false;
    };
    if e.sealed {
        return false;
    }
    let (option, choice) = key
        .split_once('=')
        .map_or((key.as_str(), None), |(key, value)| (key, Some(value)));
    let Some(spec) = e.session.schema.iter_mut().find(|s| s.key == option) else {
        return false;
    };
    if let Some(value) = choice {
        let Some(choice) = spec.choices.iter_mut().find(|c| c.value == value) else {
            return false;
        };
        if choice.aliases.len() >= 32 || alias.is_empty() {
            return false;
        }
        choice.aliases.push(alias);
        return true;
    }
    if spec.aliases.len() >= 32 || alias.is_empty() {
        return false;
    }
    spec.aliases.push(alias);
    true
}
/// # Safety
/// All strings are readable for the supplied lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_greater(
    handle: u64,
    higher: *const u8,
    higher_len: usize,
    lower: *const u8,
    lower_len: usize,
    message: *const u8,
    message_len: usize,
) -> bool {
    let Ok(higher) = (unsafe { text(higher, higher_len) }) else {
        return false;
    };
    let Ok(lower) = (unsafe { text(lower, lower_len) }) else {
        return false;
    };
    let Ok(message) = (unsafe { text(message, message_len) }) else {
        return false;
    };
    let Ok(mut r) = registry().lock() else {
        return false;
    };
    let Some(e) = r.get_mut(&handle) else {
        return false;
    };
    if e.sealed {
        return false;
    }
    e.session
        .add_rule(Constraint::GreaterThan {
            higher,
            lower,
            message,
        })
        .is_ok()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_options_seal(handle: u64) -> bool {
    let Ok(mut r) = registry().lock() else {
        return false;
    };
    let Some(e) = r.get_mut(&handle) else {
        return false;
    };
    if e.sealed {
        return false;
    }
    match e.session.seal() {
        Ok(()) => {
            e.sealed = true;
            true
        }
        Err(error) => {
            e.message = error;
            false
        }
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_options_close(handle: u64) {
    if let Ok(mut r) = registry().lock() {
        r.remove(&handle);
    }
}
/// # Safety
/// `input` is readable for `len` bytes (unless zero); non-null token is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_event(
    handle: u64,
    kind: u32,
    input: *const u8,
    len: usize,
    number: f64,
    token: *mut u64,
) -> u32 {
    if !token.is_null() {
        unsafe {
            *token = 0;
        }
    }
    let Ok(mut r) = registry().lock() else {
        return 4;
    };
    let Some(e) = r.get_mut(&handle) else {
        return 4;
    };
    if !e.sealed {
        e.message = "Register and seal the option schema first".into();
        return 4;
    }
    // A fresh input event retires the previous token, even if the new event
    // navigates, cancels, changes a draft or is rejected.
    e.candidate = None;
    let result = (|| {
        let input = unsafe { text(input, len) }?;
        let event = match kind {
            1 => OptionEvent::Activate(input),
            2 => OptionEvent::Submit(input),
            3 => {
                if number != 0. && number != 1. {
                    return Err("Invalid toggle value".into());
                }
                OptionEvent::Set(input, OptionValue::Toggle(number == 1.))
            }
            4 => {
                let (key, value) = input.split_once('=').ok_or("Choice requires key=value")?;
                OptionEvent::Set(key.into(), OptionValue::Choice(value.into()))
            }
            5 => OptionEvent::Set(input, OptionValue::Number(number)),
            6 => OptionEvent::Escape,
            7 => {
                if number != 1. && number != -1. {
                    return Err("Invalid selection direction".into());
                }
                OptionEvent::Select(number as i32)
            }
            8 => {
                e.session.set_draft(input);
                return Ok(OptionOutcome::Navigated);
            }
            9 => {
                let (key, raw) = input
                    .split_once('=')
                    .ok_or("Numeric input requires key=raw value")?;
                OptionEvent::SetText(key.into(), raw.into())
            }
            _ => return Err("Invalid option event".into()),
        };
        e.session.dispatch(event)
    })();
    e.message.clear();
    e.action.clear();
    match result {
        Ok(OptionOutcome::Unhandled) => 0,
        Ok(OptionOutcome::Navigated) => 1,
        Ok(OptionOutcome::Action(action)) => {
            e.action = action;
            3
        }
        Ok(OptionOutcome::Candidate(candidate)) => {
            let id = identity();
            if id == 0 {
                return 4;
            }
            e.candidate = Some((id, candidate));
            if !token.is_null() {
                unsafe {
                    *token = id;
                }
            }
            2
        }
        Err(message) => {
            e.message = message;
            4
        }
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_options_accept(handle: u64, token: u64, accept: bool) -> bool {
    let Ok(mut r) = registry().lock() else {
        return false;
    };
    let Some(e) = r.get_mut(&handle) else {
        return false;
    };
    if !e.candidate.as_ref().is_some_and(|(id, _)| *id == token) {
        return false;
    }
    let Some((_, candidate)) = e.candidate.take() else {
        return false;
    };
    if !accept {
        return true;
    }
    match e.session.accept(candidate) {
        Ok(()) => true,
        Err(error) => {
            e.message = error;
            false
        }
    }
}
/// # Safety
/// Non-null out is writable for cap bytes. Returns required bytes including NUL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_snapshot(handle: u64, out: *mut u8, cap: usize) -> usize {
    let Ok(r) = registry().lock() else { return 0 };
    let Some(e) = r.get(&handle) else { return 0 };
    unsafe { copy(&e.session.snapshot().text, out, cap) }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_options_item_count(handle: u64) -> usize {
    registry()
        .lock()
        .ok()
        .and_then(|r| r.get(&handle).map(|e| e.session.snapshot().items.len()))
        .unwrap_or(0)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_options_revision(handle: u64) -> u64 {
    registry()
        .lock()
        .ok()
        .and_then(|r| r.get(&handle).map(|e| e.session.revision()))
        .unwrap_or(0)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_options_change_count(handle: u64, token: u64) -> usize {
    registry()
        .lock()
        .ok()
        .and_then(|r| {
            r.get(&handle).and_then(|e| {
                e.candidate
                    .as_ref()
                    .filter(|(id, _)| *id == token)
                    .map(|(_, c)| c.changes.len())
            })
        })
        .unwrap_or(0)
}
/// # Safety
/// key and out obey the same buffer contract as value/snapshot.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_choice_name(
    handle: u64,
    key: *const u8,
    len: usize,
    index: usize,
    out: *mut u8,
    cap: usize,
) -> usize {
    let Ok(key) = (unsafe { text(key, len) }) else {
        return 0;
    };
    let Ok(r) = registry().lock() else { return 0 };
    let Some(e) = r.get(&handle) else { return 0 };
    let Some(choice) = e
        .session
        .schema
        .iter()
        .find(|s| s.key == key)
        .and_then(|s| s.choices.get(index))
    else {
        return 0;
    };
    unsafe { copy(&choice.value, out, cap) }
}
/// # Safety
/// out is writable for cap bytes; returns canonical typed assignment, not code.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_change_text(
    handle: u64,
    token: u64,
    index: usize,
    out: *mut u8,
    cap: usize,
) -> usize {
    let Ok(r) = registry().lock() else { return 0 };
    let Some(e) = r.get(&handle) else { return 0 };
    let Some((id, c)) = &e.candidate else {
        return 0;
    };
    if *id != token {
        return 0;
    }
    let Some((key, value)) = c.changes.get(index) else {
        return 0;
    };
    let suffix = if matches!(value, OptionValue::Number(_))
        && e.session
            .schema
            .iter()
            .any(|s| s.key == *key && s.unit == "mm")
    {
        "mm"
    } else {
        ""
    };
    unsafe { copy(&format!("{}={}{}", key, value.text(), suffix), out, cap) }
}
/// # Safety
/// Non-null out points to a writable Om9OptionItem.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_item(
    handle: u64,
    index: usize,
    out: *mut Om9OptionItem,
) -> bool {
    if out.is_null() {
        return false;
    }
    let Ok(r) = registry().lock() else {
        return false;
    };
    let Some(e) = r.get(&handle) else {
        return false;
    };
    let s = e.session.snapshot();
    let Some(item) = s.items.get(index) else {
        return false;
    };
    unsafe {
        *out = Om9OptionItem {
            start: item.start as u32,
            length: item.length as u32,
            enabled: u32::from(item.enabled),
            selected: u32::from(s.selected == Some(index)),
        };
    }
    true
}
/// # Safety
/// Non-null out is writable for cap bytes. field: 0 identity, 1 input, 2 reason,
/// 3 the single declared, unambiguous shortcut character (empty if absent).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_item_text(
    handle: u64,
    index: usize,
    field: u32,
    out: *mut u8,
    cap: usize,
) -> usize {
    let Ok(r) = registry().lock() else { return 0 };
    let Some(e) = r.get(&handle) else { return 0 };
    let s = e.session.snapshot();
    let Some(item) = s.items.get(index) else {
        return 0;
    };
    let value = match field {
        0 => &item.key,
        1 => &item.input,
        2 => &item.reason,
        3 => &item.mnemonic,
        _ => return 0,
    };
    unsafe { copy(value, out, cap) }
}
/// # Safety
/// out is writable for cap bytes. field: 0 error,1 action,2 pending identity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_message(
    handle: u64,
    field: u32,
    out: *mut u8,
    cap: usize,
) -> usize {
    let Ok(r) = registry().lock() else { return 0 };
    let Some(e) = r.get(&handle) else { return 0 };
    let value = match field {
        0 => e.message.as_str(),
        1 => e.action.as_str(),
        2 => e.session.pending_key().unwrap_or(""),
        3 => e.session.feature_id(),
        _ => return 0,
    };
    unsafe { copy(value, out, cap) }
}
/// # Safety
/// key is readable for len bytes; non-null out is a writable Om9OptionValue.
/// token=0 reads accepted values; nonzero reads only the current candidate.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_value(
    handle: u64,
    key: *const u8,
    len: usize,
    token: u64,
    out: *mut Om9OptionValue,
) -> bool {
    if out.is_null() {
        return false;
    }
    let Ok(key) = (unsafe { text(key, len) }) else {
        return false;
    };
    let Ok(r) = registry().lock() else {
        return false;
    };
    let Some(e) = r.get(&handle) else {
        return false;
    };
    let values = if token == 0 {
        e.session.values()
    } else {
        let Some((id, c)) = &e.candidate else {
            return false;
        };
        if *id != token {
            return false;
        }
        &c.values
    };
    let Some(value) = values.get(&key) else {
        return false;
    };
    let v = match value {
        OptionValue::Toggle(b) => Om9OptionValue {
            kind: 1,
            choice: 0,
            number: if *b { 1. } else { 0. },
        },
        OptionValue::Number(n) => Om9OptionValue {
            kind: 3,
            choice: 0,
            number: *n,
        },
        OptionValue::Choice(value) => {
            let Some(s) = e.session.schema.iter().find(|s| s.key == key) else {
                return false;
            };
            let Some(i) = s.choices.iter().position(|c| c.value == *value) else {
                return false;
            };
            Om9OptionValue {
                kind: 2,
                choice: i as u32,
                number: 0.,
            }
        }
    };
    unsafe {
        *out = v;
    }
    true
}
/// # Safety
/// key/reason are readable for the supplied lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_gate(
    handle: u64,
    key: *const u8,
    len: usize,
    enabled: bool,
    reason: *const u8,
    reason_len: usize,
) -> bool {
    let Ok(key) = (unsafe { text(key, len) }) else {
        return false;
    };
    let Ok(reason) = (unsafe { text(reason, reason_len) }) else {
        return false;
    };
    let Ok(mut r) = registry().lock() else {
        return false;
    };
    let Some(e) = r.get_mut(&handle) else {
        return false;
    };
    e.session.set_gate(&key, enabled, &reason).is_ok()
}
/// # Safety
/// key is readable for len bytes; no pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_visible(handle: u64, key: *const u8, len: usize) -> bool {
    let Ok(key) = (unsafe { text(key, len) }) else {
        return false;
    };
    let Ok(r) = registry().lock() else {
        return false;
    };
    let Some(e) = r.get(&handle) else {
        return false;
    };
    e.session
        .schema
        .iter()
        .find(|s| s.key == key)
        .is_some_and(|s| s.enabled && e.session.visible(s))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_options_sync_curve(handle: u64) -> bool {
    let Some((id, values, gates, generation, revision)) = crate::curve_ffi::command_option_state()
    else {
        return false;
    };
    let Ok(mut r) = registry().lock() else {
        return false;
    };
    let Some(e) = r.get_mut(&handle) else {
        return false;
    };
    if e.curve_generation
        .is_some_and(|previous| previous != generation)
        || e.session.feature_id() != id
        || e.session.synchronize(values).is_err()
    {
        return false;
    }
    e.curve_generation = Some(generation);
    if e.curve_revision
        .is_some_and(|previous| previous != revision)
        && e.session.context_changed().is_err()
    {
        return false;
    }
    e.curve_revision = Some(revision);
    for (key, enabled, reason) in gates {
        if e.session.schema.iter().any(|s| s.key == key) {
            let _ = e.session.set_gate(key, enabled, reason);
        }
    }
    true
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_options_sync_solid(handle: u64) -> bool {
    let scale = {
        let Ok(r) = registry().lock() else {
            return false;
        };
        let Some(e) = r.get(&handle) else {
            return false;
        };
        e.session.unit_scale()
    };
    let Some((generation, revision, model)) = crate::solid::command_option_state(scale) else {
        return false;
    };
    let Ok(mut r) = registry().lock() else {
        return false;
    };
    let Some(e) = r.get_mut(&handle) else {
        return false;
    };
    if e.session.feature_id() != model.feature_id()
        || e.solid_generation.is_some_and(|g| g != generation)
    {
        return false;
    }
    if e.solid_generation.is_none() {
        e.session = model;
    } else if e.solid_revision != Some(revision) && e.session.refresh_presentation(model).is_err() {
        return false;
    }
    if e.solid_revision != Some(revision) {
        e.candidate = None;
        e.action.clear();
    }
    e.solid_generation = Some(generation);
    e.solid_revision = Some(revision);
    true
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_options_question_length(handle: u64) -> usize {
    registry()
        .lock()
        .ok()
        .and_then(|r| r.get(&handle).map(|e| e.session.snapshot().question_length))
        .unwrap_or(0)
}
/// # Safety
/// name is readable for len bytes; non-null out is writable for cap bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_invocation_line(
    name: *const u8,
    len: usize,
    out: *mut u8,
    cap: usize,
) -> usize {
    let Ok(name) = (unsafe { text(name, len) }) else {
        return 0;
    };
    let Ok(line) = command_invocation_line(&name) else {
        return 0;
    };
    unsafe { copy(&line, out, cap) }
}
/// # Safety
/// question is readable for len bytes and is copied into owned state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_question(
    handle: u64,
    question: *const u8,
    len: usize,
) -> bool {
    let Ok(question) = (unsafe { text(question, len) }) else {
        return false;
    };
    let Ok(mut r) = registry().lock() else {
        return false;
    };
    r.get_mut(&handle)
        .is_some_and(|e| e.session.set_question(&question).is_ok())
}
/// # Safety
/// key is readable for len bytes and no pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_options_shown(
    handle: u64,
    key: *const u8,
    len: usize,
    shown: bool,
) -> bool {
    let Ok(key) = (unsafe { text(key, len) }) else {
        return false;
    };
    let Ok(mut r) = registry().lock() else {
        return false;
    };
    r.get_mut(&handle)
        .is_some_and(|e| e.session.set_shown(&key, shown).is_ok())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_options_accept_curve(handle: u64, token: u64) -> u32 {
    let Ok(mut r) = registry().lock() else {
        return 0;
    };
    let Some(e) = r.get_mut(&handle) else {
        return 0;
    };
    let Some((serial, candidate)) = e.candidate.as_ref() else {
        return 0;
    };
    if *serial != token {
        return 0;
    }
    let candidate = candidate.clone();
    let changes = candidate.changes.clone();
    let feature = e.session.feature_id().to_owned();
    match crate::curve_ffi::apply_command_options(
        &feature,
        e.curve_generation.unwrap_or(0),
        e.curve_revision.unwrap_or(u64::MAX),
        &changes,
        || e.session.accept(candidate),
    ) {
        Ok(effect) => {
            e.candidate = None;
            effect
        }
        Err(error) => {
            e.message = error;
            0
        }
    }
}
