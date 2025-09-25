#import bevy_pbr::{
    pbr_types,
    pbr_functions::alpha_discard,
    pbr_fragment::pbr_input_from_standard_material,
    decal::clustered::apply_decal_base_color,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
}
#endif

#ifdef MESHLET_MESH_MATERIAL_PASS
#import bevy_pbr::meshlet_visibility_buffer_resolve::resolve_vertex_output
#endif

#ifdef OIT_ENABLED
#import bevy_core_pipeline::oit::oit_draw
#endif // OIT_ENABLED

#ifdef FORWARD_DECAL
#import bevy_pbr::decal::forward::get_forward_decal_info
#endif

// Rust의 FacialExpressionExtension 구조체와 1:1로 대응되는 부분
struct FacialExpressionExtension {
    mouth_index: vec4<u32>,
};

@group(2) @binding(100) var mouth_atlas: texture_2d<f32>;
@group(2) @binding(101) var mouth_sampler: sampler;
@group(2) @binding(102) var<uniform> extension: FacialExpressionExtension;

@fragment
fn fragment(
#ifdef MESHLET_MESH_MATERIAL_PASS
    @builtin(position) frag_coord: vec4<f32>,
#else
    vertex_output: VertexOutput,
    @builtin(front_facing) is_front: bool,
#endif
) -> FragmentOutput {
#ifdef MESHLET_MESH_MATERIAL_PASS
    let vertex_output = resolve_vertex_output(frag_coord);
    let is_front = true;
#endif

    var in = vertex_output;

    // If we're in the crossfade section of a visibility range, conditionally
    // discard the fragment according to the visibility pattern.
#ifdef VISIBILITY_RANGE_DITHER
    pbr_functions::visibility_range_dither(in.position, in.visibility_range_dither);
#endif

#ifdef FORWARD_DECAL
    let forward_decal_info = get_forward_decal_info(in);
    in.world_position = forward_decal_info.world_position;
    in.uv = forward_decal_info.uv;
#endif

    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // 지정된 UV 좌표 영역(u <= 0.25, v >= 0.75)인지 확인합니다.
    if (in.uv.x <= 0.25 && in.uv.y >= 0.75) {
        // UV 좌표를 조건 영역(0..1)에 맞게 리매핑(remap)합니다.
        let remapped_uv = vec2<f32>(
            in.uv.x / 0.25,
            (in.uv.y - 0.75) / 0.25
        );

        // 입 모양 아틀라스에서 현재 인덱스에 맞는 텍스처를 샘플링합니다.
        let atlas_cols = 4.0;
        let atlas_rows = 1.0;
        let tile_size = vec2<f32>(1.0 / atlas_cols, 1.0 / atlas_rows);
        let mouth_x = f32(extension.mouth_index.x % u32(atlas_cols));
        let mouth_y = floor(f32(extension.mouth_index.x) / atlas_cols);
        let mouth_uv_offset = vec2<f32>(mouth_x * tile_size.x, mouth_y * tile_size.y);
        let mouth_uv = remapped_uv * tile_size + mouth_uv_offset;
        let mouth_color = textureSample(mouth_atlas, mouth_sampler, mouth_uv);

        // Overwrite 로직: 입 모양 텍스처가 투명하지 않으면 base_color를 덮어씁니다.
        if (mouth_color.a > 0.5) {
            pbr_input.material.base_color = mouth_color;
        }
    }

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    // clustered decals
    pbr_input.material.base_color = apply_decal_base_color(
        in.world_position.xyz,
        in.position.xy,
        pbr_input.material.base_color
    );
#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in, pbr_input);
#else
    // in forward mode, we calculate the lit color immediately, and then apply some post-lighting effects here.
    // in deferred mode the lit color and these effects will be calculated in the deferred lighting shader
    var out: FragmentOutput;
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
    } else {
        out.color = pbr_input.material.base_color;
    }

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

#ifdef OIT_ENABLED
    let alpha_mode = pbr_input.material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITS;
    if alpha_mode != pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE {
        // The fragments will only be drawn during the oit resolve pass.
        oit_draw(in.position, out.color);
        discard;
    }
#endif // OIT_ENABLED

#ifdef FORWARD_DECAL
        out.color.a = min(forward_decal_info.alpha, out.color.a);
#endif

        return out;
}
