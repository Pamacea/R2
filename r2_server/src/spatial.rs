//! Spatial Hashing pour Area of Interest (AOI)
//!
//! Permet de ne pas envoyer tout l'état du monde à chaque joueur.
//! Seules les entités proches sont envoyées.

use std::collections::{HashMap, HashSet};

/// Identifiant d'entité spatiale
pub type SpatialEntityId = u64;

/// Grille de hachage spatiale 2D
pub struct SpatialHashGrid<T> {
    /// Taille d'une cellule (unités monde)
    cell_size: f32,
    /// Grille: coordonnées cellule → entités
    grid: HashMap<(i32, i32), Vec<T>>,
    /// Mapping inverse: entité → cellules occupées
    entity_cells: HashMap<T, HashSet<(i32, i32)>>,
}

impl<T> SpatialHashGrid<T>
where
    T: Copy + Eq + std::hash::Hash,
{
    /// Créer une nouvelle grille
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
            entity_cells: HashMap::new(),
        }
    }

    /// Convertir une position monde en coordonnées de cellule
    #[inline]
    fn world_to_cell(&self, x: f32, y: f32) -> (i32, i32) {
        (
            (x / self.cell_size).floor() as i32,
            (y / self.cell_size).floor() as i32,
        )
    }

    /// Insérer une entité à une position
    pub fn insert(&mut self, entity_id: T, x: f32, y: f32) {
        let cell = self.world_to_cell(x, y);

        // Ajouter à la grille
        self.grid.entry(cell).or_default().push(entity_id);

        // Mettre à jour le mapping inverse
        self.entity_cells
            .entry(entity_id)
            .or_default()
            .insert(cell);
    }

    /// Retirer une entité
    pub fn remove(&mut self, entity_id: &T) {
        if let Some(cells) = self.entity_cells.remove(entity_id) {
            for cell in cells {
                if let Some(cell_entities) = self.grid.get_mut(&cell) {
                    cell_entities.retain(|e| e != entity_id);
                }
            }
        }
    }

    /// Déplacer une entité
    pub fn update(&mut self, entity_id: T, _old_x: f32, _old_y: f32, new_x: f32, new_y: f32) {
        self.remove(&entity_id);
        self.insert(entity_id, new_x, new_y);
    }

    /// Rechercher les entités dans un rayon autour d'une position
    pub fn query_radius(&self, x: f32, y: f32, radius: f32) -> Vec<T> {
        let center_cell = self.world_to_cell(x, y);
        let cell_radius = (radius / self.cell_size).ceil() as i32;

        let mut result = Vec::new();
        let mut seen = HashSet::new();

        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                let cell = (center_cell.0 + dx, center_cell.1 + dy);

                if let Some(entities) = self.grid.get(&cell) {
                    for &entity_id in entities {
                        if seen.insert(entity_id) {
                            result.push(entity_id);
                        }
                    }
                }
            }
        }

        result
    }

    /// Obtenir les entités dans les cellules adjacentes (8 voisins + centre)
    pub fn query_adjacent(&self, x: f32, y: f32) -> Vec<T> {
        let center_cell = self.world_to_cell(x, y);

        let mut result = Vec::new();
        let mut seen = HashSet::new();

        for dx in -1..=1 {
            for dy in -1..=1 {
                let cell = (center_cell.0 + dx, center_cell.1 + dy);

                if let Some(entities) = self.grid.get(&cell) {
                    for &entity_id in entities {
                        if seen.insert(entity_id) {
                            result.push(entity_id);
                        }
                    }
                }
            }
        }

        result
    }

    /// Nettoyer les cellules vides
    pub fn cleanup(&mut self) {
        self.grid.retain(|_, entities| !entities.is_empty());
    }

    /// Obtenir le nombre de cellules occupées
    pub fn cell_count(&self) -> usize {
        self.grid.len()
    }

    /// Obtenir le nombre d'entités
    pub fn entity_count(&self) -> usize {
        self.entity_cells.len()
    }
}

impl<T> Default for SpatialHashGrid<T>
where
    T: Copy + Eq + std::hash::Hash,
{
    fn default() -> Self {
        Self::new(100.0) // 100 unités par cellule par défaut
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_query() {
        let mut grid = SpatialHashGrid::new(100.0);

        grid.insert(1, 50.0, 50.0);
        grid.insert(2, 150.0, 50.0);
        grid.insert(3, 1000.0, 1000.0);

        // Query proche de 1 et 2
        let nearby = grid.query_radius(100.0, 50.0, 200.0);
        assert!(nearby.contains(&1));
        assert!(nearby.contains(&2));
        assert!(!nearby.contains(&3));
    }

    #[test]
    fn test_remove() {
        let mut grid = SpatialHashGrid::new(100.0);

        grid.insert(1, 50.0, 50.0);
        grid.insert(2, 150.0, 50.0);

        grid.remove(&1);

        let nearby = grid.query_radius(100.0, 50.0, 200.0);
        assert!(!nearby.contains(&1));
        assert!(nearby.contains(&2));
    }

    #[test]
    fn test_update() {
        let mut grid = SpatialHashGrid::new(100.0);

        grid.insert(1, 50.0, 50.0);
        grid.update(1, 50.0, 50.0, 1000.0, 1000.0);

        let nearby = grid.query_radius(100.0, 50.0, 200.0);
        assert!(!nearby.contains(&1));

        let far = grid.query_radius(1000.0, 1000.0, 200.0);
        assert!(far.contains(&1));
    }
}
