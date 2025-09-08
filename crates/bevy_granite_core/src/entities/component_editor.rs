use bevy::{
    prelude::*,
    reflect::{serde::TypedReflectDeserializer, FromType, ReflectDeserialize, TypeRegistration},
};
use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};
use serde::de::DeserializeSeed;
use std::{any::Any, borrow::Cow, collections::HashMap};

// All structs defined by #[granite_component]
// get this tag so we can easily filter in UI
#[derive(Clone)]
pub struct BridgeTag;
impl<T> FromType<T> for BridgeTag {
    fn from_type() -> Self {
        BridgeTag
    }
}

pub fn is_bridge_component_check(registration: &TypeRegistration) -> bool {
    registration.data::<BridgeTag>().is_some()
}

#[derive(Clone)]
pub struct ExposedToEditor {
    pub read_only: bool,
}

pub fn is_exposed_bevy_component(registration: &TypeRegistration) -> bool {
    registration.data::<ExposedToEditor>().is_some()
}

//

#[derive(Debug)]
pub struct ReflectedComponent {
    pub type_name: Cow<'static, str>,
    pub reflected_data: Box<dyn PartialReflect>,
    pub type_registration: TypeRegistration,
}

impl Clone for ReflectedComponent {
    fn clone(&self) -> Self {
        Self {
            type_name: self.type_name.clone(),
            reflected_data: self
                .reflected_data
                .reflect_clone()
                .expect("ReflectedComponent to be clonable"),
            type_registration: self.type_registration.clone(),
        }
    }
}
impl PartialEq for ReflectedComponent {
    fn eq(&self, other: &Self) -> bool {
        if self.type_name != other.type_name {
            return false;
        }
        self.reflected_data
            .reflect_partial_eq(&*other.reflected_data)
            .unwrap_or(false)
    }
}

#[derive(Resource, Clone, Default)]
pub struct ComponentEditor {
    pub selected_entity: Option<Entity>,
    pub type_registry: AppTypeRegistry,
}

impl PartialEq for ComponentEditor {
    fn eq(&self, other: &Self) -> bool {
        self.selected_entity == other.selected_entity
    }
}

impl ComponentEditor {
    /// Constructor
    pub fn new(type_registry: AppTypeRegistry) -> Self {
        Self {
            selected_entity: None,
            type_registry,
        }
    }

    /// Set selected entity
    pub fn set_selected_entity(&mut self, entity: Entity) {
        self.selected_entity = Some(entity);
    }

    /// Get entity components that are reflectable
    pub fn get_reflected_components(
        &self,
        world: &World,
        entity: Entity,
        filter: bool,
    ) -> Vec<ReflectedComponent> {
        let mut components = Vec::new();

        let entity_ref = world.entity(entity);
        let archetype = entity_ref.archetype();
        let type_registry = self.type_registry.read();

        for component_id in archetype.components() {
            let component_info = world.components().get_info(component_id).unwrap();

            if let Some(type_id) = component_info.type_id() {
                if let Some(registration) = type_registry.get(type_id) {
                    let type_name = registration.type_info().type_path();
                    if filter && self.should_skip_component(registration) {
                        continue;
                    }
                    if let Some(reflect_component) = registration.data::<ReflectComponent>() {
                        if let Some(reflected) = reflect_component.reflect(entity_ref) {
                            if let Ok(clone) = reflected.reflect_clone() {
                                components.push(ReflectedComponent {
                                    type_name: type_name.into(),
                                    reflected_data: clone,
                                    type_registration: registration.clone(),
                                });
                            } else {
                                log!(
                                    LogType::Editor,
                                    LogLevel::Error,
                                    LogCategory::Entity,
                                    "Failed to clone reflected data for component: {}",
                                    type_name
                                );
                            }
                        }
                    }
                } else {
                    // silently handle this... not all internal components that are on an entity
                    // are reflect registered
                    continue;
                }
            }
        }

        components
    }

    /// Remove component by name
    pub fn remove_component_by_name(
        &self,
        world: &mut World,
        entity: Entity,
        component_type_name: &str,
    ) {
        let type_registry = self.type_registry.clone();

        if let Some(registration) = type_registry
            .clone()
            .read()
            .get_with_type_path(component_type_name)
        {
            if let Some(reflect_component) = registration.data::<ReflectComponent>() {
                let mut entity_mut = world.entity_mut(entity);
                reflect_component.remove(&mut entity_mut);
                log!(
                    LogType::Editor,
                    LogLevel::OK,
                    LogCategory::Entity,
                    "Removed component: {}",
                    component_type_name
                );
            }
        }
    }

