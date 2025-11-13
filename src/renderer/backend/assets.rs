use glm::*;

pub struct AssetManifest {
    pub models: Vec<ModelAsset>,
    pub textures: Vec<TextureAsset>,
    pub materials: Vec<MaterialAsset>,
}

pub struct ModelAsset {
    pub name: String,
    pub obj_path: String,
    pub pre_transform: Option<Mat4>,
    pub material_path: Option<String>,
}

pub struct TextureAsset {
    pub name: String,
    pub path: String,
}

pub struct MaterialAsset {
    pub name: String,
    pub properties: MaterialProperties, // Color, texture path, etc.
}

#[derive(Debug, Clone)]
pub struct MaterialProperties {
    pub name: String, // newmtl
    pub ns: f32,      // Ns: specular exponent
    pub ka: [f32; 3], // Ka: ambient color (RGB)
    pub kd: [f32; 3], // Kd: diffuse color (RGB)
    pub ks: [f32; 3], // Ks: specular color (RGB)
    pub ke: [f32; 3], // Ke: emissive color (RGB)
    pub ni: f32,      // Ni: optical density (index of refraction)
    pub d: f32,       // d: dissolve/transparency (1.0 = opaque)
    pub illum: u8,    // illum: illumination model
                      // // Optionally add more fields (e.g. texture maps, if needed):
                      // pub map_kd: Option<String>,   // map_Kd: diffuse texture map path
                      // pub map_ks: Option<String>,   // map_Ks: specular texture map path
                      // pub map_bump: Option<String>, // bump: bump map
}
