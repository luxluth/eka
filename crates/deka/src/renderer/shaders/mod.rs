pub mod rectfs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/renderer/shaders/rect.frag.glsl"
    }
}

pub mod rectvs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/renderer/shaders/rect.vert.glsl"
    }
}
