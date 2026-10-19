# OM9-L3-01-03-007 — Group Membership Model (typed API)

Owner module `OM9-L2-01-03` (document scene entities, stage `E01`, gate **AUTO_ONLY**).
Full module contract: `implementation/design/M01/OM9-L2-01-03/api.md` §7.1.
Implementation owner: `crates/om9-core/src/{model,transaction}.rs`. Reuse decision: **ADD the group record and its two mutations; REUSE the identity ledger**.

Oracle — positive: *Group rồi ungroup không đổi hình học và ID; xoá thành viên cuối thì group tự mất.*
Negative: *Thành viên không tồn tại/lặp/rỗng bị từ chối; GroupId đã nghỉ không được cấp lại.*

## 1. Typed API

```rust
pub struct Group { pub id: GroupId, pub name: String, pub members: Vec<ObjectId>, pub revision: u64 }
impl Group { pub fn new(name, members) -> Self; pub fn contains(&self, id: ObjectId) -> bool; }
impl Document { pub fn group(&self, id) -> Result<&Group>; pub fn groups_of(&self, id: ObjectId) -> Vec<GroupId>;
                pub(crate) fn prune_membership(&mut self, removed: ObjectId) -> Result<()>; pub groups: BTreeMap<GroupId, Group>; }
Mutation::{GroupCreate, GroupDelete} { document, expected_revision, .. }
```

## 2. Errors (exact strings)

- `Missing("group member")        — thành viên không tồn tại`
- `Invalid("duplicate group member") — cùng object hai lần trong một group`
- `Invalid("empty group")          — group không có thành viên`
- `Invalid("allocated group UUID")  — GroupId đã nghỉ bị cấp lại`
- `Missing("group")               — xoá group không tồn tại`

## 3. Tests that carry the oracle

- `cargo test -p om9-core --test document_graph l3_007 -> 5 passed`

## 4. Evidence

- `implementation/evidence/M01/OM9-L3-01-03-007/OM9-L4-01-03-007-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-03-007/OM9-L4-01-03-007-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-03-007/OM9-L4-01-03-007-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-03-007/positive.json`, `negative.json`
