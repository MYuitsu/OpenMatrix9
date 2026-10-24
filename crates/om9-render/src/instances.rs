//! CPU-side instance table shared by the GPU instanced renderer.
//!
//! One record per occurrence; geometry is referenced by mesh index, never copied. Sorting by mesh
//! turns the table into contiguous draw ranges, which is what `multi_draw_indexed_indirect` needs.

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRecord {
    pub mesh: u32,
    pub flags: u32,
    /// Row-major model matrix, camera-relative f32.
    pub model: [[f32; 4]; 4],
    pub tint: [f32; 4],
}

#[derive(Default)]
pub struct InstanceTable {
    records: Vec<InstanceRecord>,
}

impl InstanceTable {
    pub fn push(&mut self, record: InstanceRecord) {
        self.records.push(record);
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn records(&self) -> &[InstanceRecord] {
        &self.records
    }

    /// Updates one occurrence in place (preview/gumball/selection), avoiding a table rebuild.
    /// Indirect draw arguments per mesh range: `[index_count, instance_count, first_index,
    /// base_vertex, first_instance]`, ready for `multi_draw_indexed_indirect`.
    pub fn indirect_args(&mut self, index_counts: &[u32]) -> Vec<[u32; 5]> {
        self.grouped_ranges()
            .into_iter()
            .filter_map(|(mesh, first_instance, instance_count)| {
                let index_count = *index_counts.get(mesh as usize)?;
                Some([index_count, instance_count, 0, 0, first_instance])
            })
            .collect()
    }

    pub fn set_model(&mut self, index: usize, model: [[f32; 4]; 4]) -> bool {
        match self.records.get_mut(index) {
            Some(record) => {
                record.model = model;
                true
            }
            None => false,
        }
    }

    pub fn set_flags(&mut self, index: usize, flags: u32) -> bool {
        match self.records.get_mut(index) {
            Some(record) => {
                record.flags = flags;
                true
            }
            None => false,
        }
    }

    /// Sorts by mesh and returns `(mesh, first_instance, instance_count)` ranges.
    pub fn grouped_ranges(&mut self) -> Vec<(u32, u32, u32)> {
        self.records.sort_unstable_by_key(|record| record.mesh);
        let mut ranges = Vec::new();
        let mut start = 0usize;
        while start < self.records.len() {
            let mesh = self.records[start].mesh;
            let mut end = start + 1;
            while end < self.records.len() && self.records[end].mesh == mesh {
                end += 1;
            }
            ranges.push((mesh, start as u32, (end - start) as u32));
            start = end;
        }
        ranges
    }
}

/// Instance records uploaded once as a storage buffer for the GPU cull/vertex stages.
pub struct GpuInstanceBuffer {
    pub buffer: wgpu::Buffer,
    pub count: u32,
}

impl GpuInstanceBuffer {
    pub fn upload(device: &wgpu::Device, table: &InstanceTable) -> Self {
        use wgpu::util::DeviceExt;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance records"),
            contents: bytemuck::cast_slice(table.records()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            buffer,
            count: table.len() as u32,
        }
    }

    /// Updates a single occurrence (preview/selection) without re-uploading the table.
    pub fn write_one(&self, queue: &wgpu::Queue, index: u32, record: InstanceRecord) {
        if index >= self.count {
            return;
        }
        queue.write_buffer(
            &self.buffer,
            index as u64 * std::mem::size_of::<InstanceRecord>() as u64,
            bytemuck::bytes_of(&record),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(mesh: u32) -> InstanceRecord {
        InstanceRecord {
            mesh,
            flags: 0,
            model: [[0.0; 4]; 4],
            tint: [1.0; 4],
        }
    }

    #[test]
    fn instances_group_into_contiguous_mesh_ranges() {
        let mut table = InstanceTable::default();
        for mesh in [2, 1, 2, 3, 1] {
            table.push(record(mesh));
        }
        let ranges = table.grouped_ranges();
        assert_eq!(ranges, vec![(1, 0, 2), (2, 2, 2), (3, 4, 1)]);
        assert_eq!(table.len(), 5);
    }

    #[test]
    fn one_occurrence_can_be_updated_without_rebuilding() {
        let mut table = InstanceTable::default();
        table.push(record(1));
        let model = [[2.0; 4]; 4];
        assert!(table.set_model(0, model));
        assert!(table.set_flags(0, 9));
        assert_eq!(table.records()[0].model, model);
        assert_eq!(table.records()[0].flags, 9);
        assert!(!table.set_model(5, model));
    }

    #[test]
    fn indirect_args_follow_the_mesh_ranges() {
        let mut table = InstanceTable::default();
        for mesh in [1, 2, 1] {
            table.push(record(mesh));
        }
        let args = table.indirect_args(&[0, 36, 12]);
        assert_eq!(args, vec![[36, 2, 0, 0, 0], [12, 1, 0, 0, 2]]);
    }

    #[test]
    fn a_27k_table_becomes_full_mesh_ranges() {
        let mut table = InstanceTable::default();
        for index in 0..27_000u32 {
            table.push(record(index % 42));
        }
        let index_counts = vec![36u32; 42];
        let args = table.indirect_args(&index_counts);
        assert_eq!(args.len(), 42);
        assert_eq!(args.iter().map(|arg| arg[1]).sum::<u32>(), 27_000);
        assert_eq!(args[0][4], 0);
    }
}
