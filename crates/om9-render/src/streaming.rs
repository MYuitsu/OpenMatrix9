//! Cluster streaming: keep VRAM under a budget instead of uploading a whole archive.
//!
//! A 2 GB 3DM file cannot be resident. The rule a streamer needs is simple and testable: clusters are
//! uploaded when they are requested, the least recently used ones are evicted when the budget would be
//! exceeded, and a cluster larger than the whole budget is refused instead of thrashing.
//!
//! This is the bookkeeping half; the GPU buffers are allocated from the same numbers, so the accounting is
//! what decides whether the scene fits.

use std::collections::HashMap;

/// What the streamer refuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamError {
    /// The cluster alone is bigger than the whole budget.
    TooLarge { bytes: u64, budget: u64 },
    /// The request is empty or the budget is zero.
    Invalid,
}

/// Counters a caller can show in a status line.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StreamStats {
    pub uploads: u64,
    pub evictions: u64,
    pub hits: u64,
    pub resident: usize,
    pub resident_bytes: u64,
    pub budget_bytes: u64,
    /// Bytes refused because a single cluster did not fit.
    pub refused: u64,
}

#[derive(Debug, Clone, Copy)]
struct Resident {
    bytes: u64,
    last_used: u64,
}

/// LRU cluster residency under a byte budget.
pub struct ClusterStream {
    budget_bytes: u64,
    resident: HashMap<u32, Resident>,
    resident_bytes: u64,
    clock: u64,
    stats: StreamStats,
}

impl ClusterStream {
    pub fn new(budget_bytes: u64) -> Self {
        Self {
            budget_bytes,
            resident: HashMap::new(),
            resident_bytes: 0,
            clock: 0,
            stats: StreamStats {
                budget_bytes,
                ..StreamStats::default()
            },
        }
    }

    /// Makes a cluster resident, evicting the least recently used ones as needed.
    ///
    /// Returns `Ok(true)` when the cluster had to be uploaded, `Ok(false)` when it was already resident.
    pub fn request(&mut self, id: u32, bytes: u64) -> Result<bool, StreamError> {
        if bytes == 0 || self.budget_bytes == 0 {
            return Err(StreamError::Invalid);
        }
        if bytes > self.budget_bytes {
            self.stats.refused += bytes;
            return Err(StreamError::TooLarge {
                bytes,
                budget: self.budget_bytes,
            });
        }
        self.clock += 1;
        if let Some(entry) = self.resident.get_mut(&id) {
            entry.last_used = self.clock;
            self.stats.hits += 1;
            return Ok(false);
        }
        while self.resident_bytes + bytes > self.budget_bytes {
            let Some(victim) = self
                .resident
                .iter()
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(id, _)| *id)
            else {
                break;
            };
            if let Some(entry) = self.resident.remove(&victim) {
                self.resident_bytes -= entry.bytes;
                self.stats.evictions += 1;
            }
        }
        self.resident.insert(
            id,
            Resident {
                bytes,
                last_used: self.clock,
            },
        );
        self.resident_bytes += bytes;
        self.stats.uploads += 1;
        self.stats.resident = self.resident.len();
        self.stats.resident_bytes = self.resident_bytes;
        Ok(true)
    }

    /// Records a use without changing residency, for clusters already drawn this frame.
    pub fn touch(&mut self, id: u32) {
        self.clock += 1;
        if let Some(entry) = self.resident.get_mut(&id) {
            entry.last_used = self.clock;
        }
    }

    pub fn is_resident(&self, id: u32) -> bool {
        self.resident.contains_key(&id)
    }

    pub fn resident_bytes(&self) -> u64 {
        self.resident_bytes
    }

    pub fn resident_count(&self) -> usize {
        self.resident.len()
    }

    pub fn stats(&self) -> StreamStats {
        StreamStats {
            resident: self.resident.len(),
            resident_bytes: self.resident_bytes,
            ..self.stats
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_budget_is_never_exceeded() {
        let mut stream = ClusterStream::new(300);
        for id in 0..5u32 {
            stream.request(id, 100).expect("fits");
        }
        assert!(stream.resident_bytes() <= 300);
        assert_eq!(
            stream.resident_count(),
            3,
            "only three hundred-byte clusters fit"
        );
        assert_eq!(stream.stats().evictions, 2);
        assert_eq!(stream.stats().uploads, 5);
    }

    #[test]
    fn a_repeated_request_is_a_cache_hit() {
        let mut stream = ClusterStream::new(1000);
        assert!(stream.request(7, 100).expect("upload"));
        assert!(!stream.request(7, 100).expect("hit"));
        assert_eq!(stream.stats().uploads, 1);
        assert_eq!(stream.stats().hits, 1);
    }

    #[test]
    fn a_cluster_larger_than_the_budget_is_refused() {
        let mut stream = ClusterStream::new(100);
        let error = stream.request(1, 400).expect_err("must refuse");
        assert_eq!(
            error,
            StreamError::TooLarge {
                bytes: 400,
                budget: 100
            }
        );
        assert_eq!(stream.resident_count(), 0);
        assert!(stream.stats().refused >= 400);
    }

    #[test]
    fn a_two_gigabyte_archive_never_needs_two_gigabytes_of_vram() {
        // 2 GiB of clusters at 30 KiB each, streamed through a 512 MiB budget.
        let total_clusters = (2u64 * 1024 * 1024 * 1024) / (30 * 1024);
        let budget = 512 * 1024 * 1024;
        let mut stream = ClusterStream::new(budget);
        for id in 0..total_clusters {
            stream
                .request(id as u32, 30 * 1024)
                .expect("every cluster fits alone");
        }
        let stats = stream.stats();
        assert!(
            stats.resident_bytes <= budget,
            "resident {} > budget {budget}",
            stats.resident_bytes
        );
        assert_eq!(
            stats.uploads, total_clusters,
            "each cluster uploaded once in a linear walk"
        );
        assert!(stats.evictions > 0, "the walk must evict older clusters");
        assert!(
            stats.resident_bytes < 2 * 1024 * 1024 * 1024 / 4,
            "VRAM stays far below the archive size"
        );
    }

    #[test]
    fn touching_a_cluster_keeps_it_resident() {
        let mut stream = ClusterStream::new(200);
        stream.request(1, 100).expect("a");
        stream.request(2, 100).expect("b");
        stream.touch(1);
        stream.request(3, 100).expect("c");
        assert!(stream.is_resident(1), "the touched cluster must survive");
        assert!(!stream.is_resident(2), "the untouched one is evicted");
    }
}