    /// Save components for entities
    pub fn serialize_entity_components(
        &self,
        world: &World,
        entity: Entity,
    ) -> HashMap<String, String> {
        log!(
            LogType::Game,
            LogLevel::Info,
            LogCategory::System,
            "Serialize entity components called"
        );
        let mut serialized_components = HashMap::new();
        let type_registry = self.type_registry.read();

        let entity_ref = world.entity(entity);
        let archetype = entity_ref.archetype();

        for component_id in archetype.components() {
            let component_info = world.components().get_info(component_id).unwrap();

            if let Some(type_id) = component_info.type_id() {
                if let Some(registration) = type_registry.get(type_id) {
                    let type_name = registration.type_info().type_path();

                    if self.should_skip_component(registration) {
                        continue;
                    }

                    if let Some(reflect_component) = registration.data::<ReflectComponent>() {
                        if let Some(reflected_value) = reflect_component.reflect(entity_ref) {
                            let serializer = bevy::reflect::serde::ReflectSerializer::new(
                                reflected_value,
                                &type_registry,
                            );
                            if let Ok(serialized) = ron::to_string(&serializer) {
                                serialized_components.insert(type_name.to_string(), serialized);
                            }
                        }
                    }
                }
            }
        }

        serialized_components
    }

    /// Insert components from serialized
    pub fn load_components_from_scene_data(
        &self,
        world: &mut World,
        entity: Entity,
        serialized_components: HashMap<String, String>,
        type_registry: AppTypeRegistry,
    ) {
        for (component_name, serialized_data) in serialized_components {
            log!(
                LogType::Game,
                LogLevel::Info,
                LogCategory::System,
                "Processing component: {} with data: {}",
                component_name,
                serialized_data
            );

            if let Some(registration) = type_registry.read().get_with_type_path(&component_name) {
                // Parse the wrapper to extract just the component data part
                if let Ok(parsed) = ron::from_str::<HashMap<String, ron::Value>>(&serialized_data) {
                    if let Some(_component_value) = parsed.get(&component_name) {
                        // Find the component name in quotes and extract what comes after the colon
                        let search_pattern = format!("\"{}\":", component_name);
                        if let Some(start) = serialized_data.find(&search_pattern) {
                            let after_colon = start + search_pattern.len();
                            let ron_part = &serialized_data[after_colon..serialized_data.len() - 1]; // Remove trailing }
                            let clean_ron = ron_part.trim();

                            if let Ok(mut deserializer) = ron::de::Deserializer::from_str(clean_ron)
                            {
                                // Try ReflectDeserialize first, then fall back to reflection-based deserialization
                                if let Some(reflect_deserialize) =
                                    registration.data::<ReflectDeserialize>()
                                {
                                    match reflect_deserialize.deserialize(&mut deserializer) {
                                        Ok(component_data) => {
                                            if let Some(reflect_component) =
                                                registration.data::<ReflectComponent>()
                                            {
                                                let mut entity_mut = world.entity_mut(entity);
                                                if entity_mut
                                                    .contains_type_id(reflect_component.type_id())
                                                {
                                                    reflect_component
                                                        .apply(&mut entity_mut, &*component_data);
                                                } else {
                                                    reflect_component.insert(
                                                        &mut entity_mut,
                                                        &*component_data,
                                                        &type_registry.read(),
                                                    );
                                                }
                                                log!(
                                                    LogType::Game,
                                                    LogLevel::Info,
                                                    LogCategory::Entity,
                                                    "Inserted: {}",
                                                    component_name
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            log!(
                                                LogType::Game,
                                                LogLevel::Error,
                                                LogCategory::System,
                                                "Failed to deserialize component {}: {:?}",
                                                component_name,
                                                e
                                            );
                                        }
                                    }
                                } else {
                                    // Try using reflection-based deserialization for types like Tonemapping
                                    if let Ok(mut full_deserializer) =
                                        ron::de::Deserializer::from_str(clean_ron)
                                    {
                                        // Try to deserialize using bevy's reflection system with a typed deserializer
                                        let type_registry_read = type_registry.read();

                                        // Use TypedReflectDeserializer for the specific type
                                        let typed_deserializer =
                                            bevy::reflect::serde::TypedReflectDeserializer::new(
                                                registration,
                                                &type_registry_read,
                                            );

                                        if let Ok(reflected_value) =
                                            typed_deserializer.deserialize(&mut full_deserializer)
                                        {
                                            if let Some(reflect_component) =
                                                registration.data::<ReflectComponent>()
                                            {
                                                let mut entity_mut = world.entity_mut(entity);
                                                if entity_mut
                                                    .contains_type_id(reflect_component.type_id())
                                                {
                                                    reflect_component
                                                        .apply(&mut entity_mut, &*reflected_value);
                                                } else {
                                                    reflect_component.insert(
                                                        &mut entity_mut,
                                                        &*reflected_value,
                                                        &type_registry_read,
                                                    );
                                                }
                                            }
                                        } else {
                                            log!(
                                                LogType::Game,
                                                LogLevel::Error,
                                                LogCategory::System,
                                                "Failed to deserialize component {} via typed reflection with data: '{}'",
                                                component_name,
                                                clean_ron
                                            );
                                        }
                                    } else {
                                        log!(
                                            LogType::Game,
                                            LogLevel::Error,
                                            LogCategory::System,
                                            "Failed to create deserializer for component {} with data: '{}'",
                                            component_name,
                                            clean_ron
                                        );
                                    }
                                }
                            } else {
                                log!(
                                    LogType::Game,
                                    LogLevel::Error,
                                    LogCategory::System,
                                    "Failed to create deserializer for component: {}",
                                    component_name
                                );
                            }
                        } else {
                            log!(
                                LogType::Game,
                                LogLevel::Error,
                                LogCategory::System,
                                "Could not find search pattern '{}' in serialized data for component: {}",
                                search_pattern,
                                component_name
                            );
                        }
                    } else {
                        log!(
                            LogType::Game,
                            LogLevel::Error,
                            LogCategory::System,
                            "Component value not found in parsed data for: {}",
                            component_name
                        );
                    }
                } else {
                    log!(
                        LogType::Game,
                        LogLevel::Error,
                        LogCategory::System,
                        "Failed to parse component data for: {}",
                        component_name
                    );
                }
            } else {
                log!(
                    LogType::Game,
                    LogLevel::Error,
                    LogCategory::System,
                    "No registration found for component: {}",
                    component_name
                );
            }
        }
    }

    /// Add new component to entity
    pub fn add_component_by_name(
        &self,
        world: &mut World,
        entity: Entity,
        component_type_name: &str,
    ) {
        let type_registry = self.type_registry.clone();
        if let Some(registration) = type_registry
            .clone()
            .read()
            .get_with_type_path(component_type_name)
        {
            if let Some(reflect_component) = registration.data::<ReflectComponent>() {
                let component = if let Some(reflect_default) = registration.data::<ReflectDefault>()
                {
                    reflect_default.default()
                } else {
                    if let Some(from_reflect) = registration.data::<ReflectFromReflect>() {
                        let dynamic_struct = bevy::reflect::DynamicStruct::default();
                        if let Some(component) = from_reflect.from_reflect(&dynamic_struct) {
                            component
                        } else {
                            log!(
                                LogType::Editor,
                                LogLevel::Error,
                                LogCategory::Entity,
                                "Failed to create component from reflection"
                            );
                            return;
                        }
                    } else {
                        log!(
                            LogType::Editor,
                            LogLevel::Error,
                            LogCategory::Entity,
                            "Component type has no Default or FromReflect"
                        );
                        return;
                    }
                };

                let mut entity_mut = world.entity_mut(entity);
                if entity_mut.contains_type_id(reflect_component.type_id()) {
                    reflect_component.apply(&mut entity_mut, &*component);
                } else {
                    reflect_component.insert(&mut entity_mut, &*component, &type_registry.read());
                }
                log!(
                    LogType::Editor,
                    LogLevel::OK,
                    LogCategory::Entity,
                    "Added new component: {}",
                    component_type_name
                );
            }
        }
    }

    /// Edit existing component on entity
    pub fn edit_component_by_name(
        &self,
        world: &mut World,
        entity: Entity,
        component_type_name: &str,
        reflected_data: &dyn bevy::reflect::PartialReflect,
    ) {
        let type_registry = self.type_registry.clone();

        if let Some(registration) = type_registry
            .clone()
            .read()
            .get_with_type_path(component_type_name)
        {
            if let Some(reflect_component) = registration.data::<ReflectComponent>() {
                let mut entity_mut = world.entity_mut(entity);
                if entity_mut.contains_type_id(reflect_component.type_id()) {
                    reflect_component.apply(&mut entity_mut, reflected_data);
                } else {
                    reflect_component.insert(
                        &mut entity_mut,
                        reflected_data,
                        &type_registry.read(),
                    );
                }
                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::Entity,
                    "Updated component: {}",
                    component_type_name
                );
            }
        }
    }

    /// Check for bridge tag
    pub fn should_skip_component(&self, registration: &TypeRegistration) -> bool {
        !is_bridge_component_check(registration) && !is_exposed_bevy_component(registration)
    }
}
