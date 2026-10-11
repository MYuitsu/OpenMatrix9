// SPDX-License-Identifier: LGPL-2.1-or-later
//! GUI-thread/value-only bridge. Pointers must be aligned, valid for the call,
//! nonoverlapping and readable/writable for their declared length. No pointer
//! is retained. Lengths/enum/UTF-8 admission precede reads and session mutation.
use crate::{core_objects::Stamp, gumball::*};
use std::{
    collections::HashMap,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Mutex, OnceLock},
};
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GumballSource {
    pub document: u64,
    pub object: u64,
    pub generation: u64,
    pub revision: u64,
    pub epoch: u64,
    pub parent: u64,
    pub world: [f64; 16],
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GumballBegin {
    pub document: u64,
    pub view_generation: u64,
    pub unit_generation: u64,
    pub mm_per_unit: f64,
    pub frame: [f64; 12],
    pub cplane: [f64; 12],
    pub ray_origin: Point,
    pub ray_direction: Point,
    pub handle: u32,
    pub modifiers: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GumballEvent {
    pub kind: u32,
    pub key: u32,
    pub pressed: u32,
    pub auto_repeat: u32,
    pub ray_origin: Point,
    pub ray_direction: Point,
    pub world_point: Point,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GumballSnapshot {
    pub enabled: u32,
    pub phase: u32,
    pub alignment: u32,
    pub handle: u32,
    pub operation: u32,
    pub copy: u32,
    pub source_count: u32,
    pub constrained: u32,
    pub generation: u64,
    pub frame: [f64; 12],
    pub matrix: [f64; 16],
    pub constraint: f64,
}
struct Registry {
    next: Option<u64>,
    sessions: HashMap<u64, GumballSession>,
}
static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
fn registry() -> &'static Mutex<Registry> {
    REGISTRY.get_or_init(|| {
        Mutex::new(Registry {
            next: Some(1),
            sessions: HashMap::new(),
        })
    })
}
fn call(f: impl FnOnce() -> u32) -> u32 {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(1)
}
/// # Safety
/// Text is readable for length bytes, both outputs writable for one u32.
/// 0 recognized, 1 another command, 2 invalid; failure leaves outputs untouched.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_gumball_command(
    text: *const u8,
    length: usize,
    kind_out: *mut u32,
    value_out: *mut u32,
) -> u32 {
    call(|| {
        if text.is_null() || length > 1024 || kind_out.is_null() || value_out.is_null() {
            return 2;
        }
        let Ok(raw) = std::str::from_utf8(unsafe { std::slice::from_raw_parts(text, length) })
        else {
            return 2;
        };
        match parse_command(raw) {
            Ok(Some(command)) => {
                unsafe {
                    *kind_out = 1 + command.kind as u32;
                    *value_out = command.value.unwrap_or(u32::MAX);
                }
                0
            }
            Ok(None) => 1,
            Err(_) => 2,
        }
    })
}
fn update(id: u64, event: Event) -> u32 {
    let Ok(mut r) = registry().lock() else {
        return 1;
    };
    let Some(s) = r.sessions.get_mut(&id) else {
        return 1;
    };
    match s.apply(event) {
        Ok(_) => 0,
        Err(_) => 2,
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_gumball_report(id:u64,out:*mut u8,capacity:usize,needed:*mut usize)->u32 {
    call(||{
        if needed.is_null()||capacity>4*1024*1024||out.is_null()&&capacity!=0{return 1;}
        let Ok(registry)=registry().lock() else{return 1;};let Some(session)=registry.sessions.get(&id) else{return 1;};
        let Ok(report)=session.report() else{return 2;};
        if out.is_null(){unsafe{*needed=report.len()};return 0;}
        if capacity<report.len(){return 1;}
        unsafe{std::ptr::copy_nonoverlapping(report.as_ptr(),out,report.len());*needed=report.len();}0
    })
}
fn frame(raw: [f64; 12]) -> Result<Frame, GumballError> {
    Frame::new(
        [raw[0], raw[1], raw[2]],
        [
            [raw[3], raw[4], raw[5]],
            [raw[6], raw[7], raw[8]],
            [raw[9], raw[10], raw[11]],
        ],
    )
}
fn packed(f: Frame) -> [f64; 12] {
    [
        f.origin[0],
        f.origin[1],
        f.origin[2],
        f.axes[0][0],
        f.axes[0][1],
        f.axes[0][2],
        f.axes[1][0],
        f.axes[1][1],
        f.axes[1][2],
        f.axes[2][0],
        f.axes[2][1],
        f.axes[2][2],
    ]
}
/// # Safety
/// Both inputs are readable for 12 doubles, satisfying the pointer contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_gumball_frame(id: u64, f: *const f64, cp: *const f64) -> u32 {
    call(|| {
        if f.is_null() || cp.is_null() {
            return 1;
        }
        let raw = unsafe { std::slice::from_raw_parts(f, 12) };
        let plane = unsafe { std::slice::from_raw_parts(cp, 12) };
        let Ok(f) = frame(raw.try_into().unwrap()) else {
            return 2;
        };
        let Ok(cp) = frame(plane.try_into().unwrap()) else {
            return 2;
        };
        update(id, Event::ObserveFrame(f, cp))
    })
}
/// # Safety
/// Source is null for transient/multiple selection or readable for one record.
/// Both frames are readable for twelve doubles; nothing borrowed is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_gumball_owner_frame(id:u64,source:*const GumballSource,f:*const f64,cp:*const f64)->u32 {
    call(|| {
        if f.is_null()||cp.is_null(){return 1;}
        let Ok(f)=frame(unsafe{std::slice::from_raw_parts(f,12)}.try_into().unwrap()) else{return 2;};
        let Ok(cp)=frame(unsafe{std::slice::from_raw_parts(cp,12)}.try_into().unwrap()) else{return 2;};
        let owner=if source.is_null(){None}else{Some(Source::from(unsafe{*source}).stamp)};
        update(id,Event::ObserveOwner(owner,f,cp))
    })
}
/// # Safety
/// Inputs are readable for 3 doubles, output writable for 12, nonoverlapping.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_gumball_frame_from_directions(
    o: *const f64,
    x: *const f64,
    n: *const f64,
    out: *mut f64,
) -> u32 {
    call(|| {
        if [o, x, n].iter().any(|p| p.is_null()) || out.is_null() {
            return 1;
        }
        let read = |p| unsafe { <[f64; 3]>::try_from(std::slice::from_raw_parts(p, 3)).unwrap() };
        let Ok(f) = Frame::from_directions(read(o), read(x), read(n)) else {
            return 2;
        };
        unsafe {
            std::ptr::copy_nonoverlapping(packed(f).as_ptr(), out, 12);
        }
        0
    })
}
impl From<GumballSource> for Source {
    fn from(s: GumballSource) -> Self {
        Self {
            stamp: Stamp {
                document: s.document,
                object: s.object,
                generation: s.generation,
                revision: s.revision,
                epoch: s.epoch,
            },
            parent: s.parent,
            world: s.world,
        }
    }
}
impl From<Source> for GumballSource {
    fn from(s: Source) -> Self {
        Self {
            document: s.stamp.document,
            object: s.stamp.object,
            generation: s.stamp.generation,
            revision: s.stamp.revision,
            epoch: s.stamp.epoch,
            parent: s.parent,
            world: s.world,
        }
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_gumball_create() -> u64 {
    catch_unwind(|| {
        let Ok(mut r) = registry().lock() else {
            return 0;
        };
        if r.sessions.len() >= 256 {
            return 0;
        }
        let Some(id) = r.next else {
            return 0;
        };
        r.next = id.checked_add(1);
        r.sessions.insert(id, GumballSession::new());
        id
    })
    .unwrap_or(0)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_gumball_destroy(id: u64) -> u32 {
    call(|| {
        let Ok(mut r) = registry().lock() else {
            return 1;
        };
        if r.sessions.remove(&id).is_some() {
            0
        } else {
            1
        }
    })
}
/// # Safety
/// Input records and source array satisfy the module pointer contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_gumball_begin(
    id: u64,
    input: *const GumballBegin,
    sources: *const GumballSource,
    count: usize,
) -> u32 {
    call(|| {
        if input.is_null()
            || sources.is_null()
            || count == 0
            || count > crate::native_input_tree::MAX_NODES
        {
            return 1;
        }
        let b = unsafe { *input };
        let Some(handle) = Handle::from_code(b.handle) else {
            return 1;
        };
        let Ok(f) = frame(b.frame) else {
            return 2;
        };
        let Ok(cp) = frame(b.cplane) else {
            return 2;
        };
        let sources = unsafe { std::slice::from_raw_parts(sources, count) }
            .iter()
            .copied()
            .map(Source::from)
            .collect();
        update(
            id,
            Event::Begin(BeginContext {
                frame: f,
                cplane: cp,
                sources,
                document: b.document,
                view_generation: b.view_generation,
                unit_generation: b.unit_generation,
                mm_per_unit: b.mm_per_unit,
                handle,
                start_ray: PointerRay {
                    origin: b.ray_origin,
                    direction: b.ray_direction,
                },
                modifiers: b.modifiers,
            }),
        )
    })
}
/// # Safety
/// Event is a readable record satisfying the module pointer contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_gumball_event(id: u64, input: *const GumballEvent) -> u32 {
    call(|| {
        if input.is_null() {
            return 1;
        }
        let e = unsafe { *input };
        let event = match e.kind {
            0 => {
                if e.pressed > 1 {
                    return 1;
                }
                Event::Enable(e.pressed == 1)
            }
            1 => Event::Align(match e.key {
                0 => Alignment::CPlane,
                1 => Alignment::Object,
                2 => Alignment::World,
                _ => return 1,
            }),
            2 => Event::Motion(PointerRay {
                origin: e.ray_origin,
                direction: e.ray_direction,
            }),
            3 => {
                if e.pressed > 1 || e.auto_repeat > 1 {
                    return 1;
                }
                Event::Modifier(ModifierEvent {
                    key: match e.key {
                        1 => ModifierKey::Shift,
                        2 => ModifierKey::Ctrl,
                        4 => ModifierKey::Alt,
                        _ => return 1,
                    },
                    pressed: e.pressed == 1,
                    auto_repeat: e.auto_repeat == 1,
                })
            }
            4 => Event::Accept,
            5 => Event::Release,
            6 => Event::Undo,
            7 => Event::Cancel,
            8 => Event::BeginRelocate,
            9 => Event::RelocatePoint(e.world_point),
            10 => Event::CommitSucceeded,
            12 => Event::RelocateRay(PointerRay {origin:e.ray_origin,direction:e.ray_direction}),
            _ => return 1,
        };
        update(id, event)
    })
}
/// # Safety
/// Text is readable for length bytes and satisfies the module pointer contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_gumball_input(id: u64, text: *const u8, length: usize) -> u32 {
    call(|| {
        if text.is_null() || length == 0 || length > 1024 {
            return 1;
        }
        let Ok(raw) = std::str::from_utf8(unsafe { std::slice::from_raw_parts(text, length) })
        else {
            return 1;
        };
        update(id, Event::Numeric(raw.to_owned()))
    })
}
/// # Safety
/// Output is writable for one record and satisfies the module pointer contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_gumball_snapshot(id: u64, out: *mut GumballSnapshot) -> u32 {
    call(|| {
        if out.is_null() {
            return 1;
        }
        let Ok(r) = registry().lock() else {
            return 1;
        };
        let Some(s) = r.sessions.get(&id) else {
            return 1;
        };
        let snap = s.snapshot();
        let p = snap.preview.as_ref();
        let value = GumballSnapshot {
            enabled: snap.enabled as u32,
            phase: snap.phase as u32,
            alignment: snap.alignment as u32,
            handle: snap.handle.map_or(u32::MAX, Handle::code),
            operation: p.map_or(0, |p| p.operation as u32),
            copy: snap.copy as u32,
            source_count: p.map_or(0, |p| p.sources.len() as u32),
            constrained: snap.constraint.is_some() as u32,
            generation: snap.generation,
            frame: packed(snap.frame),
            matrix: p.map_or(crate::geometry_frame::IDENTITY, |p| p.matrix),
            constraint: snap.constraint.unwrap_or(0.),
        };
        unsafe {
            *out = value;
        }
        0
    })
}
/// # Safety
/// Output is writable for capacity records, written for one usize; nonoverlapping.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_gumball_sources(
    id: u64,
    out: *mut GumballSource,
    capacity: usize,
    written: *mut usize,
) -> u32 {
    call(|| {
        if written.is_null()
            || capacity > crate::native_input_tree::MAX_NODES
            || (capacity != 0 && out.is_null())
        {
            return 1;
        }
        let Ok(r) = registry().lock() else {
            return 1;
        };
        let Some(s) = r.sessions.get(&id) else {
            return 1;
        };
        let Ok(p) = s.plan() else {
            return 2;
        };
        if capacity != 0 && capacity < p.sources.len() {
            return 1;
        }
        if capacity != 0 {
            for (i, source) in p.sources.into_iter().enumerate() {
                unsafe {
                    out.add(i).write(source.into());
                }
            }
            unsafe {
                *written = s.snapshot().preview.map_or(0, |p| p.sources.len());
            }
        } else {
            unsafe {
                *written = p.sources.len();
            }
        }
        0
    })
}
