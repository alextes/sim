// instanced textured-quad shader for the tile world.
//
// the quad corners are generated from the vertex index (no vertex buffer); each
// instance supplies a screen-space dest rect, an atlas uv rect, and an rgba
// tint. fragment = atlas sample * tint, which reproduces the old sdl
// set_color_mod / set_alpha_mod behavior.

struct Globals {
    // viewport size in physical pixels.
    screen: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(1) @binding(0) var atlas_tex: texture_2d<f32>;
@group(1) @binding(1) var atlas_smp: sampler;

struct InstanceIn {
    @location(0) dest: vec2<f32>,    // top-left, physical pixels
    @location(1) size: vec2<f32>,    // physical pixels
    @location(2) uv_min: vec2<f32>,
    @location(3) uv_size: vec2<f32>,
    @location(4) tint: vec4<f32>,
};

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, inst: InstanceIn) -> VsOut {
    // two triangles forming a unit quad.
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );
    let corner = corners[vi];

    let px = inst.dest + corner * inst.size;
    // pixel space (origin top-left, y down) -> clip space (y up).
    let ndc = vec2<f32>(
        px.x / globals.screen.x * 2.0 - 1.0,
        1.0 - px.y / globals.screen.y * 2.0,
    );

    var out: VsOut;
    out.pos = vec4<f32>(ndc, 0.0, 1.0);
    out.uv = inst.uv_min + corner * inst.uv_size;
    out.tint = inst.tint;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let sampled = textureSample(atlas_tex, atlas_smp, in.uv);
    return sampled * in.tint;
}
