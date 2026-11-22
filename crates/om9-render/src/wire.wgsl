// Model wires reuse the resident line list. Each pair of vertices is an instance.
struct Uniforms {
    view_projection: mat4x4<f32>,
    model: mat4x4<f32>,
    depth_plane: vec4<f32>,
    viewport: vec2<f32>,
};
@group(0) @binding(0) var<uniform> u: Uniforms;

struct Stroke {
    @builtin(position) position: vec4<f32>,
    @location(0) colour: vec4<f32>,
};

@vertex
fn wire(@location(0) a: vec3<f32>, @location(1) colour: vec4<f32>,
        @location(2) b: vec3<f32>, @builtin(vertex_index) corner: u32) -> Stroke {
    let wa = u.model * vec4<f32>(a, 1.);
    let wb = u.model * vec4<f32>(b, 1.);
    var ca = u.view_projection * wa;
    var cb = u.view_projection * wb;
    var da = dot(u.depth_plane, wa);
    var db = dot(u.depth_plane, wb);
    // Clip the centreline against the near plane before dividing by w.
    if colour.a == -2. || (ca.z < 0. && cb.z < 0.) {
        return Stroke(vec4<f32>(0., 0., -1., 1.), colour);
    }
    if ca.z < 0. {
        let t = ca.z / (ca.z - cb.z);
        ca = mix(ca, cb, t);
        da = mix(da, db, t);
    } else if cb.z < 0. {
        let t = cb.z / (cb.z - ca.z);
        cb = mix(cb, ca, t);
        db = mix(db, da, t);
    }
    let tangent = (cb.xy / cb.w - ca.xy / ca.w) * u.viewport;
    let normal = vec2<f32>(-tangent.y, tangent.x) / max(length(tangent), 1e-6);
    let at_b = corner >= 2u;
    let depth = clamp(select(da, db, at_b), 0., 1.);
    let width_px = select(mix(2.0, 1.0, depth), 1.0, colour.a < 0.);
    var clip = select(ca, cb, at_b);
    let offset = normal * select(-1., 1., (corner & 1u) != 0u) * width_px / u.viewport;
    clip = vec4<f32>(clip.xy + offset * clip.w, clip.zw);
    return Stroke(clip, vec4<f32>(colour.rgb * mix(1., 0.65, depth), abs(colour.a)));
}

@fragment
fn shade(input: Stroke) -> @location(0) vec4<f32> {
    return input.colour;
}
