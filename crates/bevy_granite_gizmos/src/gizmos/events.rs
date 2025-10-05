use super::GizmoType;
use bevy::prelude::{Entity, Message};

#[derive(Message)]
pub struct RotateInitDragEvent;

#[derive(Message)]
pub struct RotateDraggingEvent;

#[derive(Message)]
pub struct RotateResetDragEvent;

#[derive(Message)]
pub struct TransformInitDragEvent;

#[derive(Message)]
pub struct TransformDraggingEvent;

#[derive(Message)]
pub struct TransformResetDragEvent;

#[derive(Message)]
pub struct SpawnGizmoEvent(pub Entity);

#[derive(Message)]
pub struct DespawnGizmoEvent(pub GizmoType);
