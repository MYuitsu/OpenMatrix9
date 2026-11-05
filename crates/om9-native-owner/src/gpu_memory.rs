// SPDX-License-Identifier: LGPL-2.1-or-later
//! DXGI memory information from the exact DX12 adapter selected by wgpu.

/// A point-in-time OS sample. The budget can change as Windows reallocates it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DxgiMemorySample {
    pub luid_low: u32,
    pub luid_high: i32,
    pub vendor_id: u32,
    pub device_id: u32,
    pub dedicated_video_bytes: u64,
    pub shared_system_bytes: u64,
    pub local_budget_bytes: u64,
    pub local_usage_bytes: u64,
    pub nonlocal_budget_bytes: u64,
    pub nonlocal_usage_bytes: u64,
}

/// Vulkan budget for one physical memory heap; flags are raw capabilities,
/// not a promise that the heap is exclusively VRAM or system RAM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VulkanHeapSample {
    pub size_bytes: u64,
    pub budget_bytes: u64,
    pub usage_bytes: u64,
    pub device_local: bool,
    pub host_visible: bool,
}

/// Query budgets only when the actual wgpu Vulkan device enabled the extension.
/// Returns `None` for mismatched handles, unsupported backends or no extension.
pub fn vulkan_device_memory(adapter: &wgpu::Adapter, device: &wgpu::Device) -> Option<Vec<VulkanHeapSample>> {
    use ash::{ext, khr, vk};
    // SAFETY: Both HAL guards keep the wgpu objects alive. Handles are compared
    // before use; all calls query memory properties without mutating GPU state.
    unsafe {
        let a = adapter.as_hal::<wgpu::hal::api::Vulkan>()?;
        let d = device.as_hal::<wgpu::hal::api::Vulkan>()?;
        let instance = a.shared_instance();
        if a.raw_physical_device() != d.raw_physical_device()
            || instance.raw_instance().handle() != d.shared_instance().raw_instance().handle()
            || !d.enabled_device_extensions().contains(&ext::memory_budget::NAME)
        { return None; }
        let mut budget = vk::PhysicalDeviceMemoryBudgetPropertiesEXT::default();
        let mut props = vk::PhysicalDeviceMemoryProperties2::default().push_next(&mut budget);
        if instance.instance_api_version() >= vk::API_VERSION_1_1 {
            instance.raw_instance().get_physical_device_memory_properties2(a.raw_physical_device(), &mut props);
        } else if instance.extensions().contains(&khr::get_physical_device_properties2::NAME) {
            let extension = khr::get_physical_device_properties2::Instance::new(instance.entry(), instance.raw_instance());
            extension.get_physical_device_memory_properties2(a.raw_physical_device(), &mut props);
        } else { return None; }
        let memory = props.memory_properties;
        let heap_count = usize::try_from(memory.memory_heap_count).ok()?;
        let type_count = usize::try_from(memory.memory_type_count).ok()?;
        if heap_count > memory.memory_heaps.len() || type_count > memory.memory_types.len() { return None; }
        let mut heaps = Vec::with_capacity(heap_count);
        for i in 0..heap_count {
            let heap = memory.memory_heaps[i];
            let host_visible = memory.memory_types[..type_count].iter().any(|ty| ty.heap_index as usize == i && ty.property_flags.contains(vk::MemoryPropertyFlags::HOST_VISIBLE));
            heaps.push(VulkanHeapSample {
                size_bytes: heap.size,
                budget_bytes: budget.heap_budget[i],
                usage_bytes: budget.heap_usage[i],
                device_local: heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL),
                host_visible,
            });
        }
        Some(heaps)
    }
}

/// Returns `None` for another backend or a failed DXGI query. This does not
/// classify UMA hardware or impose a memory limit.
pub fn dx12_adapter_memory(adapter: &wgpu::Adapter) -> Option<DxgiMemorySample> {
    // SAFETY: The HAL guard keeps the exact DX12 adapter alive throughout the query.
    unsafe {
        let hal = adapter.as_hal::<wgpu::hal::api::Dx12>()?;
        dxgi_memory_sample(hal.as_raw())
    }
}

