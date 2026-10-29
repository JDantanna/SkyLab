//! Persistent file provenance; geometry keeps its layer through every edit.
use crate::{Project, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<u32>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub hidden: bool,
    pub id: u32,
    pub name: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}
pub fn validate(project: &Project) -> Result<()> {
    if project.layers.len() > 1000 || (!project.layers.is_empty() && project.version < 3) {
        return Err("Estrutura de camadas inválida.".into());
    }
    let mut ids = HashSet::new();
    for layer in &project.layers {
        if layer.id == 0
            || !ids.insert(layer.id)
            || layer.name.trim().is_empty()
            || layer.name.len() > 1024
            || layer.name.chars().any(char::is_control)
            || ![
                "KML",
                "KMZ",
                "MIS",
                "Generated",
                "Project",
                "Drawing",
                "Group",
            ]
            .contains(&layer.source.as_str())
        {
            return Err("Identificação de camada inválida.".into());
        }
    }
    let index: std::collections::HashMap<_, _> = project.layers.iter().map(|l| (l.id, l)).collect();
    if project.version < 5
        && (project
            .layers
            .iter()
            .any(|l| l.parent.is_some() || l.hidden || l.source == "Group")
            || project.lines.iter().any(|l| l.hidden))
    {
        return Err("Layer hierarchy and visibility require project format 5.".into());
    }
    for layer in &project.layers {
        let mut seen = HashSet::from([layer.id]);
        let mut parent = layer.parent;
        while let Some(id) = parent {
            let ancestor = index.get(&id).ok_or("Parent group is unavailable")?;
            if ancestor.source != "Group" || !seen.insert(id) || seen.len() > 32 {
                return Err("Invalid group hierarchy".into());
            }
            parent = ancestor.parent;
        }
    }
    for id in project
        .lines
        .iter()
        .filter_map(|l| l.layer)
        .chain(project.polygons.iter().filter_map(|p| p.layer))
    {
        if !ids.contains(&id) || index[&id].source == "Group" {
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
    let mut included = HashSet::from([id]);
    for _ in 0..32 {
        let before = included.len();
        for layer in &project.layers {
            if layer.parent.is_some_and(|p| included.contains(&p)) {
                included.insert(layer.id);
            }
        }
        if included.len() == before {
            break;
        }
    }
    let mut selected: Vec<_> = project
        .layers
        .iter()
        .filter(|l| included.contains(&l.id))
        .cloned()
        .collect();
    for layer in &mut selected {
        if !layer.parent.is_some_and(|p| included.contains(&p)) {
            layer.parent = None;
        }
    }
    let warnings = selected.iter().flat_map(|l| l.warnings.clone()).collect();
    let result = Project {
        version: project.version.max(3),
        crs: project.crs.clone(),
        warnings,
        layers: selected,
        lines: project
            .lines
            .iter()
            .filter(|l| l.layer.is_some_and(|id| included.contains(&id)))
            .cloned()
            .collect(),
        polygons: project
            .polygons
            .iter()
            .filter(|p| p.layer.is_some_and(|id| included.contains(&id)))
            .cloned()
            .collect(),
    };
    if result.lines.is_empty() && result.polygons.is_empty() {
        return Err("Camada sem geometrias para exportar.".into());
    }
    result.validate()?;
    Ok(result)
}

pub fn visible(project: &Project, mut id: Option<u32>) -> bool {
    for _ in 0..=32 {
        let Some(current) = id else {
            return true;
        };
        let Some(layer) = project.layers.iter().find(|l| l.id == current) else {
            return true;
        };
        if layer.hidden {
            return false;
        }
        id = layer.parent;
    }
    false
}
