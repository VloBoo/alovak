use std::fs;

use naga::{
    back::spv::{self},
    front::glsl,
    valid::{Capabilities, ValidationFlags, Validator},
    ShaderStage,
};

fn main() {
    let shader_vert = fs::write(
        "./shaders/vert.spv",
        compile_shader(
            &fs::read_to_string("./shaders/shader.vert").unwrap(),
            ShaderStage::Vertex,
        )
        .unwrap()
        .iter()
        .map(|value| *value as u8)
        .collect::<Vec<u8>>(),
    );
    let shader_vert = fs::write(
        "./shaders/frag.spv",
        compile_shader(
            &fs::read_to_string("./shaders/shader.frag").unwrap(),
            ShaderStage::Fragment,
        )
        .unwrap()
        .iter()
        .map(|value| *value as u8)
        .collect::<Vec<u8>>(),
    );
}

fn compile_shader(src: &str, stage: ShaderStage) -> Result<Vec<u32>, String> {
    // Загружаем GLSL шейдер с помощью Naga
    let options = glsl::Options::from(stage);
    let mut frontend = glsl::Frontend::default();
    let module = frontend
        .parse(&options, src)
        .map_err(|e| format!("{:?}", e))
        .unwrap();

    // Валидация
    let mut validator = Validator::new(ValidationFlags::default(), Capabilities::default());
    let module_info = validator
        .validate(&module)
        .map_err(|e| format!("{:?}", e))
        .unwrap();

    // Генерация SPIR-V
    let spv_options = spv::Options::default();
    let mut spv_writer = spv::Writer::new(&spv_options).map_err(|e| format!("{:?}", e))?;
    let mut spv_binary = vec![];
    spv_writer
        .write(&module, &module_info, None, &None, &mut spv_binary)
        .map_err(|e| format!("{:?}", e))
        .unwrap();

    Ok(spv_binary)
}