fn dxgi_memory_sample(raw: &windows::Win32::Graphics::Dxgi::IDXGIAdapter3) -> Option<DxgiMemorySample> {
    use windows::Win32::Graphics::Dxgi::{DXGI_MEMORY_SEGMENT_GROUP_LOCAL, DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL, DXGI_QUERY_VIDEO_MEMORY_INFO};
    // SAFETY: `raw` is a live COM adapter. Queries retain no pointers or resources.
    unsafe {
        let description = raw.GetDesc1().ok()?;
        let mut local = DXGI_QUERY_VIDEO_MEMORY_INFO::default();
        let mut nonlocal = DXGI_QUERY_VIDEO_MEMORY_INFO::default();
        raw.QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, &mut local).ok()?;
        raw.QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL, &mut nonlocal).ok()?;
        Some(DxgiMemorySample {
            luid_low: description.AdapterLuid.LowPart,
            luid_high: description.AdapterLuid.HighPart,
            vendor_id: description.VendorId,
            device_id: description.DeviceId,
            dedicated_video_bytes: description.DedicatedVideoMemory as u64,
            shared_system_bytes: description.SharedSystemMemory as u64,
            local_budget_bytes: local.Budget,
            local_usage_bytes: local.CurrentUsage,
            nonlocal_budget_bytes: nonlocal.Budget,
            nonlocal_usage_bytes: nonlocal.CurrentUsage,
        })
    }
}

