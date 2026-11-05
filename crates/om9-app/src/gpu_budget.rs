// SPDX-License-Identifier: LGPL-2.1-or-later
//! Conservative budget preview. It is diagnostic only until all GPU owners
//! are accounted and complete over-budget drawing is implemented.

const FALLBACK_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Backing {
    Dedicated,
    SharedSystem,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BudgetSample {
    pub backing: Backing,
    /// Dedicated capacity must come from the actual selected adapter.
    pub dedicated_bytes: Option<u64>,
    /// OS budget and current usage are a point-in-time observation.
    pub os_budget_bytes: Option<u64>,
    pub os_usage_bytes: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct BudgetPreview {
    pub backing: Backing,
    pub hardware_ceiling_verified: bool,
    pub ram_accounting_required: Option<bool>,
    pub hardware_or_fallback_bytes: u64,
    pub os_ceiling_after_headroom_bytes: Option<u64>,
    pub os_remaining_after_headroom_bytes: Option<u64>,
    pub configured_ceiling_bytes: u64,
    /// A preview is not a reservation or permission to allocate.
    pub enforced: bool,
}

impl BudgetPreview {
    /// Necessary per-allocation lower bound, not a GPU reservation. Existing
    /// OS usage includes this process, so remaining headroom can be smaller
    /// than the configured aggregate ceiling during resize or external load.
    pub fn target_admission_ceiling(self) -> u64 {
        self.configured_ceiling_bytes.min(self.os_remaining_after_headroom_bytes.unwrap_or(u64::MAX))
    }
}

pub fn preview(sample: BudgetSample, lower_limit_bytes: Option<u64>) -> Option<BudgetPreview> {
    if lower_limit_bytes == Some(0) { return None; }
    let capacity = match (sample.backing, sample.dedicated_bytes) {
        (Backing::Dedicated, Some(bytes)) if bytes > 0 => Some(bytes),
        _ => None,
    };
    let hardware_or_fallback_bytes = capacity.map_or(FALLBACK_BYTES, |bytes| bytes / 2);
    let os_ceiling_after_headroom_bytes = sample.os_budget_bytes
        .map(|bytes| (u128::from(bytes) * 9 / 10) as u64);
    let os_remaining_after_headroom_bytes = os_ceiling_after_headroom_bytes
        .zip(sample.os_usage_bytes).map(|(ceiling, used)| ceiling.saturating_sub(used));
    let configured_ceiling_bytes = hardware_or_fallback_bytes
        .min(os_ceiling_after_headroom_bytes.unwrap_or(u64::MAX))
        .min(lower_limit_bytes.unwrap_or(u64::MAX));
    Some(BudgetPreview {
        backing: sample.backing,
        hardware_ceiling_verified: capacity.is_some(),
        ram_accounting_required: match sample.backing {
            Backing::Dedicated => Some(false), Backing::SharedSystem => Some(true), Backing::Unknown => None,
        },
        hardware_or_fallback_bytes,
        os_ceiling_after_headroom_bytes,
        os_remaining_after_headroom_bytes,
        configured_ceiling_bytes,
        enforced: false,
    })
}

pub fn unverified_preview(device_type: eframe::wgpu::DeviceType) -> BudgetPreview {
    let backing = match device_type {
        eframe::wgpu::DeviceType::IntegratedGpu => Backing::SharedSystem,
        _ => Backing::Unknown,
    };
    preview(BudgetSample { backing, dedicated_bytes: None, os_budget_bytes: None, os_usage_bytes: None }, None)
        .expect("automatic preview has no invalid manual limit")
}

/// A single Vulkan local heap supplies an OS budget, but device-local and
/// non-host-visible flags do not prove dedicated VRAM. Retain the unverified
/// hardware fallback; multiple local heaps are ambiguous and not summed.
#[cfg(windows)]
pub fn preview_vulkan(heaps: &[om9_native_owner::gpu_memory::VulkanHeapSample], device_type: eframe::wgpu::DeviceType) -> BudgetPreview {
    let mut local = heaps.iter().filter(|heap| heap.device_local);
    let first = local.next();
    if let (Some(heap), None) = (first, local.next()) {
        if heap.size_bytes > 0 {
            let backing = match device_type {
                eframe::wgpu::DeviceType::IntegratedGpu => Backing::SharedSystem,
                _ => Backing::Unknown,
            };
            return preview(BudgetSample { backing, dedicated_bytes: None,
                os_budget_bytes: Some(heap.budget_bytes), os_usage_bytes: Some(heap.usage_bytes) }, None)
                .expect("automatic preview has no invalid manual limit");
        }
    }
    unverified_preview(device_type)
}

/// Query only the adapter/device selected by wgpu, retaining the unverified
/// fallback for unsupported or ambiguous backend memory information.
pub fn selected_adapter_preview(adapter: &eframe::wgpu::Adapter, device: &eframe::wgpu::Device) -> BudgetPreview {
    let kind = adapter.get_info().device_type;
    #[cfg(windows)]
    if let Some(sample) = om9_native_owner::gpu_memory::dx12_adapter_memory(adapter) {
        return preview_dxgi(sample, kind);
    }
    #[cfg(windows)]
    if let Some(sample) = om9_native_owner::gpu_memory::vulkan_dxgi_adapter_memory(adapter, device) {
        return preview_dxgi(sample, kind);
    }
    #[cfg(windows)]
    if let Some(heaps) = om9_native_owner::gpu_memory::vulkan_device_memory(adapter, device) {
        return preview_vulkan(&heaps, kind);
    }
    #[cfg(not(windows))]
    let _ = device;
    unverified_preview(kind)
}

/// Use only a sample from the exact DXGI adapter selected by wgpu. Device
/// type is the adapter's reported class, not an independent UMA certificate.
#[cfg(windows)]
pub fn preview_dxgi(sample: om9_native_owner::gpu_memory::DxgiMemorySample, device_type: eframe::wgpu::DeviceType)
    -> BudgetPreview {
    let backing = match device_type {
        eframe::wgpu::DeviceType::DiscreteGpu if sample.dedicated_video_bytes > 0 => Backing::Dedicated,
        eframe::wgpu::DeviceType::IntegratedGpu => Backing::SharedSystem,
        _ => Backing::Unknown,
    };
    preview(BudgetSample { backing, dedicated_bytes: Some(sample.dedicated_video_bytes),
        os_budget_bytes: Some(sample.local_budget_bytes), os_usage_bytes: Some(sample.local_usage_bytes) }, None)
        .expect("automatic preview has no invalid manual limit")
}

#[cfg(test)]
mod tests {
    use super::*;
    const MIB: u64 = 1024 * 1024;

    #[test]
    fn dedicated_capacity_os_headroom_and_lower_limit_are_independent() {
        let sample = BudgetSample { backing: Backing::Dedicated, dedicated_bytes: Some(8 * 1024 * MIB),
            os_budget_bytes: Some(5 * 1024 * MIB), os_usage_bytes: Some(4 * 1024 * MIB) };
        let auto = preview(sample, None).unwrap();
        assert_eq!(auto.hardware_or_fallback_bytes, 4 * 1024 * MIB);
        assert_eq!(auto.os_ceiling_after_headroom_bytes, Some(4608 * MIB));
        assert_eq!(auto.os_remaining_after_headroom_bytes, Some(512 * MIB));
        assert_eq!(auto.configured_ceiling_bytes, 4 * 1024 * MIB);
        assert_eq!(auto.target_admission_ceiling(), 512 * MIB);
        assert!(auto.hardware_ceiling_verified && !auto.enforced);
        assert_eq!(serde_json::to_value(auto).unwrap()["enforced"], false);
        assert_eq!(preview(sample, Some(1024 * MIB)).unwrap().configured_ceiling_bytes, 1024 * MIB);
        assert!(preview(sample, Some(0)).is_none());
    }

    #[test]
    fn shared_unknown_and_pressure_are_not_claimed_as_verified_vram() {
        let shared = preview(BudgetSample { backing: Backing::SharedSystem, dedicated_bytes: Some(4 * MIB),
            os_budget_bytes: Some(100 * MIB), os_usage_bytes: Some(99 * MIB) }, None).unwrap();
        assert_eq!(shared.hardware_or_fallback_bytes, 256 * MIB);
        assert_eq!(shared.configured_ceiling_bytes, 90 * MIB);
        assert_eq!(shared.os_remaining_after_headroom_bytes, Some(0));
        assert_eq!(shared.target_admission_ceiling(), 0);
        assert_eq!(shared.ram_accounting_required, Some(true));
        assert!(!shared.hardware_ceiling_verified);
        let unknown = preview(BudgetSample { backing: Backing::Unknown, dedicated_bytes: None,
            os_budget_bytes: None, os_usage_bytes: None }, None).unwrap();
        assert_eq!(unknown.configured_ceiling_bytes, 256 * MIB);
        assert_eq!(unknown.os_remaining_after_headroom_bytes, None);
        assert_eq!(unknown.target_admission_ceiling(), 256 * MIB);
        assert!(!unknown.hardware_ceiling_verified);
        assert_eq!(unknown.ram_accounting_required, None);
    }

    #[test]
    fn arithmetic_handles_maximum_and_zero_budget() {
        let sample = BudgetSample { backing: Backing::Dedicated, dedicated_bytes: Some(u64::MAX),
            os_budget_bytes: Some(u64::MAX), os_usage_bytes: Some(u64::MAX) };
        let result = preview(sample, None).unwrap();
        assert_eq!(result.hardware_or_fallback_bytes, u64::MAX / 2);
        assert_eq!(result.os_remaining_after_headroom_bytes, Some(0));
        assert_eq!(preview(BudgetSample { os_budget_bytes: Some(0), ..sample }, None).unwrap().configured_ceiling_bytes, 0);
    }

    #[test]
    #[cfg(windows)]
    fn vulkan_uses_one_local_heap_budget_without_claiming_dedicated_memory() {
        use eframe::wgpu::DeviceType;
        use om9_native_owner::gpu_memory::VulkanHeapSample;
        let heap = VulkanHeapSample { size_bytes: 8 * 1024 * MIB, budget_bytes: 3 * 1024 * MIB,
            usage_bytes: MIB, device_local: true, host_visible: false };
        let dedicated = preview_vulkan(&[heap], DeviceType::DiscreteGpu);
        assert!(!dedicated.hardware_ceiling_verified);
        assert_eq!(dedicated.hardware_or_fallback_bytes, 256 * MIB);
        assert_eq!(dedicated.os_ceiling_after_headroom_bytes, Some((3 * 1024 * MIB * 9) / 10));
        for ambiguous in [vec![], vec![heap, heap]] {
            let result = preview_vulkan(&ambiguous, DeviceType::DiscreteGpu);
            assert!(!result.hardware_ceiling_verified);
            assert_eq!(result.hardware_or_fallback_bytes, 256 * MIB);
        }
        let integrated = preview_vulkan(&[heap], DeviceType::IntegratedGpu);
        assert!(!integrated.hardware_ceiling_verified);
        assert_eq!(integrated.ram_accounting_required, Some(true));
        let tight = VulkanHeapSample { budget_bytes: 100 * MIB, host_visible: true, ..heap };
        assert_eq!(preview_vulkan(&[tight], DeviceType::IntegratedGpu).configured_ceiling_bytes, 90 * MIB);
    }

    #[cfg(windows)]
    #[test]
    fn dxgi_classification_uses_adapter_type_without_guessing_uma_from_memory_size() {
        let sample = om9_native_owner::gpu_memory::DxgiMemorySample {
            luid_low: 1, luid_high: 0, vendor_id: 2, device_id: 3,
            dedicated_video_bytes: 4 * 1024 * MIB, shared_system_bytes: 8 * 1024 * MIB,
            local_budget_bytes: 3 * 1024 * MIB, local_usage_bytes: MIB,
            nonlocal_budget_bytes: 2 * 1024 * MIB, nonlocal_usage_bytes: 0,
        };
        let discrete = preview_dxgi(sample, eframe::wgpu::DeviceType::DiscreteGpu);
        assert_eq!(discrete.configured_ceiling_bytes, 2 * 1024 * MIB);
        assert!(discrete.hardware_ceiling_verified);
        let integrated = preview_dxgi(sample, eframe::wgpu::DeviceType::IntegratedGpu);
        assert_eq!(integrated.hardware_or_fallback_bytes, 256 * MIB);
        assert_eq!(integrated.ram_accounting_required, Some(true));
        assert!(!integrated.hardware_ceiling_verified);
        let unknown = unverified_preview(eframe::wgpu::DeviceType::DiscreteGpu);
        assert_eq!(unknown.configured_ceiling_bytes, 256 * MIB);
        assert_eq!(unknown.ram_accounting_required, None);
    }
}
