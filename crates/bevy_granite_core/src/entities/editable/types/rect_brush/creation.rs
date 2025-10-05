use super::RectBrush;
use crate::{
    entities::EntitySaveReadyData, AvailableEditableMaterials, GraniteEditorSerdeEntity,
    GraniteType, GraniteTypes, HasRuntimeData, IdentityData, NeedsTangents,
};
use bevy::{
    asset::{AssetServer, Assets, RenderAssetUsages},
    ecs::{
        bundle::Bundle,
        entity::Entity,
        system::{Commands, Res, ResMut},
    },
    math::{Vec2, Vec3},
    mesh::{Indices, Mesh, Mesh3d, PrimitiveTopology},
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::Name,
    transform::components::Transform,
};
use uuid::Uuid;

impl RectBrush {
    /// Extract needed info to spawn this entity via save data
    pub fn spawn_from_save_data(
        save_data: &EntitySaveReadyData,
        commands: &mut Commands,
        standard_materials: &mut ResMut<Assets<StandardMaterial>>,
        available_materials: &mut ResMut<AvailableEditableMaterials>,
        asset_server: &Res<AssetServer>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> Entity {
        let identity = &save_data.identity;
        let save_transform = &save_data.transform;

        Self::spawn_from_identity(
            commands,
            identity,
            save_transform.to_bevy(),
            standard_materials,
            available_materials,
            asset_server,
            meshes,
        )
    }

    /// Take the name and class from identity to spawn
    pub fn spawn_from_identity(
        commands: &mut Commands,
        identity: &IdentityData,
        transform: Transform,
        standard_materials: &mut ResMut<Assets<StandardMaterial>>,
        available_materials: &mut ResMut<AvailableEditableMaterials>,
        asset_server: &Res<AssetServer>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> Entity {
        let mut class = Self::extract_class(&identity);

        class.spawn(
            identity,
            commands,
            transform,
            standard_materials,
            available_materials,
            asset_server,
            meshes,
        )
    }

    /// Generally to be used from UI popups as it gives default name
    pub fn spawn_from_new_identity(
        &mut self,
        commands: &mut Commands,
        transform: Transform,
        standard_materials: &mut ResMut<Assets<StandardMaterial>>,
        available_materials: &mut ResMut<AvailableEditableMaterials>,
        asset_server: &Res<AssetServer>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> Entity {
        let identity = IdentityData {
            name: self.type_name(),
            uuid: Uuid::new_v4(),
            class: GraniteTypes::RectBrush(self.clone()),
        };
        self.spawn(
            &identity,
            commands,
            transform,
            standard_materials,
            available_materials,
            asset_server,
            meshes,
        )
    }

    /// Private core logic
    fn spawn(
        &mut self,
        identity: &IdentityData,
        commands: &mut Commands,
        transform: Transform,
        standard_materials: &mut ResMut<Assets<StandardMaterial>>,
        available_materials: &mut ResMut<AvailableEditableMaterials>,
        asset_server: &Res<AssetServer>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> Entity {
        // Load and configure the material
        self.load_and_configure_material(available_materials, standard_materials, asset_server);

        commands
            .spawn(Self::get_bundle(
                self.clone(), // Clone AFTER fixing materials
                identity.clone(),
                transform,
                meshes,
            ))
            .id()
    }

    /// Build a bundle that is ready to spawn from a rect brush
    fn get_bundle(
        rectangle_brush: RectBrush,
        identity: IdentityData,
        transform: Transform,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> impl Bundle {
        let rect_mesh = Self::create_mesh(
            rectangle_brush.size.x,
            rectangle_brush.size.y,
            rectangle_brush.size.z,
            rectangle_brush.uv_scale,
        );
        let rect_handle = meshes.add(rect_mesh);

        (
            transform,
            Mesh3d(rect_handle),
            MeshMaterial3d(
                rectangle_brush
                    .material
                    .current
                    .handle
                    .clone()
                    .expect("Default material should always have a handle"),
            ),
            Name::new(identity.name.clone()),
            HasRuntimeData,
            GraniteEditorSerdeEntity,
            NeedsTangents,
            IdentityData {
                name: identity.name.clone(),
                uuid: identity.uuid.clone(),
                class: GraniteTypes::RectBrush(rectangle_brush.clone()), // Use the updated rectangle_brush, not the old identity.class
            },
        )
    }

    fn extract_class(identity: &IdentityData) -> RectBrush {
        match &identity.class {
            GraniteTypes::RectBrush(rect_data) => rect_data.clone(),
            _ => panic!("Expected RectBrush class data, got different type from save data"),
        }
    }

    pub fn get_vertices(x: f32, y: f32, z: f32) -> Vec<[f32; 3]> {
        vec![
            // Front face (0-3)
            [-x / 2.0, -y / 2.0, z / 2.0],
            [x / 2.0, -y / 2.0, z / 2.0],
            [x / 2.0, y / 2.0, z / 2.0],
            [-x / 2.0, y / 2.0, z / 2.0],
            // Back face (4-7)
            [-x / 2.0, -y / 2.0, -z / 2.0],
            [x / 2.0, -y / 2.0, -z / 2.0],
            [x / 2.0, y / 2.0, -z / 2.0],
            [-x / 2.0, y / 2.0, -z / 2.0],
            // Right face (8-11)
            [x / 2.0, -y / 2.0, z / 2.0],
            [x / 2.0, -y / 2.0, -z / 2.0],
            [x / 2.0, y / 2.0, -z / 2.0],
            [x / 2.0, y / 2.0, z / 2.0],
            // Left face (12-15)
            [-x / 2.0, -y / 2.0, -z / 2.0],
            [-x / 2.0, -y / 2.0, z / 2.0],
            [-x / 2.0, y / 2.0, z / 2.0],
            [-x / 2.0, y / 2.0, -z / 2.0],
            // Top face (16-19)
            [-x / 2.0, y / 2.0, z / 2.0],
            [x / 2.0, y / 2.0, z / 2.0],
            [x / 2.0, y / 2.0, -z / 2.0],
            [-x / 2.0, y / 2.0, -z / 2.0],
            // Bottom face (20-23)
            [-x / 2.0, -y / 2.0, -z / 2.0],
            [x / 2.0, -y / 2.0, -z / 2.0],
            [x / 2.0, -y / 2.0, z / 2.0],
            [-x / 2.0, -y / 2.0, z / 2.0],
        ]
    }

    pub fn get_uvs(uv_scale: Vec2) -> Vec<[f32; 2]> {
        vec![
            // Front face
            [0.0, uv_scale.y],
            [uv_scale.x, uv_scale.y],
            [uv_scale.x, 0.0],
            [0.0, 0.0],
            // Back face
            [0.0, uv_scale.y],
            [uv_scale.x, uv_scale.y],
            [uv_scale.x, 0.0],
            [0.0, 0.0],
            // Right face
            [0.0, uv_scale.y],
            [uv_scale.x, uv_scale.y],
            [uv_scale.x, 0.0],
            [0.0, 0.0],
            // Left face
            [0.0, uv_scale.y],
            [uv_scale.x, uv_scale.y],
            [uv_scale.x, 0.0],
            [0.0, 0.0],
            // Top face
            [0.0, uv_scale.y],
            [uv_scale.x, uv_scale.y],
            [uv_scale.x, 0.0],
            [0.0, 0.0],
            // Bottom face
            [0.0, uv_scale.y],
            [uv_scale.x, uv_scale.y],
            [uv_scale.x, 0.0],
            [0.0, 0.0],
        ]
    }

    pub fn get_indices() -> Vec<u32> {
        vec![
            0, 1, 2, 2, 3, 0, // Front face
            4, 7, 6, 6, 5, 4, // Back face
            8, 9, 10, 10, 11, 8, // Right face
            12, 13, 14, 14, 15, 12, // Left face
            16, 17, 18, 18, 19, 16, // Top face
            20, 21, 22, 22, 23, 20, // Bottom face
        ]
    }

    pub fn update_normals(mesh: &mut Mesh) {
        if let (Some(positions), Some(indices)) =
            (mesh.attribute(Mesh::ATTRIBUTE_POSITION), mesh.indices())
        {
            let vertices: Vec<[f32; 3]> = positions
                .as_float3()
                .expect("Position attribute should be Vec3")
                .to_vec();

            let index_data = match indices {
                Indices::U16(data) => data.iter().map(|&i| i as u32).collect(),
                Indices::U32(data) => data.clone(),
            };

            let normals = Self::calculate_normals(&vertices, &index_data);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        }
    }

    pub fn calculate_normals(vertices: &Vec<[f32; 3]>, indices: &Vec<u32>) -> Vec<[f32; 3]> {
        let mut normals = vec![[0.0, 0.0, 0.0]; vertices.len()];

        for chunk in indices.chunks(3) {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            let v0 = Vec3::from(vertices[i0]);
            let v1 = Vec3::from(vertices[i1]);
            let v2 = Vec3::from(vertices[i2]);

            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let normal = edge1.cross(edge2).normalize();

            let normal_array = [normal.x, normal.y, normal.z];
            normals[i0] = normal_array;
            normals[i1] = normal_array;
            normals[i2] = normal_array;
        }

        normals
    }

    pub fn create_mesh(x: f32, y: f32, z: f32, uv_scale: Vec2) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        let vertices = Self::get_vertices(x, y, z);
        let uvs = Self::get_uvs(uv_scale);
        let indices = Self::get_indices();
        let normals = Self::calculate_normals(&vertices, &indices);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_indices(Indices::U32(indices));

        mesh
    }

    /// Load and configure material with proper metadata
    /// Uses a static friendly name and internal material as fallback as default rect brushes share the same material
    fn load_and_configure_material(
        &mut self,
        available_materials: &mut ResMut<AvailableEditableMaterials>,
        standard_materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: &Res<AssetServer>,
    ) {
        // Ensure material existence and presence in scene
        // If doesn't exist, this it probable the first time we are using a rect brush
        let fallback_path = Self::internal_material_path();

        // Use saved material path if available, otherwise use fallback
        let material_path = if !self.material.path.is_empty() {
            &self.material.path
        } else {
            &fallback_path
        };

        // Set the path on the current material before loading
        self.material.current.path = material_path.to_string();

        let _created_new = self.material.current.material_exists_and_load(
            available_materials,
            standard_materials,
            asset_server,
            &self.type_name(),
            material_path,
        );

        // Fix the material metadata after loading (since loaded materials have "None" path)
        self.material.current.path = material_path.to_string();
        self.material.current.friendly_name = self.type_name();

        // Always set last = current after material loading, regardless of whether it was new or existing
        self.material.last = self.material.current.clone();

        // Only update path if we used fallback (new entity), not when loading from save
        if self.material.path.is_empty() {
            self.material.path = fallback_path.clone();
        }
    }
}
