# Adding New Granite Type 

Follow these exact steps to add a new type to the Granite system. Assuming you follow these steps correctly, this new module should allow the editor to see, save, edit, and load your new type. Please look at all the examples under the existing `types/` folder.

## Step 1: Create the Type Folder Structure
Create a new folder under `types/` with your type name (use snake_case). For example: `types/camera_2d/`

Required files in your new type folder:
- `mod.rs` - Main struct definition and GraniteType trait implementation
- `plugin.rs` - Bevy plugin registration
- `creation.rs` - Entity spawning/creation logic
- `ui.rs` - UI rendering and editing functions  
- `update_event.rs` - Event struct and system for handling updates
- `YourType.png` - Icon file (32x32 or 64x64 recommended)

## Step 2: Implement Your Type Struct
In `types/your_type/mod.rs`, create a struct that implements `GraniteType`:

```rust
use crate::entities::editable::definition::GraniteType;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct YourType {
    // Your type's fields here
}

impl GraniteType for YourType {
    // Implement all required trait methods
    // See other types or definition.rs for details
}
```

## Step 3: Create Update Event
In `types/your_type/update_event.rs`:

```rust
use bevy::prelude::*;

#[derive(Message)]
pub struct UpdateYourTypeEvent {
    pub entity: Entity,
    pub new_data: YourType,
}

pub fn update_your_type_system(
    mut events: MessageReader<UpdateYourTypeEvent>,
    mut query: Query<&mut YourType>, // and any related bevy info you need
) {
    // Handle type update events
}
```

## Step 4: Create Plugin
In `types/your_type/plugin.rs`:

```rust
use bevy::prelude::*;
use super::*;

pub struct YourTypePlugin;

impl Plugin for YourTypePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<UpdateYourTypeEvent>()
           .register_type::<YourType>
           .add_systems(Update, update_your_type_system);
    }
}
```

## Step 5: Register in Main Plugin
Add your plugin to `types/plugin.rs`:

```rust
// Add import
use super::your_type::plugin::YourTypePlugin;

// Add to plugin build method
app.add_plugins(YourTypePlugin);
```

## Step 6: Add to GraniteTypes Enum
**CRITICAL**: Add your type to the `GraniteTypes` enum in `editable/mod.rs`:

```rust
// Add your variant to the enum
pub enum GraniteTypes {
    // ...existing variants...
    YourType(YourType),
}

// Add to the all() function
impl GraniteTypes {
    pub fn all() -> Vec<GraniteTypes> {
        vec![
            // ...existing types...
            GraniteTypes::YourType(YourType::default()),
        ]
    }
}
```

## Step 7: Add Update Event to RequestEntityUpdateFromClass
In `editable/mod.rs`, add your update event to the `RequestEntityUpdateFromClass` SystemParam:

```rust
// Add MessageWriter for your type
pub your_type_writer: MessageWriter<'w, UpdateYourTypeEvent>,
```

## Step 8: Export Your Type
Add to `types/mod.rs`:

```rust
pub mod your_type;
pub use your_type::*;
```

## Examples to Reference:
- **Simple type**: Check `empty/` folder
- **Complex type with components**: Check `obj/` folder  
- **Light type**: Check `point_light/` or `directional_light/`
- **Custom mesh type**: Check `rectangle_brush/`
- **Camera type**: Check `camera_3d/`

## Required Trait Methods (GraniteType):
Refer to `definition.rs` for the complete trait definition. You must implement:
- Identification methods (name, category, etc.)
- UI methods (render_ui, get_icon, etc.)
- Entity methods (spawn_entity, update_entity, etc.)
- Serialization support (via derives)

## Testing Your New Type:
1. Build the project to ensure no compilation errors
2. Run the application and verify your type appears in the UI
3. Test creating, editing, and updating entities of your type
4. Verify serialization/deserialization works correctly

**Important**: Missing any of these steps will result in your type not being fully integrated into the Granite system.