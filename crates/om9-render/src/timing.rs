// SPDX-License-Identifier: LGPL-2.1-or-later
//! Nonblocking, optional GPU timestamps. A sample identifies the submitted OM9 frame, not presentation.
use std::sync::mpsc;

pub struct GpuTimer {
    queries: wgpu::QuerySet,
    resolved: wgpu::Buffer,
    readback: wgpu::Buffer,
    pending: Option<(u64, mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>)>,
}
impl GpuTimer {
    pub fn new(device: &wgpu::Device) -> Option<Self> {
        if !device.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
            return None;
        }
        Some(Self {
            queries: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("OM9 frame timestamps"),
                ty: wgpu::QueryType::Timestamp,
                count: 2,
            }),
            resolved: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("OM9 timestamp resolve"),
                size: 16,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }),
            readback: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("OM9 timestamp readback"),
                size: 16,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            pending: None,
        })
    }
    pub fn available(&self) -> bool {
        self.pending.is_none()
    }
    /// Attachment-only timestamp markers require only TIMESTAMP_QUERY, including adapters without
    /// TIMESTAMP_QUERY_INSIDE_ENCODERS. They bracket every OM9 pass in this viewport.
    pub fn marker(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        end: bool,
    ) {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("OM9 timestamp marker"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: Some(wgpu::RenderPassTimestampWrites {
                query_set: &self.queries,
                beginning_of_pass_write_index: (!end).then_some(0),
                end_of_pass_write_index: end.then_some(1),
            }),
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    pub fn resolve(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.resolve_query_set(&self.queries, 0..2, &self.resolved, 0);
        encoder.copy_buffer_to_buffer(&self.resolved, 0, &self.readback, 0, 16);
    }
    /// Call only after the encoder containing resolve() has been submitted.
    pub fn submitted(&mut self, frame: u64) {
        let (tx, rx) = mpsc::channel();
        self.readback
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = tx.send(result);
            });
        self.pending = Some((frame, rx));
    }
    pub fn poll(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> Option<(u64, f64)> {
        let _ = device.poll(wgpu::PollType::Poll);
        let (frame, rx) = self.pending.as_ref()?;
        let result = match rx.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.pending = None;
                return None;
            }
            Err(_) => return None,
        };
        let frame = *frame;
        self.pending = None;
        if result.is_err() {
            return None;
        }
        let bytes = self.readback.slice(..).get_mapped_range().ok()?;
        let start = u64::from_le_bytes(bytes[0..8].try_into().ok()?);
        let end = u64::from_le_bytes(bytes[8..16].try_into().ok()?);
        drop(bytes);
        self.readback.unmap();
        Some((
            frame,
            end.saturating_sub(start) as f64 * queue.get_timestamp_period() as f64 / 1e6,
        ))
    }
}
