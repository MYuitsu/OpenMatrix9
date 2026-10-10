//! Source archive identity and preservation policy. No host or parser dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ArchiveMode {
    GeometryOnly = 0,
    Preserve = 1,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ArchiveCapability {
    Editable = 1,
    DisplayRetained = 2,
    Retained = 3,
    Incompatible = 4,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArchiveIdentity {
    pub import_namespace: String,
    pub source_uuid: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentityError;
fn canonical_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 36
        && bytes.iter().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                *b == b'-'
            } else {
                b.is_ascii_digit() || (b'a'..=b'f').contains(b)
            }
        })
        && bytes.iter().any(|b| *b != b'0' && *b != b'-')
}
impl ArchiveIdentity {
    pub fn new(import_namespace: &str, source_uuid: &str) -> Result<Self, IdentityError> {
        if !canonical_uuid(import_namespace) || !canonical_uuid(source_uuid) {
            return Err(IdentityError);
        }
        Ok(Self {
            import_namespace: import_namespace.into(),
            source_uuid: source_uuid.into(),
        })
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_3dm_archive_mode_valid(mode: u32) -> bool {
    mode <= 1
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_3dm_archive_capability_valid(capability: u32) -> bool {
    (1..=4).contains(&capability)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_3dm_archive_legacy_export_allowed(mode: u32) -> bool {
    mode == ArchiveMode::GeometryOnly as u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClosureError {
    Malformed,
    Cycle,
    Limit,
}
pub const MAX_ARCHIVE_NODES: usize = 1_000_000;
pub const MAX_ARCHIVE_EDGES: usize = 16_000_000;
pub const MAX_COPY_BYTES: usize = 512 * 1024 * 1024;
#[unsafe(no_mangle)]
pub extern "C" fn om9_3dm_copy_budget(bytes: usize, copies: usize, total: usize) -> isize {
    match bytes
        .checked_mul(copies)
        .and_then(|value| total.checked_add(value))
    {
        Some(value) if value <= MAX_COPY_BYTES => value as isize,
        _ => -1,
    }
}
// actions: unchanged=0, replace geometry=1, transform instance=2, duplicate=3,
// native non-instance geometry transform=4.
#[unsafe(no_mangle)]
pub extern "C" fn om9_3dm_overlay_allowed(
    capability: u32,
    action: u32,
    known_references: bool,
    is_instance: bool,
) -> bool {
    if !known_references || !om9_3dm_archive_capability_valid(capability) || capability == 4 {
        return false;
    }
    match action {
        0 | 3 => true,
        1 => capability == 1,
        2 => is_instance,
        4 => !is_instance,
        _ => false,
    }
}
pub fn dependency_closure(
    node_count: usize,
    offsets: &[usize],
    edges: &[usize],
    selected: &[usize],
) -> Result<Vec<usize>, ClosureError> {
    if node_count > MAX_ARCHIVE_NODES
        || edges.len() > MAX_ARCHIVE_EDGES
        || selected.len() > MAX_ARCHIVE_NODES
    {
        return Err(ClosureError::Limit);
    }
    if offsets.len() != node_count + 1
        || offsets.first() != Some(&0)
        || offsets.last() != Some(&edges.len())
        || offsets.windows(2).any(|p| p[0] > p[1])
        || edges.iter().any(|i| *i >= node_count)
        || selected.iter().any(|i| *i >= node_count)
    {
        return Err(ClosureError::Malformed);
    }
    let mut state = vec![0u8; node_count];
    let mut stack = Vec::new();
    for &root in selected {
        if state[root] == 2 {
            continue;
        }
        state[root] = 1;
        stack.push((root, offsets[root]));
        while let Some((node, next)) = stack.last_mut() {
            if *next == offsets[*node + 1] {
                state[*node] = 2;
                stack.pop();
                continue;
            }
            let child = edges[*next];
            *next += 1;
            match state[child] {
                1 => return Err(ClosureError::Cycle),
                0 => {
                    state[child] = 1;
                    stack.push((child, offsets[child]));
                }
                _ => {}
            }
        }
    }
    Ok(state
        .iter()
        .enumerate()
        .filter_map(|(i, s)| (*s == 2).then_some(i))
        .collect())
}

/// # Safety
/// Nonempty pointer ranges must be aligned and readable for the declared lengths.
/// Output must be writable for capacity entries and must not overlap input ranges.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_3dm_dependency_closure(
    node_count: usize,
    offsets: *const usize,
    edges: *const usize,
    edge_count: usize,
    selected: *const usize,
    selected_count: usize,
    output: *mut usize,
    capacity: usize,
) -> isize {
    if node_count > MAX_ARCHIVE_NODES
        || edge_count > MAX_ARCHIVE_EDGES
        || selected_count > MAX_ARCHIVE_NODES
    {
        return -3;
    }
    if offsets.is_null()
        || (edge_count > 0 && edges.is_null())
        || (selected_count > 0 && selected.is_null())
        || (capacity > 0 && output.is_null())
    {
        return -1;
    }
    let offsets = unsafe { std::slice::from_raw_parts(offsets, node_count + 1) };
    let edges = if edge_count == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(edges, edge_count) }
    };
    let selected = if selected_count == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(selected, selected_count) }
    };
    match dependency_closure(node_count, offsets, edges, selected) {
        Err(ClosureError::Malformed) => -1,
        Err(ClosureError::Cycle) => -2,
        Err(ClosureError::Limit) => -3,
        Ok(result) => {
            if capacity == 0 {
                return result.len() as isize;
            }
            if capacity < result.len() {
                return -4;
            }
            if !result.is_empty() {
                unsafe { std::ptr::copy_nonoverlapping(result.as_ptr(), output, result.len()) };
            }
            result.len() as isize
        }
    }
}
