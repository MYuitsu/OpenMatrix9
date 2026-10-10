#![forbid(unsafe_code)]
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
pub type SnapError = String;
pub type ObjectToken = u64;
pub type Point = [f64; 3];
pub type ScreenPoint = [f64; 3];
#[derive(Clone, Debug, PartialEq)]
pub struct ViewKey {
    pub document: u64,
    pub view: u64,
    pub generation: u64,
    pub width: u32,
    pub height: u32,
    pub camera: [f64; 16],
}
#[derive(Clone, Copy, Debug)]
pub struct Row {
    pub object: ObjectToken,
    pub bounds: [f64; 4],
}
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub objects: usize,
    pub per_object: usize,
    pub total: usize,
}
impl Limits {
    fn validate(&self) -> Result<(), SnapError> {
        if !(1..=64).contains(&self.objects)
            || !(1..=2048).contains(&self.per_object)
            || !(1..=8192).contains(&self.total)
        {
            Err("Invalid snap query budget".into())
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Debug)]
struct Candidates {
    points: Vec<(Point, ScreenPoint)>,
    visited: usize,
}
type CacheKey = (ObjectToken, u32, usize);
struct Build {
    key: ViewKey,
    rows: Vec<Row>,
    tokens: HashSet<u64>,
    tiles: HashMap<(i32, i32), Vec<usize>>,
    entries: usize,
    failed: bool,
}
#[derive(Default)]
pub struct SnapIndex {
    current: Option<Build>,
    staged: Option<Build>,
    epoch: u64,
    cache: HashMap<CacheKey, Arc<Candidates>>,
    cache_keys: HashMap<ObjectToken, HashSet<CacheKey>>,
    cached_points: usize,
}
#[derive(Clone, Debug, Default)]
pub struct SnapResult {
    pub complete: bool,
    pub picked: bool,
    pub point: Point,
    pub points: Vec<Point>,
    pub nearby_objects: usize,
    pub visited_objects: usize,
    pub visited_topology: usize,
    pub generated: usize,
    pub max_per_object: usize,
}
pub struct SnapQuery {
    epoch: u64,
    objects: Vec<ObjectToken>,
    modes: u32,
    limits: Limits,
    cursor: [f64; 2],
    best: f64,
    counts: HashMap<u64, usize>,
    consumed: HashSet<(u64, u32)>,
    cache: HashMap<CacheKey, Arc<Candidates>>,
    new_cache: HashMap<CacheKey, Arc<Candidates>>,
    result: SnapResult,
    pub generated: usize,
}
impl SnapIndex {
    pub fn invalidate(&mut self) {
        self.current = None;
        self.staged = None;
        self.cache.clear();
        self.cache_keys.clear();
        self.cached_points = 0;
        self.epoch = self.epoch.saturating_add(1);
    }
    pub fn begin_build(&mut self, key: ViewKey) -> Result<(), SnapError> {
        self.invalidate();
        if key.document == 0
            || key.view == 0
            || key.width == 0
            || key.height == 0
            || key.width > 32768
            || key.height > 32768
            || key.camera.iter().any(|n| !n.is_finite())
        {
            return Err("Invalid native view snapshot".into());
        }
        if self.epoch == u64::MAX {
            return Err("Snap generation exhausted".into());
        }
        self.staged = Some(Build {
            key,
            rows: Vec::new(),
            tokens: HashSet::new(),
            tiles: HashMap::new(),
            entries: 0,
            failed: false,
        });
        Ok(())
    }
    pub fn add_row(&mut self, row: Row) -> Result<(), SnapError> {
        let build = self.staged.as_mut().ok_or("No staged snap index")?;
        if build.failed {
            return Err("Staged snap index already failed".into());
        }
        build.failed = true;
        if row.object == 0
            || row.bounds.iter().any(|x| !x.is_finite() || x.abs() > 1e9)
            || row.bounds[0] > row.bounds[2]
            || row.bounds[1] > row.bounds[3]
            || build.rows.len() >= 1_000_000
            || build.tokens.contains(&row.object)
        {
            return Err("Invalid or over-budget native snap row".into());
        }
        let [x0, y0, x1, y1] = row.bounds;
        let w = f64::from(build.key.width);
        let h = f64::from(build.key.height);
        if x1 < -64. || y1 < -64. || x0 > w + 64. || y0 > h + 64. {
            build.failed = false;
            return Ok(());
        }
        let left = ((x0 - 64.).clamp(0., w) / 64.).floor() as i32;
        let right = ((x1 + 64.).clamp(0., w) / 64.).floor() as i32;
        let top = ((y0 - 64.).clamp(0., h) / 64.).floor() as i32;
        let bottom = ((y1 + 64.).clamp(0., h) / 64.).floor() as i32;
        let extra = (right - left + 1) as usize * (bottom - top + 1) as usize;
        if extra > 2_000_000usize.saturating_sub(build.entries) {
            return Err("Snap tile allocation budget exhausted".into());
        }
        let index = build.rows.len();
        build.rows.push(row);
        build.tokens.insert(row.object);
        build.entries += extra;
        for y in top..=bottom {
            for x in left..=right {
                build.tiles.entry((x, y)).or_default().push(index);
            }
        }
        build.failed = false;
        Ok(())
    }
    pub fn finish_build(&mut self) -> Result<(), SnapError> {
        let build = self.staged.take().ok_or("No complete staged snap index")?;
        if build.failed {
            return Err("Cannot publish failed staged snap index".into());
        }
        self.current = Some(build);
        Ok(())
    }
    pub fn abort_build(&mut self) {
        self.invalidate();
    }
    pub fn matches(&self, key: &ViewKey) -> bool {
        self.current.as_ref().is_some_and(|b| b.key == *key)
    }
    pub fn near_objects(
        &self,
        cursor: [f64; 2],
        radius: f64,
        limits: Limits,
    ) -> Result<Vec<ObjectToken>, SnapError> {
        limits.validate()?;
        if cursor.iter().any(|n| !n.is_finite() || n.abs() > 1e9)
            || !radius.is_finite()
            || radius <= 0.
            || radius > 64.
        {
            return Err("Invalid snap cursor/radius".into());
        }
        let build = self.current.as_ref().ok_or("Snap index incomplete")?;
        let tile = (
            (cursor[0] / 64.).floor().max(0.) as i32,
            (cursor[1] / 64.).floor().max(0.) as i32,
        );
        let mut result = Vec::new();
        if let Some(rows) = build.tiles.get(&tile) {
            for index in rows {
                let row = &build.rows[*index];
                let [x0, y0, x1, y1] = row.bounds;
                if cursor[0] + radius < x0
                    || cursor[0] - radius > x1
                    || cursor[1] + radius < y0
                    || cursor[1] - radius > y1
                {
                    continue;
                }
                if result.len() == limits.objects {
                    return Err("Snap object budget exhausted".into());
                }
                result.push(row.object);
            }
        }
        Ok(result)
    }
    pub fn start_query(
        &mut self,
        cursor: [f64; 2],
        radius: f64,
        modes: u32,
        limits: Limits,
    ) -> Result<SnapQuery, SnapError> {
        if modes & 14 == 0 || modes & !14 != 0 {
            return Err("Invalid native CAD snap modes".into());
        }
        let objects = self.near_objects(cursor, radius, limits)?;
        // Lookup only nearby objects and share immutable, independently owned data.
        let cache = objects
            .iter()
            .filter_map(|object| self.cache_keys.get(object))
            .flat_map(|keys| keys.iter())
            .filter(|key| key.1 & modes != 0)
            .filter_map(|key| self.cache.get(key).map(|value| (*key, Arc::clone(value))))
            .collect();
        Ok(SnapQuery {
            epoch: self.epoch,
            objects: objects.clone(),
            modes,
            limits,
            cursor,
            best: radius * radius,
            counts: HashMap::new(),
            consumed: HashSet::new(),
            cache,
            new_cache: HashMap::new(),
            result: SnapResult {
                complete: true,
                nearby_objects: objects.len(),
                ..Default::default()
            },
            generated: 0,
        })
    }
    pub fn publish_query(&mut self, query: SnapQuery) -> Result<SnapResult, SnapError> {
        if self.current.is_none() || query.epoch != self.epoch {
            return Err("Snap query generation changed".into());
        }
        if query.is_complete() {
            for (key, value) in &query.new_cache {
                let count = value.points.len();
                if self.cache.len() >= 4096 || self.cached_points + count > 131072 {
                    self.cache.clear();
                    self.cache_keys.clear();
                    self.cached_points = 0;
                }
                if let Some(old) = self.cache.insert(*key, value.clone()) {
                    self.cached_points -= old.points.len();
                }
                self.cached_points += count;
                self.cache_keys.entry(key.0).or_default().insert(*key);
            }
        }
        Ok(query.finish())
    }
}
impl SnapQuery {
    pub fn objects(&self) -> &[ObjectToken] {
        &self.objects
    }
    pub fn remaining_budget(&self, object: ObjectToken) -> usize {
        if !self.result.complete || !self.objects.contains(&object) {
            return 0;
        }
        self.limits
            .per_object
            .saturating_sub(*self.counts.get(&object).unwrap_or(&0))
            .min(self.limits.total.saturating_sub(self.result.points.len()))
    }
    pub fn cached(&mut self, object: ObjectToken, mode: u32) -> Result<bool, SnapError> {
        let key = (object, mode, self.remaining_budget(object));
        if let Some(value) = self.cache.get(&key).cloned() {
            self.apply(object, mode, &value.points, true, value.visited, false)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    pub fn consume(
        &mut self,
        object: ObjectToken,
        mode: u32,
        points: &[(Point, ScreenPoint)],
        complete: bool,
        visited: usize,
    ) -> Result<(), SnapError> {
        self.apply(object, mode, points, complete, visited, true)
    }
    fn apply(
        &mut self,
        object: ObjectToken,
        mode: u32,
        points: &[(Point, ScreenPoint)],
        complete: bool,
        visited: usize,
        new: bool,
    ) -> Result<(), SnapError> {
        let budget = self.remaining_budget(object);
        let invalid = !self.result.complete
            || !self.objects.contains(&object)
            || ![2, 4, 8].contains(&mode)
            || mode & self.modes == 0
            || self.consumed.contains(&(object, mode))
            || points.len() > budget
            || visited > 1_000_001
            || points
                .iter()
                .any(|(world, screen)| world.iter().chain(screen).any(|n| !n.is_finite()));
        if invalid {
            self.abort();
            return Err("Invalid, repeated or over-budget snap candidates".into());
        }
        self.consumed.insert((object, mode));
        self.result.visited_topology = self.result.visited_topology.saturating_add(visited);
        let count = self.counts.entry(object).or_insert(0);
        if *count == 0 {
            self.result.visited_objects = self
                .consumed
                .iter()
                .map(|p| p.0)
                .collect::<HashSet<_>>()
                .len();
        }
        if new {
            self.generated += points.len();
        }
        if !complete {
            self.abort();
            return Ok(());
        }
        *count += points.len();
        self.result.max_per_object = self.result.max_per_object.max(*count);
        if new {
            self.new_cache.insert(
                (object, mode, budget),
                Arc::new(Candidates {
                    points: points.to_vec(),
                    visited,
                }),
            );
        }
        for (world, screen) in points {
            self.result.points.push(*world);
            if screen[2] < 0. || screen[2] > 1. {
                continue;
            }
            let dx = screen[0] - self.cursor[0];
            let dy = screen[1] - self.cursor[1];
            let distance = dx * dx + dy * dy;
            if distance <= self.best {
                self.best = distance;
                self.result.point = *world;
                self.result.picked = true;
            }
        }
        Ok(())
    }
    pub fn abort(&mut self) {
        self.result.complete = false;
        self.result.picked = false;
    }
    fn is_complete(&self) -> bool {
        self.result.complete
            && self.consumed.len() == self.objects.len() * self.modes.count_ones() as usize
    }
    pub fn finish(mut self) -> SnapResult {
        if !self.is_complete() {
            self.abort();
        }
        self.result.generated = self.generated;
        self.result
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn query_retains_the_same_immutable_candidate_allocation() {
        let mut index = SnapIndex::default();
        index
            .begin_build(ViewKey {
                document: 1,
                view: 2,
                generation: 3,
                width: 1000,
                height: 800,
                camera: [0.; 16],
            })
            .unwrap();
        index
            .add_row(Row {
                object: 1,
                bounds: [0., 0., 10., 10.],
            })
            .unwrap();
        index.finish_build().unwrap();
        let limits = Limits {
            objects: 64,
            per_object: 2048,
            total: 8192,
        };
        let mut cold = index.start_query([0., 0.], 8., 2, limits).unwrap();
        cold.consume(1, 2, &[([1., 0., 0.], [1., 0., 0.5])], true, 1)
            .unwrap();
        index.publish_query(cold).unwrap();
        let key = (1, 2, 2048);
        let original = index.cache[&key].points.as_ptr();
        let mut warm = index.start_query([0., 0.], 8., 2, limits).unwrap();
        assert_eq!(
            warm.cache[&key].points.as_ptr(),
            original,
            "warm query must share immutable candidates instead of allocating copies"
        );
        assert!(warm.cached(1, 2).unwrap());
        assert_eq!(warm.cache[&key].points.as_ptr(), original);
        // Ownership must survive index invalidation, while epoch prevents publishing.
        index.invalidate();
        assert_eq!(warm.cache[&key].points[0].0, [1., 0., 0.]);
        assert!(index.publish_query(warm).is_err());
    }
}
