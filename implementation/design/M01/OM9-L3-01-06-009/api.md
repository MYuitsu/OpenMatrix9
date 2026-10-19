# Device loss recovery (OM9-L3-01-06-009)

`simulate_device_loss()` destroys the wgpu device and clears the cache; `render` then returns
`RenderError::Lost` instead of panicking; `recover()` builds a fresh device and pipeline and returns true.
The gate measured the same scenario in `native/gpu-probe --bin loss` (callback fires once, the destroyed
device refuses to draw, recovery takes 0.337 s).

Test: `a_lost_device_is_detected_and_recovered` — draw, lose, `Err(Lost)`, recover, draw again, and the
cache must refill from zero (one upload on the fresh device).
