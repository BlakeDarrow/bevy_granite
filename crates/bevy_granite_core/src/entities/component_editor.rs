use bevy::{
    prelude::*,
    reflect::{FromType, ReflectDeserialize, TypeRegistration},
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
            let component_info = world.components().get_info(component_id.clone()).unwrap();

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
                                    "Failed to clone reflected data for component: {}\nReflect info: {:?}\nType info: {:?}",
                                    type_name,
                                    reflected.reflect_kind(),
                                    registration.type_info()
                                );
                                
                                // Try to get more details about what field is causing the issue
                                if let bevy::reflect::ReflectKind::Struct = reflected.reflect_kind() {
                                    if let Ok(struct_ref) = reflected.reflect_ref().as_struct() {
                                        log!(
                                            LogType::Editor,
                                            LogLevel::Error,
                                            LogCategory::Entity,
                                            "Struct has {} fields",
                                            struct_ref.field_len()
                                        );
                                        for i in 0..struct_ref.field_len() {
                                            if let Some(field) = struct_ref.field_at(i) {
                                                let field_name = struct_ref.name_at(i).unwrap_or("unknown");
                                                if field.reflect_clone().is_err() {
                                                    log!(
                                                        LogType::Editor,
                                                        LogLevel::Error,
                                                        LogCategory::Entity,
                                                        "  Field '{}' (index {}) failed to clone. Type: {:?}",
                                                        field_name,
                                                        i,
                                                        field.reflect_type_path()
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
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
        //log!(
        //    LogType::Game,
        //    LogLevel::Info,
        //    LogCategory::System,
        //    "Serialize entity components called"
        //);
        let mut serialized_components = HashMap::new();
        let type_registry = self.type_registry.read();

        let entity_ref = world.entity(entity);
        let archetype = entity_ref.archetype();

        for component_id in archetype.components() {
            let component_info = world.components().get_info(component_id.clone()).unwrap();

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

    /// Insert components from serialized data with proper error handling
    pub fn load_components_from_scene_data(
        &self,
        world: &mut World,
        entity: Entity,
        serialized_components: HashMap<String, String>,
        type_registry: AppTypeRegistry,
    ) {
        let mut success_count = 0;
        let mut error_count = 0;

        for (component_name, serialized_data) in serialized_components {
            match self.process_single_component(
                world,
                entity,
                &component_name,
                &serialized_data,
                &type_registry,
            ) {
                Ok(()) => {
                    success_count += 1;
                }
                Err(e) => {
                    error_count += 1;
                    log!(
                        LogType::Game,
                        LogLevel::Error,
                        LogCategory::System,
                        "Failed to load component {}: {}",
                        component_name,
                        e
                    );
                }
            }
        }

        log!(
            LogType::Game,
            LogLevel::Info,
            LogCategory::Entity,
            "Component loading complete: {} successful, {} failed",
            success_count,
            error_count
        );
    }

    /// Process a single component with comprehensive error handling
    fn process_single_component(
        &self,
        world: &mut World,
        entity: Entity,
        component_name: &str,
        serialized_data: &str,
        type_registry: &AppTypeRegistry,
    ) -> Result<(), String> {
        let registration = {
            let type_registry_read = type_registry.read();
            type_registry_read
                .get_with_type_path(component_name)
                .ok_or_else(|| format!("No registration found for component: {}", component_name))?
                .clone()
        };
        let clean_ron = self
            .extract_component_data(component_name, serialized_data)
            .ok_or_else(|| format!("Failed to extract component data for: {}", component_name))?;

        self.deserialize_and_insert_component(
            world,
            entity,
            component_name,
            &clean_ron,
            &registration,
            type_registry,
        )
    }

    /// Extract the data for a component using proper RON parsing
    fn extract_component_data(
        &self,
        component_name: &str,
        serialized_data: &str,
    ) -> Option<String> {
        log!(
            LogType::Game,
            LogLevel::Info,
            LogCategory::System,
            "Extract component data called for '{}' with data: {}",
            component_name,
            serialized_data
        );
        
        // First, try to parse the original data as RON to see if we can extract the component directly
        if let Some(extracted) = self.try_extract_ron_component(component_name, serialized_data) {
            log!(
                LogType::Game,
                LogLevel::Info,
                LogCategory::System,
                "Successfully extracted via try_extract_ron_component"
            );
            return Some(extracted);
        }
        
        log!(
            LogType::Game,
            LogLevel::Info,
            LogCategory::System,
            "try_extract_ron_component returned None, falling back to map parsing"
        );

        // Store the original serialized data for Unit value lookups
        let original_data = serialized_data;

        // Fallback to the existing JSON-based approach for backwards compatibility
        let parsed = ron::from_str::<HashMap<String, ron::Value>>(serialized_data).ok()?;
        let component_value = parsed.get(component_name)?;

        log!(
            LogType::Game,
            LogLevel::Info,
            LogCategory::System,
            "Parsed component value: {:?}",
            component_value
        );

        let result = match component_value {
            // For string values return without quotes
            ron::Value::String(s) => {
                log!(
                    LogType::Game,
                    LogLevel::Info,
                    LogCategory::System,
                    "Returning string value: '{}'",
                    s
                );
                Some(s.clone())
            }
            // For unit values we need to extract the original identifier
            ron::Value::Unit => {
                log!(
                    LogType::Game,
                    LogLevel::Info,
                    LogCategory::System,
                    "Found unit value - extracting identifier from original data"
                );

                // For Unit values, we need to extract the original identifier from the serialized data
                // Look for the pattern: "component_name":IDENTIFIER
                let search_pattern = format!("\"{}\":", component_name);
                if let Some(start) = serialized_data.find(&search_pattern) {
                    let after_colon = start + search_pattern.len();
                    let remaining = &serialized_data[after_colon..];

                    // Find the identifier (everything until } or end)
                    let identifier = remaining
                        .trim_start()
                        .split('}')
                        .next()
                        .unwrap_or("")
                        .trim();

                    log!(
                        LogType::Game,
                        LogLevel::Info,
                        LogCategory::System,
                        "Extracted identifier: '{}'",
                        identifier
                    );

                    if !identifier.is_empty() {
                        Some(identifier.to_string())
                    } else {
                        // For unit structs like ()
                        Some("()".to_string())
                    }
                } else {
                    None
                }
            }
            // For Map values, convert to proper RON struct syntax
            ron::Value::Map(map) => {
                log!(
                    LogType::Game,
                    LogLevel::Info,
                    LogCategory::System,
                    "Converting Map to RON struct syntax"
                );
                Some(self.convert_map_to_ron_struct(map, original_data))
            }
            // For Sequence values, convert to tuple format for tuple structs
            ron::Value::Seq(seq) => {
                log!(
                    LogType::Game,
                    LogLevel::Info,
                    LogCategory::System,
                    "Converting Seq to tuple format for tuple struct"
                );
                Some(self.convert_seq_to_tuple(seq))
            }
            // For other types, keep as RON format instead of converting to JSON
            other => {
                // Try to serialize back to RON to maintain the expected format
                match ron::to_string(other) {
                    Ok(component_ron) => {
                        log!(
                            LogType::Game,
                            LogLevel::Info,
                            LogCategory::System,
                            "Serializing to RON: {:?} -> {}",
                            other,
                            component_ron
                        );
                        Some(component_ron)
                    }
                    Err(e) => {
                        log!(
                            LogType::Game,
                            LogLevel::Error,
                            LogCategory::System,
                            "Failed to serialize component value for {}: {:?}",
                            component_name,
                            e
                        );
                        None
                    }
                }
            }
        };

        log!(
            LogType::Game,
            LogLevel::Info,
            LogCategory::System,
            "Final extracted data for '{}': {:?}",
            component_name,
            result
        );

        result
    }

    /// Try to extract component data directly from RON format
    fn try_extract_ron_component(
        &self,
        component_name: &str,
        serialized_data: &str,
    ) -> Option<String> {
        log!(
            LogType::Game,
            LogLevel::Info,
            LogCategory::System,
            "try_extract_ron_component: Looking for component '{}'",
            component_name
        );
        
        // The data format is: {"component_name": (struct_data)}
        // We need to extract just the (struct_data) part
        let search_pattern = format!("\"{}\":", component_name);
        if let Some(start) = serialized_data.find(&search_pattern) {
            log!(
                LogType::Game,
                LogLevel::Info,
                LogCategory::System,
                "Found pattern at position {}",
                start
            );
            
            let after_colon = start + search_pattern.len();
            let remaining = &serialized_data[after_colon..].trim_start();
            
            // The data should be like: (manufacturer:Some(...),components:[(unit:W,...)])
            // Just extract from the opening ( to the closing }
            if remaining.starts_with('(') {
                // Find the matching closing paren
                let mut depth = 0;
                let mut end_pos = None;
                
                for (i, ch) in remaining.chars().enumerate() {
                    match ch {
                        '(' | '[' => depth += 1,
                        ')' | ']' => {
                            depth -= 1;
                            if depth == 0 {
                                end_pos = Some(i + 1);
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                
                if let Some(end) = end_pos {
                    let extracted = &remaining[..end];
                    
                    // Clean up Float() wrappers
                    let cleaned = self.clean_extracted_ron(extracted);
                    
                    log!(
                        LogType::Game,
                        LogLevel::Info,
                        LogCategory::System,
                        "Successfully extracted RON data: {}",
                        &cleaned[..cleaned.len().min(200)]
                    );
                    
                    return Some(cleaned);
                }
            }
        }
        
        log!(
            LogType::Game,
            LogLevel::Info,
            LogCategory::System,
            "try_extract_ron_component: Failed to extract"
        );
        None
    }
    
    /// Clean extracted RON data by removing Float() wrappers while preserving structure
    fn clean_extracted_ron(&self, ron_data: &str) -> String {
        let mut result = ron_data.to_string();
        
        // Remove Float() wrappers
        while result.contains("Float(") {
            // Find Float( and its matching closing )
            if let Some(start) = result.find("Float(") {
                // Find the matching closing paren
                let after_float = start + 6; // length of "Float("
                let remaining = &result[after_float..];
                
                // Simple approach: find the first closing paren (works for simple numbers)
                if let Some(end_offset) = remaining.find(')') {
                    let number = &remaining[..end_offset];
                    let before = &result[..start];
                    let after = &remaining[end_offset + 1..];
                    result = format!("{}{}{}", before, number, after);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        result
    }

    /// Convert a RON Map to proper struct syntax
    fn convert_map_to_ron_struct(&self, map: &ron::Map, original_data: &str) -> String {
        let mut fields = Vec::new();

        for (key, value) in map.iter() {
            if let ron::Value::String(field_name) = key {
                // Recursively convert the value to proper RON format
                let field_value = self.convert_ron_value_to_struct(value, original_data);
                fields.push(format!("{}:{}", field_name, field_value));
            }
        }

        format!("({})", fields.join(","))
    }

    /// Convert a RON value to proper struct syntax recursively
    fn convert_ron_value_to_struct(&self, value: &ron::Value, original_data: &str) -> String {
        match value {
            ron::Value::Map(map) => {
                // Recursively convert nested maps to struct syntax
                self.convert_map_to_ron_struct(map, original_data)
            }
            ron::Value::Seq(seq) => {
                // Check if this is a numeric array (Vec2, Vec3, Vec4, etc.)
                let is_numeric_array = seq.iter().all(|v| matches!(v, ron::Value::Number(_)));
                
                // Convert sequence elements recursively
                let elements: Vec<String> = seq
                    .iter()
                    .map(|v| self.convert_ron_value_to_struct(v, original_data))
                    .collect();
                
                if is_numeric_array {
                    // Use tuple syntax for numeric arrays (Vec2, Vec3, etc.)
                    format!("({})", elements.join(","))
                } else {
                    // Use array syntax for struct arrays
                    format!("[{}]", elements.join(","))
                }
            }
            ron::Value::Option(opt) => {
                match opt {
                    Some(inner) => format!("Some({})", self.convert_ron_value_to_struct(inner, original_data)),
                    None => "None".to_string()
                }
            }
            ron::Value::Unit => {
                // For Unit values, try to extract the original enum variant name from the source
                // This is a workaround because ron::Value::Unit loses the variant name
                // We'll return a placeholder that needs to be handled by a smarter parser
                "()".to_string()
            }
            other => {
                // For primitives, serialize and clean up
                let serialized = ron::to_string(other).unwrap_or_default();
                self.clean_ron_value(&serialized)
            }
        }
    }

    /// Convert a RON Seq to tuple format for tuple structs
    fn convert_seq_to_tuple(&self, seq: &Vec<ron::Value>) -> String {
        let mut values = Vec::new();

        for value in seq.iter() {
            // Serialize each value and clean it up
            let serialized_value = ron::to_string(value).unwrap_or_default();
            let cleaned_value = self.clean_ron_value(&serialized_value);
            values.push(cleaned_value);
        }

        format!("({})", values.join(","))
    }

    /// Clean up RON serialized values by removing wrapper types
    fn clean_ron_value(&self, ron_str: &str) -> String {
        let mut result = ron_str.to_string();

        // Remove Float() wrappers
        while result.contains("Float(") {
            result = result.replace("Float(", "").replace(")", "");
        }

        // Don't convert arrays to tuples - leave them as arrays
        // RON deserializer expects proper array syntax for sequence fields

        result
    }

    /// Try to deserialize using multiple strategies
    fn deserialize_and_insert_component(
        &self,
        world: &mut World,
        entity: Entity,
        component_name: &str,
        clean_ron: &str,
        registration: &TypeRegistration,
        type_registry: &AppTypeRegistry,
    ) -> Result<(), String> {
        let Ok(mut deserializer) = ron::de::Deserializer::from_str(clean_ron) else {
            return Err(format!(
                "Failed to create deserializer for component: {}",
                component_name
            ));
        };

        // Strategy 1: Try ReflectDeserialize (for components with serde support)
        if let Some(reflect_deserialize) = registration.data::<ReflectDeserialize>() {
            if let Ok(()) = self.try_reflect_deserialize(
                world,
                entity,
                component_name,
                &mut deserializer,
                reflect_deserialize,
                registration,
                type_registry,
            ) {
                return Ok(());
            }
        }

        // Strategy 2: Fallback to TypedReflectDeserializer (for Bevy components with reflection only)
        self.try_typed_reflection_deserialize(
            world,
            entity,
            component_name,
            clean_ron,
            registration,
            type_registry,
        )
    }

    /// Try deserializing using ReflectDeserialize
    fn try_reflect_deserialize(
        &self,
        world: &mut World,
        entity: Entity,
        component_name: &str,
        deserializer: &mut ron::de::Deserializer,
        reflect_deserialize: &ReflectDeserialize,
        registration: &TypeRegistration,
        type_registry: &AppTypeRegistry,
    ) -> Result<(), String> {
        match reflect_deserialize.deserialize(deserializer) {
            Ok(component_data) => {
                self.insert_reflected_component(
                    world,
                    entity,
                    component_name,
                    &*component_data,
                    registration,
                    type_registry,
                )?;
                Ok(())
            }
            Err(e) => Err(format!(
                "Failed to deserialize component {}: {:?}",
                component_name, e
            )),
        }
    }

    /// Try deserializing using TypedReflectDeserializer
    fn try_typed_reflection_deserialize(
        &self,
        world: &mut World,
        entity: Entity,
        component_name: &str,
        clean_ron: &str,
        registration: &TypeRegistration,
        type_registry: &AppTypeRegistry,
    ) -> Result<(), String> {
        let Ok(mut full_deserializer) = ron::de::Deserializer::from_str(clean_ron) else {
            return Err(format!(
                "Failed to create deserializer for typed reflection: {}",
                component_name
            ));
        };

        let type_registry_read = type_registry.read();
        let typed_deserializer =
            bevy::reflect::serde::TypedReflectDeserializer::new(registration, &type_registry_read);

        match typed_deserializer.deserialize(&mut full_deserializer) {
            Ok(reflected_value) => {
                self.insert_reflected_component(
                    world,
                    entity,
                    component_name,
                    &*reflected_value,
                    registration,
                    type_registry,
                )?;
                log!(
                    LogType::Game,
                    LogLevel::Info,
                    LogCategory::Entity,
                    "Inserted via typed reflection: {}",
                    component_name
                );
                Ok(())
            }
            Err(e) => Err(format!(
                "Failed to deserialize component {} via typed reflection: {:?}",
                component_name, e
            )),
        }
    }

    /// Insert a reflected component into an entity
    fn insert_reflected_component(
        &self,
        world: &mut World,
        entity: Entity,
        component_name: &str,
        component_data: &dyn bevy::reflect::PartialReflect,
        registration: &TypeRegistration,
        type_registry: &AppTypeRegistry,
    ) -> Result<(), String> {
        let Some(reflect_component) = registration.data::<ReflectComponent>() else {
            return Err(format!("No ReflectComponent found for: {}", component_name));
        };

        let mut entity_mut = world.entity_mut(entity);
        if entity_mut.contains_type_id(reflect_component.type_id()) {
            reflect_component.apply(&mut entity_mut, component_data);
        } else {
            reflect_component.insert(&mut entity_mut, component_data, &type_registry.read());
        }

        log!(
            LogType::Game,
            LogLevel::Info,
            LogCategory::Entity,
            "Inserted: {}",
            component_name
        );
        Ok(())
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