/// Resolve the exact Vulkan physical device to DXGI using its Windows LUID.
/// Returns None when LUID, a single node, matching IDs or a query is unavailable.
pub fn vulkan_dxgi_adapter_memory(adapter: &wgpu::Adapter, device: &wgpu::Device) -> Option<DxgiMemorySample> {
    use ash::{khr, vk};
    use windows::Win32::{Foundation::LUID, Graphics::Dxgi::{CreateDXGIFactory1, IDXGIAdapter3, IDXGIFactory4}};
    // SAFETY: HAL guards keep the exact adapter, device and instance alive;
    // Vulkan and DXGI calls only query properties and a matching COM adapter.
    unsafe {
        let a = adapter.as_hal::<wgpu::hal::api::Vulkan>()?;
        let d = device.as_hal::<wgpu::hal::api::Vulkan>()?;
        let instance = a.shared_instance();
        if a.raw_physical_device() != d.raw_physical_device()
            || instance.raw_instance().handle() != d.shared_instance().raw_instance().handle()
        { return None; }
        let mut identity = vk::PhysicalDeviceIDProperties::default();
        let mut properties = vk::PhysicalDeviceProperties2::default().push_next(&mut identity);
        if instance.instance_api_version() >= vk::API_VERSION_1_1 {
            instance.raw_instance().get_physical_device_properties2(a.raw_physical_device(), &mut properties);
        } else if instance.extensions().contains(&khr::get_physical_device_properties2::NAME)
            && instance.extensions().contains(&khr::external_memory_capabilities::NAME) {
            let ext = khr::get_physical_device_properties2::Instance::new(instance.entry(), instance.raw_instance());
            ext.get_physical_device_properties2(a.raw_physical_device(), &mut properties);
        } else { return None; }
        let ids = (properties.properties.vendor_id, properties.properties.device_id);
        if identity.device_luid_valid != vk::TRUE || identity.device_node_mask != 1 { return None; }
        let luid = LUID {
            LowPart: u32::from_le_bytes(identity.device_luid[..4].try_into().ok()?),
            HighPart: i32::from_le_bytes(identity.device_luid[4..].try_into().ok()?),
        };
        let factory: IDXGIFactory4 = CreateDXGIFactory1().ok()?;
        let raw: IDXGIAdapter3 = factory.EnumAdapterByLuid(luid).ok()?;
        let sample = dxgi_memory_sample(&raw)?;
        if (sample.luid_low, sample.luid_high) != (luid.LowPart, luid.HighPart)
            || (sample.vendor_id, sample.device_id) != ids { return None; }
        Some(sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block_on<F: std::future::Future>(future: F) -> F::Output {
        let mut future = std::pin::pin!(future);
        let mut context = std::task::Context::from_waker(std::task::Waker::noop());
        loop {
            match future.as_mut().poll(&mut context) {
                std::task::Poll::Ready(value) => return value,
                std::task::Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    #[test]
    fn query_uses_each_actual_dx12_adapter() {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: Default::default(),
        });
        let adapters = block_on(instance.enumerate_adapters(wgpu::Backends::DX12));
        if adapters.is_empty() { eprintln!("SKIP: no DX12 adapter"); return; }
        for adapter in adapters {
            let info = adapter.get_info();
            let sample = dx12_adapter_memory(&adapter).expect("DXGI query on actual wgpu DX12 adapter");
            assert_eq!(sample.vendor_id, info.vendor);
            assert_eq!(sample.device_id, info.device);
            eprintln!("DX12 sample: LUID={:08x}:{:08x}, dedicated={} local_budget={} local_usage={}", sample.luid_high as u32, sample.luid_low, sample.dedicated_video_bytes, sample.local_budget_bytes, sample.local_usage_bytes);
        }
    }

    #[test]
    fn non_dx12_adapter_is_not_misidentified() {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: Default::default(),
        });
        let adapters = block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN));
        if adapters.is_empty() { eprintln!("SKIP: no Vulkan adapter"); return; }
        for adapter in adapters {
            assert_eq!(adapter.get_info().backend, wgpu::Backend::Vulkan);
            assert!(dx12_adapter_memory(&adapter).is_none());
        }
    }

    #[test]
    fn vulkan_budget_uses_the_matching_wgpu_device() {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            flags: Default::default(), memory_budget_thresholds: Default::default(),
            backend_options: Default::default(), display: Default::default(),
        });
        let adapters = block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN));
        if adapters.is_empty() { eprintln!("SKIP: no Vulkan adapter"); return; }
        let adapter = &adapters[0];
        let (device, _queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).expect("Vulkan device");
        // SAFETY: This guard is read-only and lives only for the extension check.
        let supported = unsafe { device.as_hal::<wgpu::hal::api::Vulkan>().unwrap().enabled_device_extensions().contains(&ash::ext::memory_budget::NAME) };
        let sample = vulkan_device_memory(adapter, &device);
        if let Some(dxgi) = vulkan_dxgi_adapter_memory(adapter, &device) {
            let info = adapter.get_info();
            assert_eq!((dxgi.vendor_id, dxgi.device_id), (info.vendor, info.device));
            assert!(dxgi.dedicated_video_bytes > 0 || info.device_type != wgpu::DeviceType::DiscreteGpu);
            eprintln!("Vulkan exact-LUID DXGI sample: {dxgi:?}");
        } else {
            eprintln!("Vulkan exact-LUID DXGI query unavailable; conservative fallback remains active");
        }
        if supported {
            let heaps = sample.expect("enabled Vulkan memory budget extension");
            assert!(!heaps.is_empty());
            for heap in heaps { assert!(heap.size_bytes > 0); eprintln!("Vulkan heap: {heap:?}"); }
        } else {
            assert!(sample.is_none());
            eprintln!("Vulkan adapter does not enable VK_EXT_memory_budget");
        }
        if let Some(other) = adapters.get(1) {
            assert!(vulkan_device_memory(other, &device).is_none(), "different physical devices cannot share a sample");
            assert!(vulkan_dxgi_adapter_memory(other, &device).is_none(), "different physical devices cannot share a DXGI identity");
        }
    }
}
