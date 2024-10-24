use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

#[cfg(debug_assertions)]
use log::trace;
use log_err::LogErrOption;

use crate::{
    markers::MarkerCategoryTreeNode,
    points::Point3,
    settings::{TrailColor, TrailWidth},
};

use super::MarkerCategoryTree;

#[derive(Debug)]
pub struct ActiveMarkerCategories<'a> {
    pub active_map_id: u32,
    pub all_points_of_interest: HashMap<u32, Vec<ActivePointOfInterest>>,
    pub all_trails: HashMap<u32, Vec<ActiveTrail<'a>>>,
    empty_poi_vec: Vec<ActivePointOfInterest>,
    empty_trail_vec: Vec<ActiveTrail<'a>>,
}

impl<'a> ActiveMarkerCategories<'a> {
    pub fn new() -> Self {
        Self {
            active_map_id: 0,
            all_points_of_interest: HashMap::default(),
            all_trails: HashMap::default(),
            empty_poi_vec: vec![],
            empty_trail_vec: vec![],
        }
    }

    pub fn read_from_tree(&mut self, tree: &'a MarkerCategoryTree) {
        self.all_points_of_interest.clear();
        self.all_trails.clear();

        fn collect_active_categories<'a>(
            parent: &MarkerCategoryTreeNode<'a>,
            parent_is_active: bool,
            parent_trail_color: &TrailColor,
            parent_trail_width: &TrailWidth,
            all_points_of_interest: &mut HashMap<u32, Vec<ActivePointOfInterest>>,
            all_trails: &mut HashMap<u32, Vec<ActiveTrail<'a>>>,
        ) {
            for child in parent.children() {
                let category = child.data();

                let child_is_active = category.is_active.borrow().unwrap_or(parent_is_active);
                let child_trail_color = category
                    .trail_color
                    .borrow()
                    .unwrap_or_else(|| *parent_trail_color);
                let child_trail_width = category
                    .trail_width
                    .borrow()
                    .unwrap_or_else(|| *parent_trail_width);

                if child_is_active
                    && (!category.points_of_interest.is_empty() || !category.trails.is_empty())
                {
                    for _poi in &category.points_of_interest {
                        // TODO
                    }

                    let mut hasher = DefaultHasher::new();
                    category.identifier.hash(&mut hasher);
                    let hash = hasher.finish();

                    for trail in &category.trails {
                        all_trails
                            .entry(trail.map_id)
                            .or_default()
                            .push(ActiveTrail {
                                #[cfg(debug_assertions)]
                                id: &category.identifier,
                                hash,
                                trail_width: child_trail_width,
                                trail_color: child_trail_color,
                                points: &trail.points,
                            });
                    }
                }

                collect_active_categories(
                    &child,
                    child_is_active,
                    &child_trail_color,
                    &child_trail_width,
                    all_points_of_interest,
                    all_trails,
                );
            }
        }

        let root = tree.tree.root().log_unwrap();
        let root_category = root.data();

        collect_active_categories(
            &root,
            false,
            &root_category.trail_color.borrow().log_unwrap(),
            &root_category.trail_width.borrow().log_unwrap(),
            &mut self.all_points_of_interest,
            &mut self.all_trails,
        );

        #[cfg(debug_assertions)]
        {
            trace!("loaded active marker categories");
            trace!(
                "points of interest: {}",
                self.all_points_of_interest
                    .values()
                    .map(|points_of_interest| points_of_interest.len())
                    .sum::<usize>(),
            );
            trace!(
                "trails: {}",
                self.all_trails
                    .values()
                    .map(|trails| trails.len())
                    .sum::<usize>(),
            );
            trace!(
                "total points: {}",
                self.all_trails
                    .values()
                    .flat_map(|trails| trails.iter().map(|trail| trail.points.len()))
                    .sum::<usize>(),
            );
        }

        self.set_active_map(self.active_map_id);
    }

    pub fn set_active_map(&mut self, map_id: u32) {
        self.active_map_id = map_id;

        #[cfg(debug_assertions)]
        {
            trace!("changed active map to {map_id}");

            const MAX: usize = 10;

            let active_points_of_interest = self.active_points_of_interest();

            trace!(
                "active points of interest ({}): {:?}",
                active_points_of_interest.len(),
                active_points_of_interest
                    .iter()
                    .take(MAX)
                    .collect::<Vec<_>>(),
            );

            let active_trails = self.active_trails();

            trace!(
                "active trails ({}): {:?}",
                active_trails.len(),
                active_trails
                    .iter()
                    .map(|trail| trail.id.join("."))
                    .take(MAX)
                    .collect::<Vec<_>>(),
            );
            trace!(
                "active trail points: {}",
                active_trails
                    .iter()
                    .map(|trail| trail.points.len())
                    .sum::<usize>(),
            )
        }
    }

    pub fn active_points_of_interest(&self) -> &Vec<ActivePointOfInterest> {
        self.all_points_of_interest
            .get(&self.active_map_id)
            .unwrap_or_else(|| &self.empty_poi_vec)
    }

    pub fn active_trails(&self) -> &Vec<ActiveTrail> {
        self.all_trails
            .get(&self.active_map_id)
            .unwrap_or_else(|| &self.empty_trail_vec)
    }
}

#[derive(Debug)]
pub struct ActiveTrail<'a> {
    #[cfg(debug_assertions)]
    pub id: &'a Vec<String>,
    pub hash: u64,
    pub trail_width: TrailWidth,
    pub trail_color: TrailColor,
    pub points: &'a Vec<Point3>,
}

#[derive(Debug)]
pub struct ActivePointOfInterest {
    // TODO
}
