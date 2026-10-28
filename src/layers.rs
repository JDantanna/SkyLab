//! Persistent file provenance; geometry keeps its layer through every edit.
use crate::{Project, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layer {
    pub id: u32,
    pub name: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}
pub fn validate(project: &Project) -> Result<()> {
    if project.layers.len() > 1000 || (!project.layers.is_empty() && project.version != 3) {
        return Err("Estrutura de camadas inválida.".into());
    }
    let mut ids = HashSet::new();
    for layer in &project.layers {
        if layer.id == 0
            || !ids.insert(layer.id)
            || layer.name.trim().is_empty()
            || layer.name.len() > 1024
            || layer.name.chars().any(char::is_control)
            || !["KML", "KMZ", "MIS", "Generated", "Project"].contains(&layer.source.as_str())
        {
            return Err("Identificação de camada inválida.".into());
        }
    }
    for id in project
        .lines
        .iter()
        .filter_map(|l| l.layer)
        .chain(project.polygons.iter().filter_map(|p| p.layer))
    {
        if !ids.contains(&id) {
            return Err("Geometria aponta para uma camada inexistente.".into());
        }
    }
    Ok(())
}
pub fn next_id(project: &Project) -> Result<u32> {
    project
        .layers
        .iter()
        .map(|l| l.id)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| "Limite de camadas atingido.".into())
}
pub fn subset(project: &Project, id: u32) -> Result<Project> {
    project.validate()?;
    if !project.layers.iter().any(|layer| layer.id == id) {
        return Err("Camada inexistente.".into());
    }
    let layer = project.layers.iter().find(|l| l.id == id).unwrap().clone();
    let result = Project {
        version: 3,
        crs: project.crs.clone(),
        warnings: layer.warnings.clone(),
        layers: vec![layer],
        lines: project
            .lines
            .iter()
            .filter(|l| l.layer == Some(id))
            .cloned()
            .collect(),
        polygons: project
            .polygons
            .iter()
            .filter(|p| p.layer == Some(id))
            .cloned()
            .collect(),
    };
    if result.lines.is_empty() && result.polygons.is_empty() {
        return Err("Camada sem geometrias para exportar.".into());
    }
    result.validate()?;
    Ok(result)
}
